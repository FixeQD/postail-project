//! High-level seal/unseal over raw TBS
//! Sealed blob format: [u32 le priv_len][priv bytes][u32 le pub_len][pub bytes].

use rand::RngCore;

use crate::error::{Result, SecurityError};
use crate::tpm::store::paths::tpm_err;

use super::proto;
use super::tbs::TbsContext;

fn submit(tbs: &TbsContext, cmd: &[u8]) -> Result<Vec<u8>> {
    tbs.submit(cmd)
}

fn fresh_nonce() -> [u8; 32] {
    let mut nonce = [0u8; 32];
    rand::rng().fill_bytes(&mut nonce);
    nonce
}

/// Create the AES-128-CFB storage primary key under the Owner hierarchy
pub fn create_primary_key(tbs: &TbsContext) -> Result<u32> {
    let resp = submit(tbs, &proto::cmd_create_primary())?;
    proto::parse_create_primary(&resp).map_err(tpm_err)
}

pub fn flush_context(tbs: &TbsContext, handle: u32) -> Result<()> {
    submit(tbs, &proto::cmd_flush_context(handle))?;
    Ok(())
}

/// Run TPM2_PolicyPCR in a trial session, return the resulting policy digest
fn compute_pcr_policy_digest(tbs: &TbsContext) -> Result<Vec<u8>> {
    let resp = submit(
        tbs,
        &proto::cmd_start_auth_session(proto::TPM2_SE_TRIAL, &fresh_nonce()),
    )?;
    let (session, _nonce_tpm) = proto::parse_start_auth_session(&resp).map_err(tpm_err)?;

    submit(tbs, &proto::cmd_policy_pcr(session))?;

    let resp = submit(tbs, &proto::cmd_policy_get_digest(session))?;
    let digest = proto::parse_policy_get_digest(&resp).map_err(tpm_err)?;

    flush_context(tbs, session)?;
    Ok(digest)
}

/// Seal `data` under `primary`, gated by the current PCR7 (Secure Boot) state.
pub fn seal_data(tbs: &TbsContext, primary: u32, data: &[u8]) -> Result<Vec<u8>> {
    let policy_digest = compute_pcr_policy_digest(tbs)?;

    let resp = submit(tbs, &proto::cmd_create(primary, &policy_digest, data))?;
    let (priv_bytes, pub_bytes) = proto::parse_create(&resp).map_err(tpm_err)?;

    let mut blob = Vec::with_capacity(8 + priv_bytes.len() + pub_bytes.len());
    blob.extend_from_slice(&(priv_bytes.len() as u32).to_le_bytes());
    blob.extend_from_slice(&priv_bytes);
    blob.extend_from_slice(&(pub_bytes.len() as u32).to_le_bytes());
    blob.extend_from_slice(&pub_bytes);
    Ok(blob)
}

fn parse_sealed_blob(blob: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    if blob.len() < 8 {
        return Err(SecurityError::Tpm("corrupted sealed blob".into()));
    }

    let priv_len = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    if blob.len() < 4 + priv_len + 4 {
        return Err(SecurityError::Tpm("corrupted sealed blob".into()));
    }
    let priv_bytes = blob[4..4 + priv_len].to_vec();

    let pub_offset = 4 + priv_len;
    let pub_len = u32::from_le_bytes([
        blob[pub_offset],
        blob[pub_offset + 1],
        blob[pub_offset + 2],
        blob[pub_offset + 3],
    ]) as usize;
    if blob.len() < pub_offset + 4 + pub_len {
        return Err(SecurityError::Tpm("corrupted sealed blob".into()));
    }
    let pub_bytes = blob[pub_offset + 4..pub_offset + 4 + pub_len].to_vec();

    Ok((priv_bytes, pub_bytes))
}

/// Unseal a blob produced by [`seal_data`]
pub fn unseal_data(tbs: &TbsContext, primary: u32, blob: &[u8]) -> Result<Vec<u8>> {
    let (priv_bytes, pub_bytes) = parse_sealed_blob(blob)?;

    let resp = submit(tbs, &proto::cmd_load(primary, &priv_bytes, &pub_bytes))?;
    let item_handle = proto::parse_load(&resp).map_err(tpm_err)?;

    let result: Result<Vec<u8>> = (|| {
        let resp = submit(
            tbs,
            &proto::cmd_start_auth_session(proto::TPM2_SE_POLICY, &fresh_nonce()),
        )?;
        let (session, nonce_tpm) = proto::parse_start_auth_session(&resp).map_err(tpm_err)?;

        submit(tbs, &proto::cmd_policy_pcr(session)).map_err(|e| {
            SecurityError::Tpm(format!(
                "PCR policy check failed - boot state has changed: {e}"
            ))
        })?;

        let name = proto::object_name(&pub_bytes);
        let resp = submit(
            tbs,
            &proto::cmd_unseal(item_handle, session, &fresh_nonce(), &nonce_tpm, &name),
        )
        .map_err(|e| SecurityError::Tpm(format!("Failed to unseal: {e}")))?;

        // policy session is auto-flushed by the TPM (continueSession = 0)
        proto::parse_unseal(&resp).map_err(tpm_err)
    })();

    let _ = flush_context(tbs, item_handle);
    result
}
