#[cfg(feature = "tpm")]
use tss_esapi::{
    attributes::SessionAttributesBuilder,
    constants::SessionType,
    handles::{KeyHandle, SessionHandle},
    interface_types::{
        algorithm::{HashingAlgorithm, PublicAlgorithm},
        resource_handles::Hierarchy,
        session_handles::{AuthSession, PolicySession},
    },
    structures::{
        CreatePrimaryKeyResult, Digest, Public, PublicBuilder, PublicKeyedHashParameters,
        SensitiveData, SymmetricDefinition, SymmetricDefinitionObject,
    },
    traits::{Marshall, UnMarshall},
    Context,
};

#[cfg(feature = "tpm")]
use std::convert::TryInto;
use std::path::PathBuf;

use crate::error::{Result, SecurityError};

pub const SEALED_FILE_NAME: &str = "master_key.tpm";

pub fn default_storage_path() -> PathBuf {
    if let Ok(dir) = std::env::var("POSTAIL_DATA_DIR") {
        return PathBuf::from(dir).join("security");
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
        .join("security")
}

#[cfg(feature = "tpm")]
pub fn tpm_err(e: impl std::fmt::Display) -> SecurityError {
    SecurityError::Tpm(e.to_string())
}

#[cfg(feature = "tpm")]
pub fn create_hmac_session(ctx: &mut Context) -> Result<AuthSession> {
    let session = ctx
        .start_auth_session(
            None,
            None,
            None,
            SessionType::Hmac,
            SymmetricDefinition::AES_128_CFB,
            HashingAlgorithm::Sha256,
        )
        .map_err(tpm_err)?
        .ok_or_else(|| SecurityError::Tpm("failed to create auth session".into()))?;

    let attrs = SessionAttributesBuilder::new()
        .with_decrypt(true)
        .with_encrypt(true)
        .build();

    ctx.tr_sess_set_attributes(session, attrs.0, attrs.1)
        .map_err(tpm_err)?;

    Ok(session)
}

#[cfg(feature = "tpm")]
pub fn create_primary_key(ctx: &mut Context) -> Result<CreatePrimaryKeyResult> {
    let session = create_hmac_session(ctx)?;

    let primary_public = PublicBuilder::new()
        .with_public_algorithm(PublicAlgorithm::SymCipher)
        .with_name_hashing_algorithm(HashingAlgorithm::Sha256)
        .with_object_attributes(
            tss_esapi::attributes::ObjectAttributesBuilder::new()
                .with_fixed_tpm(true)
                .with_fixed_parent(true)
                .with_sensitive_data_origin(true)
                .with_user_with_auth(true)
                .with_decrypt(true)
                .with_restricted(true)
                .build()
                .map_err(tpm_err)?,
        )
        .with_symmetric_cipher_parameters(tss_esapi::structures::SymmetricCipherParameters::new(
            SymmetricDefinitionObject::AES_128_CFB,
        ))
        .with_symmetric_cipher_unique_identifier(Default::default())
        .build()
        .map_err(tpm_err)?;

    ctx.execute_with_session(Some(session), |ctx| {
        ctx.create_primary(Hierarchy::Owner, primary_public, None, None, None, None)
    })
    .map_err(tpm_err)
}

#[cfg(feature = "tpm")]
pub fn compute_pcr_policy_digest(ctx: &mut Context) -> Result<Digest> {
    let trial_session = ctx
        .start_auth_session(
            None,
            None,
            None,
            SessionType::Trial,
            SymmetricDefinition::Null,
            HashingAlgorithm::Sha256,
        )
        .map_err(tpm_err)?
        .ok_or_else(|| SecurityError::Tpm("failed to create trial session".into()))?;

    let pcr_selection = super::pcr::create_pcr_selection_for_boot_state().map_err(tpm_err)?;

    let policy_session: PolicySession = trial_session
        .try_into()
        .map_err(|_| SecurityError::Tpm("failed to extract policy session".into()))?;

    ctx.policy_pcr(policy_session, Digest::default(), pcr_selection)
        .map_err(tpm_err)?;

    let digest = ctx.policy_get_digest(policy_session).map_err(tpm_err)?;

    ctx.flush_context(SessionHandle::from(trial_session).into())
        .map_err(tpm_err)?;

    Ok(digest)
}

#[cfg(feature = "tpm")]
pub fn seal_data(ctx: &mut Context, primary: KeyHandle, data: &[u8]) -> Result<Vec<u8>> {
    let policy_digest = compute_pcr_policy_digest(ctx)?;
    let session = create_hmac_session(ctx)?;

    let seal_public = PublicBuilder::new()
        .with_public_algorithm(PublicAlgorithm::KeyedHash)
        .with_name_hashing_algorithm(HashingAlgorithm::Sha256)
        .with_object_attributes(
            tss_esapi::attributes::ObjectAttributesBuilder::new()
                .with_fixed_tpm(true)
                .with_fixed_parent(true)
                .with_admin_with_policy(true)
                .build()
                .map_err(tpm_err)?,
        )
        .with_keyed_hash_parameters(PublicKeyedHashParameters::new(
            tss_esapi::structures::KeyedHashScheme::Null,
        ))
        .with_keyed_hash_unique_identifier(Digest::default())
        .with_auth_policy(policy_digest)
        .build()
        .map_err(tpm_err)?;

    let sensitive_data = SensitiveData::try_from(data.to_vec()).map_err(tpm_err)?;

    let result = ctx
        .execute_with_session(Some(session), |ctx| {
            ctx.create(primary, seal_public, None, Some(sensitive_data), None, None)
        })
        .map_err(tpm_err)?;

    let priv_bytes = result.out_private.to_vec();
    let pub_bytes = result.out_public.marshall().map_err(tpm_err)?;

    let mut blob = Vec::with_capacity(8 + priv_bytes.len() + pub_bytes.len());
    blob.extend_from_slice(&(priv_bytes.len() as u32).to_le_bytes());
    blob.extend_from_slice(&priv_bytes);
    blob.extend_from_slice(&(pub_bytes.len() as u32).to_le_bytes());
    blob.extend_from_slice(&pub_bytes);

    Ok(blob)
}

#[cfg(feature = "tpm")]
pub fn unseal_data(ctx: &mut Context, primary: KeyHandle, blob: &[u8]) -> Result<Vec<u8>> {
    let (private, public) = parse_sealed_blob(blob)?;

    let load_session = create_hmac_session(ctx)?;
    let sealed_handle = ctx
        .execute_with_session(Some(load_session), |ctx| ctx.load(primary, private, public))
        .map_err(tpm_err)?;

    let result: Result<Vec<u8>> = (|| {
        let policy_auth_session = ctx
            .start_auth_session(
                None,
                None,
                None,
                SessionType::Policy,
                SymmetricDefinition::Null,
                HashingAlgorithm::Sha256,
            )
            .map_err(tpm_err)?
            .ok_or_else(|| SecurityError::Tpm("failed to create policy session".into()))?;

        let policy_session: PolicySession = policy_auth_session
            .try_into()
            .map_err(|_| SecurityError::Tpm("failed to extract policy session".into()))?;

        let pcr_selection = super::pcr::create_pcr_selection_for_boot_state().map_err(tpm_err)?;

        ctx.policy_pcr(policy_session, Digest::default(), pcr_selection)
            .map_err(|e| {
                SecurityError::Tpm(format!(
                    "PCR policy check failed - boot state has changed: {}",
                    e
                ))
            })?;

        let unsealed = ctx
            .execute_with_session(Some(policy_auth_session), |ctx| {
                ctx.unseal(sealed_handle.into())
            })
            .map_err(|e| SecurityError::Tpm(format!("Failed to unseal: {}", e)))?;

        let _ = ctx.flush_context(SessionHandle::from(policy_auth_session).into());

        Ok(unsealed.to_vec())
    })();

    let _ = ctx.flush_context(sealed_handle.into());
    let _ = ctx.flush_context(SessionHandle::from(load_session).into());

    result
}

#[cfg(feature = "tpm")]
pub fn parse_sealed_blob(blob: &[u8]) -> Result<(tss_esapi::structures::Private, Public)> {
    if blob.len() < 8 {
        return Err(SecurityError::Tpm("corrupted sealed blob".into()));
    }

    let priv_len = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    if blob.len() < 4 + priv_len + 4 {
        return Err(SecurityError::Tpm("corrupted sealed blob".into()));
    }

    let priv_bytes = &blob[4..4 + priv_len];
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

    let pub_bytes = &blob[pub_offset + 4..pub_offset + 4 + pub_len];

    let private = tss_esapi::structures::Private::try_from(priv_bytes.to_vec()).map_err(tpm_err)?;
    let public = Public::unmarshall(pub_bytes).map_err(tpm_err)?;

    Ok((private, public))
}
