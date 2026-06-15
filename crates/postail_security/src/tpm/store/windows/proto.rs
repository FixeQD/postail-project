//! Raw TPM2 command builders/parsers used to seal/unseal under a PCR7 policy
//! Big-endian wire format per TCG TPM 2.0 Library Spec

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::tpm::store::pcr::PCR_INDEX_BOOT_STATE;

// ── Tags ──
pub const TPM2_ST_NO_SESSIONS: u16 = 0x8001;
pub const TPM2_ST_SESSIONS: u16 = 0x8002;

// ── Command codes ──
const TPM2_CC_CREATE_PRIMARY: u32 = 0x0000_0131;
const TPM2_CC_CREATE: u32 = 0x0000_0153;
const TPM2_CC_LOAD: u32 = 0x0000_0157;
const TPM2_CC_FLUSH_CONTEXT: u32 = 0x0000_0165;
const TPM2_CC_UNSEAL: u32 = 0x0000_015E;
const TPM2_CC_START_AUTH_SESSION: u32 = 0x0000_0176;
const TPM2_CC_POLICY_PCR: u32 = 0x0000_017F;
const TPM2_CC_POLICY_GET_DIGEST: u32 = 0x0000_0189;
/// TPM2_GetRandom - request random bytes
const TPM2_CC_GET_RANDOM: u32 = 0x0000_017B;

// ── Handles ──
const TPM2_RH_OWNER: u32 = 0x4000_0001;
const TPM2_RH_NULL: u32 = 0x4000_0007;
const TPM2_RS_PW: u32 = 0x4000_0009;

// ── Algorithms ──
const TPM2_ALG_NULL: u16 = 0x0010;
const TPM2_ALG_SHA256: u16 = 0x000B;
const TPM2_ALG_KEYEDHASH: u16 = 0x0008;
const TPM2_ALG_SYMCIPHER: u16 = 0x0025;
const TPM2_ALG_AES: u16 = 0x0006;
const TPM2_ALG_CFB: u16 = 0x0043;

// ── Session types ──
pub const TPM2_SE_POLICY: u8 = 0x01;
pub const TPM2_SE_TRIAL: u8 = 0x03;

// ── TPMA_OBJECT bits ──
const ATTR_FIXED_TPM: u32 = 1 << 1;
const ATTR_FIXED_PARENT: u32 = 1 << 4;
const ATTR_SENSITIVE_DATA_ORIGIN: u32 = 1 << 5;
const ATTR_USER_WITH_AUTH: u32 = 1 << 6;
const ATTR_ADMIN_WITH_POLICY: u32 = 1 << 7;
const ATTR_RESTRICTED: u32 = 1 << 16;
const ATTR_DECRYPT: u32 = 1 << 17;

const PRIMARY_OBJECT_ATTRS: u32 = ATTR_FIXED_TPM
    | ATTR_FIXED_PARENT
    | ATTR_SENSITIVE_DATA_ORIGIN
    | ATTR_USER_WITH_AUTH
    | ATTR_DECRYPT
    | ATTR_RESTRICTED;

const SEAL_OBJECT_ATTRS: u32 = ATTR_FIXED_TPM | ATTR_FIXED_PARENT | ATTR_ADMIN_WITH_POLICY;

// ── Serialization helpers ──

fn write_u16_be(buf: &mut Vec<u8>, val: u16) {
    buf.extend_from_slice(&val.to_be_bytes());
}

fn write_u32_be(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_be_bytes());
}

fn write_tpm2b(buf: &mut Vec<u8>, data: &[u8]) {
    write_u16_be(buf, data.len() as u16);
    buf.extend_from_slice(data);
}

fn write_tpm2b_empty(buf: &mut Vec<u8>) {
    write_u16_be(buf, 0);
}

fn read_u16_be(data: &[u8], off: &mut usize) -> Option<u16> {
    let v = u16::from_be_bytes(data.get(*off..*off + 2)?.try_into().ok()?);
    *off += 2;
    Some(v)
}

fn read_u32_be(data: &[u8], off: &mut usize) -> Option<u32> {
    let v = u32::from_be_bytes(data.get(*off..*off + 4)?.try_into().ok()?);
    *off += 4;
    Some(v)
}

fn read_tpm2b(data: &[u8], off: &mut usize) -> Option<Vec<u8>> {
    let size = read_u16_be(data, off)? as usize;
    let bytes = data.get(*off..*off + size)?.to_vec();
    *off += size;
    Some(bytes)
}

fn build_command(tag: u16, cc: u32, body: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(10 + body.len());
    write_u16_be(&mut buf, tag);
    write_u32_be(&mut buf, (10 + body.len()) as u32);
    write_u32_be(&mut buf, cc);
    buf.extend_from_slice(body);
    buf
}

fn build_password_auth_area() -> Vec<u8> {
    let mut session = Vec::new();
    write_u32_be(&mut session, TPM2_RS_PW);
    write_tpm2b_empty(&mut session); // nonce
    session.push(0x00); // sessionAttributes
    write_tpm2b_empty(&mut session); // hmac (empty = no password)

    let mut auth_area = Vec::new();
    write_u32_be(&mut auth_area, session.len() as u32);
    auth_area.extend_from_slice(&session);
    auth_area
}

fn parse_response(data: &[u8]) -> Result<&[u8], String> {
    if data.len() < 10 {
        return Err("response too short".into());
    }
    let mut off = 0;
    let _tag = read_u16_be(data, &mut off).unwrap();
    let size = read_u32_be(data, &mut off).unwrap() as usize;
    let response_code = read_u32_be(data, &mut off).unwrap();

    if size > data.len() {
        return Err(format!("response size {size} > buffer {}", data.len()));
    }
    if response_code != 0 {
        return Err(format!("TPM2 error: 0x{response_code:08X}"));
    }
    Ok(&data[10..size])
}

fn pcr_selection_boot() -> Vec<u8> {
    let mut buf = Vec::new();
    write_u32_be(&mut buf, 1); // count: 1 bank
    write_u16_be(&mut buf, TPM2_ALG_SHA256);
    buf.push(3); // sizeofSelect = 3 bytes = PCRs 0–23
    // PCR_INDEX_BOOT_STATE is bit index within the 3-byte bitmap
    let byte_idx = (PCR_INDEX_BOOT_STATE / 8) as usize;
    let bit_idx = PCR_INDEX_BOOT_STATE % 8;
    let mut bitmap = [0u8; 3];
    bitmap[byte_idx] = 1 << bit_idx;
    buf.extend_from_slice(&bitmap);
    buf
}

/// AES-128-CFB symmetric storage primary key (restricted decrypt) under Owner
fn build_primary_template() -> Vec<u8> {
    let mut t = Vec::new();
    write_u16_be(&mut t, TPM2_ALG_SYMCIPHER);
    write_u16_be(&mut t, TPM2_ALG_SHA256); // nameAlg
    write_u32_be(&mut t, PRIMARY_OBJECT_ATTRS);
    write_tpm2b_empty(&mut t); // authPolicy
    // TPMS_SYMCIPHER_PARMS
    write_u16_be(&mut t, TPM2_ALG_AES);
    write_u16_be(&mut t, 128); // keyBits
    write_u16_be(&mut t, TPM2_ALG_CFB);
    write_tpm2b_empty(&mut t); // unique
    t
}

/// KeyedHash sealed-data object gated by `auth_policy`
fn build_seal_template(auth_policy: &[u8]) -> Vec<u8> {
    let mut t = Vec::new();
    write_u16_be(&mut t, TPM2_ALG_KEYEDHASH);
    write_u16_be(&mut t, TPM2_ALG_SHA256); // nameAlg
    write_u32_be(&mut t, SEAL_OBJECT_ATTRS);
    write_tpm2b(&mut t, auth_policy);
    write_u16_be(&mut t, TPM2_ALG_NULL); // TPMS_KEYEDHASH_PARMS.scheme
    write_tpm2b_empty(&mut t); // unique
    t
}

/// Name of an object: nameAlg || H(TPMT_PUBLIC), used in cpHash for Unseal
pub fn object_name(pub_bytes: &[u8]) -> Vec<u8> {
    let digest = Sha256::digest(pub_bytes);
    let mut name = Vec::with_capacity(2 + 32);
    write_u16_be(&mut name, TPM2_ALG_SHA256);
    name.extend_from_slice(&digest);
    name
}

// ── TPM2_CreatePrimary ──

pub fn cmd_create_primary() -> Vec<u8> {
    let mut body = Vec::new();
    write_u32_be(&mut body, TPM2_RH_OWNER);
    body.extend_from_slice(&build_password_auth_area());
    // inSensitive: TPM2B_SENSITIVE_CREATE { userAuth: empty, data: empty } -> 4 bytes
    write_u16_be(&mut body, 4);
    write_tpm2b_empty(&mut body); // userAuth
    write_tpm2b_empty(&mut body); // data
    write_tpm2b(&mut body, &build_primary_template()); // inPublic
    write_tpm2b_empty(&mut body); // outsideInfo
    write_u32_be(&mut body, 0); // creationPCR: empty
    build_command(TPM2_ST_SESSIONS, TPM2_CC_CREATE_PRIMARY, &body)
}

pub fn parse_create_primary(resp: &[u8]) -> Result<u32, String> {
    let body = parse_response(resp)?;
    let mut off = 0;
    read_u32_be(body, &mut off).ok_or_else(|| "short response".into())
}

// ── TPM2_Create (seal) ──

pub fn cmd_create(parent: u32, auth_policy: &[u8], secret: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write_u32_be(&mut body, parent);
    body.extend_from_slice(&build_password_auth_area());
    // inSensitive: TPM2B_SENSITIVE_CREATE { userAuth: empty, data: secret }
    write_u16_be(&mut body, (2 + 2 + secret.len()) as u16);
    write_tpm2b_empty(&mut body); // userAuth
    write_tpm2b(&mut body, secret); // data
    write_tpm2b(&mut body, &build_seal_template(auth_policy)); // inPublic
    write_tpm2b_empty(&mut body); // outsideInfo
    write_u32_be(&mut body, 0); // creationPCR: empty
    build_command(TPM2_ST_SESSIONS, TPM2_CC_CREATE, &body)
}

pub fn parse_create(resp: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let body = parse_response(resp)?;
    let mut off = 0;
    let _param_size = read_u32_be(body, &mut off).ok_or("short response")?;
    let out_private = read_tpm2b(body, &mut off).ok_or("short response")?;
    let out_public = read_tpm2b(body, &mut off).ok_or("short response")?;
    Ok((out_private, out_public))
}

// ── TPM2_Load ──

pub fn cmd_load(parent: u32, priv_bytes: &[u8], pub_bytes: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write_u32_be(&mut body, parent);
    body.extend_from_slice(&build_password_auth_area());
    write_tpm2b(&mut body, priv_bytes);
    write_tpm2b(&mut body, pub_bytes);
    build_command(TPM2_ST_SESSIONS, TPM2_CC_LOAD, &body)
}

pub fn parse_load(resp: &[u8]) -> Result<u32, String> {
    let body = parse_response(resp)?;
    let mut off = 0;
    read_u32_be(body, &mut off).ok_or_else(|| "short response".into())
}

// ── TPM2_StartAuthSession ──

pub fn cmd_start_auth_session(session_type: u8, nonce_caller: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write_u32_be(&mut body, TPM2_RH_NULL); // tpmKey
    write_u32_be(&mut body, TPM2_RH_NULL); // bind
    write_tpm2b(&mut body, nonce_caller);
    write_tpm2b_empty(&mut body); // encryptedSalt
    body.push(session_type);
    write_u16_be(&mut body, TPM2_ALG_NULL); // symmetric: NULL
    write_u16_be(&mut body, TPM2_ALG_SHA256); // authHash
    build_command(TPM2_ST_NO_SESSIONS, TPM2_CC_START_AUTH_SESSION, &body)
}

pub fn parse_start_auth_session(resp: &[u8]) -> Result<(u32, Vec<u8>), String> {
    let body = parse_response(resp)?;
    let mut off = 0;
    let handle = read_u32_be(body, &mut off).ok_or("short response")?;
    let nonce_tpm = read_tpm2b(body, &mut off).ok_or("short response")?;
    Ok((handle, nonce_tpm))
}

// ── TPM2_PolicyPCR / TPM2_PolicyGetDigest ──

pub fn cmd_policy_pcr(session: u32) -> Vec<u8> {
    let mut body = Vec::new();
    write_u32_be(&mut body, session);
    write_tpm2b_empty(&mut body); // pcrDigest
    body.extend_from_slice(&pcr_selection_boot());
    build_command(TPM2_ST_NO_SESSIONS, TPM2_CC_POLICY_PCR, &body)
}

pub fn cmd_policy_get_digest(session: u32) -> Vec<u8> {
    let mut body = Vec::new();
    write_u32_be(&mut body, session);
    build_command(TPM2_ST_NO_SESSIONS, TPM2_CC_POLICY_GET_DIGEST, &body)
}

pub fn parse_policy_get_digest(resp: &[u8]) -> Result<Vec<u8>, String> {
    let body = parse_response(resp)?;
    let mut off = 0;
    read_tpm2b(body, &mut off).ok_or_else(|| "short response".into())
}

// ── TPM2_FlushContext ──

pub fn cmd_flush_context(handle: u32) -> Vec<u8> {
    let mut body = Vec::new();
    write_u32_be(&mut body, handle);
    build_command(TPM2_ST_NO_SESSIONS, TPM2_CC_FLUSH_CONTEXT, &body)
}

// ── TPM2_Unseal ──

/// Builds the Unseal command, computing the policy session's auth HMAC
pub fn cmd_unseal(
    item_handle: u32,
    session: u32,
    nonce_caller: &[u8],
    nonce_tpm: &[u8],
    name: &[u8],
) -> Vec<u8> {
    let mut cp = Vec::new();
    write_u32_be(&mut cp, TPM2_CC_UNSEAL);
    cp.extend_from_slice(name);
    let p_hash = Sha256::digest(&cp);

    let session_attrs: u8 = 0x00; // continueSession = clear

    let mut hmac_msg = Vec::new();
    hmac_msg.extend_from_slice(&p_hash);
    hmac_msg.extend_from_slice(nonce_caller); // nonceNewer
    hmac_msg.extend_from_slice(nonce_tpm); // nonceOlder
    hmac_msg.push(session_attrs);

    // sessionKey and authValue are both empty -> HMAC key is the empty string
    let mut mac = Hmac::<Sha256>::new_from_slice(&[]).expect("HMAC accepts empty key");
    mac.update(&hmac_msg);
    let auth_hmac = mac.finalize().into_bytes();

    let mut auth_area = Vec::new();
    write_u32_be(&mut auth_area, session);
    write_tpm2b(&mut auth_area, nonce_caller);
    auth_area.push(session_attrs);
    write_tpm2b(&mut auth_area, &auth_hmac);

    let mut body = Vec::new();
    write_u32_be(&mut body, item_handle);
    write_u32_be(&mut body, auth_area.len() as u32);
    body.extend_from_slice(&auth_area);

    build_command(TPM2_ST_SESSIONS, TPM2_CC_UNSEAL, &body)
}

pub fn parse_unseal(resp: &[u8]) -> Result<Vec<u8>, String> {
    let body = parse_response(resp)?;
    let mut off = 0;
    let _param_size = read_u32_be(body, &mut off).ok_or("short response")?;
    read_tpm2b(body, &mut off).ok_or_else(|| "short response".into())
}

/// Builds `TPM2_GetRandom { bytesRequested }`
pub fn cmd_get_random(num_bytes: u16) -> Vec<u8> {
    let mut body = Vec::new();
    write_u16_be(&mut body, num_bytes);
    build_command(TPM2_ST_NO_SESSIONS, TPM2_CC_GET_RANDOM, &body)
}

/// Parses `TPM2_GetRandom` response and returns the received bytes
/// Returns error if response_code != 0 or the response is truncated
pub fn parse_get_random(resp: &[u8]) -> Result<Vec<u8>, String> {
    let body = parse_response(resp)?; // validates response_code == 0
    let mut off = 0;
    read_tpm2b(body, &mut off).ok_or_else(|| "short GetRandom response".into())
}
