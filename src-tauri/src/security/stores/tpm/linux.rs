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
    tcti_ldr::TctiNameConf,
    traits::{Marshall, UnMarshall},
    Context,
};

#[cfg(feature = "tpm")]
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;
#[cfg(feature = "tpm")]
use std::str::FromStr;

use crate::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;
use crate::security::stores::SecretStore;

// ── Constants ──────────────────────────────────────────────────────

const SEALED_FILE_NAME: &str = "master_key.tpm";

// ── Helpers ────────────────────────────────────────────────────────

fn tpm_dev_exists() -> bool {
    std::path::Path::new("/dev/tpmrm0").exists() || std::path::Path::new("/dev/tpm0").exists()
}

fn default_storage_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
        .join("security")
}

#[cfg(feature = "tpm")]
fn tpm_err(e: impl std::fmt::Display) -> SecurityError {
    SecurityError::Tpm(e.to_string())
}

/// Creates an HMAC auth session with AES-128-CFB encryption.
#[cfg(feature = "tpm")]
fn create_hmac_session(ctx: &mut Context) -> Result<AuthSession> {
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

// ── LinuxTpmStore ──────────────────────────────────────────────────

pub struct LinuxTpmStore {
    storage_path: PathBuf,
    #[cfg(feature = "tpm")]
    tcti: TctiNameConf,
}

impl LinuxTpmStore {
    pub fn new() -> Result<Self> {
        Self::with_storage_path(default_storage_path())
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Result<Self> {
        #[cfg(feature = "tpm")]
        {
            let tcti = if std::path::Path::new("/dev/tpmrm0").exists() {
                TctiNameConf::from_str("device:/dev/tpmrm0").map_err(tpm_err)?
            } else if std::path::Path::new("/dev/tpm0").exists() {
                TctiNameConf::from_str("device:/dev/tpm0").map_err(tpm_err)?
            } else {
                TctiNameConf::Tabrmd(Default::default())
            };

            Ok(Self { storage_path, tcti })
        }

        #[cfg(not(feature = "tpm"))]
        {
            Ok(Self { storage_path })
        }
    }

    fn get_sealed_path(&self) -> PathBuf {
        self.storage_path.join(SEALED_FILE_NAME)
    }

    // ── Context & availability ─────────────────────────────────────

    #[cfg(feature = "tpm")]
    fn create_context(&self) -> Result<Context> {
        Context::new(self.tcti.clone()).map_err(tpm_err)
    }

    #[cfg(feature = "tpm")]
    pub fn check_context_silent(&self) -> bool {
        tpm_dev_exists() && self.create_context().is_ok()
    }

    #[cfg(feature = "tpm")]
    pub fn check_needs_elevation(&self) -> bool {
        tpm_dev_exists() && !self.check_context_silent()
    }

    // ── Key management ─────────────────────────────────────────────

    #[cfg(feature = "tpm")]
    fn create_primary_key(&self, ctx: &mut Context) -> Result<CreatePrimaryKeyResult> {
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
            .with_symmetric_cipher_parameters(
                tss_esapi::structures::SymmetricCipherParameters::new(
                    SymmetricDefinitionObject::AES_128_CFB,
                ),
            )
            .with_symmetric_cipher_unique_identifier(Default::default())
            .build()
            .map_err(tpm_err)?;

        ctx.execute_with_session(Some(session), |ctx| {
            ctx.create_primary(Hierarchy::Owner, primary_public, None, None, None, None)
        })
        .map_err(tpm_err)
    }

    // ── PCR policy ─────────────────────────────────────────────────

    #[cfg(feature = "tpm")]
    fn compute_pcr_policy_digest(&self, ctx: &mut Context) -> Result<Digest> {
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

    // ── Seal / unseal ──────────────────────────────────────────────

    #[cfg(feature = "tpm")]
    fn seal_data(&self, ctx: &mut Context, primary: KeyHandle, data: &[u8]) -> Result<Vec<u8>> {
        let policy_digest = self.compute_pcr_policy_digest(ctx)?;
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

        // Pack private + public into a single blob: [priv_len][priv][pub_len][pub]
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
    fn unseal_data(&self, ctx: &mut Context, primary: KeyHandle, blob: &[u8]) -> Result<Vec<u8>> {
        let (private, public) = Self::parse_sealed_blob(blob)?;

        // Load sealed object under an HMAC session.
        let load_session = create_hmac_session(ctx)?;
        let sealed_handle = ctx
            .execute_with_session(Some(load_session), |ctx| ctx.load(primary, private, public))
            .map_err(tpm_err)?;

        // Run the actual unseal in a closure so we can flush every live handle unconditionally once the closure returns
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

            // policy_session is a view into policy_auth_session — same handle, different type.
            let policy_session: PolicySession = policy_auth_session
                .try_into()
                .map_err(|_| SecurityError::Tpm("failed to extract policy session".into()))?;

            let pcr_selection =
                super::pcr::create_pcr_selection_for_boot_state().map_err(tpm_err)?;

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

            // Flush policy session — execute_with_session does not flush it for us.
            let _ = ctx.flush_context(SessionHandle::from(policy_auth_session).into());

            Ok(unsealed.to_vec())
        })();

        // Always flush the loaded object and the HMAC load session
        let _ = ctx.flush_context(sealed_handle.into());
        let _ = ctx.flush_context(SessionHandle::from(load_session).into());

        result
    }

    // ── Blob parsing ───────────────────────────────────────────────

    /// Blob format: [priv_len: u32 LE][priv_bytes][pub_len: u32 LE][pub_bytes]
    #[cfg(feature = "tpm")]
    fn parse_sealed_blob(blob: &[u8]) -> Result<(tss_esapi::structures::Private, Public)> {
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

        let private =
            tss_esapi::structures::Private::try_from(priv_bytes.to_vec()).map_err(tpm_err)?;
        let public = Public::unmarshall(pub_bytes).map_err(tpm_err)?;

        Ok((private, public))
    }
}

// ── SecretStore impl ───────────────────────────────────────────────

impl SecretStore for LinuxTpmStore {
    #[cfg(feature = "tpm")]
    fn store(&self, key: &MasterKey) -> Result<()> {
        let mut ctx = self.create_context()?;
        let primary = self.create_primary_key(&mut ctx)?;
        let sealed = self.seal_data(&mut ctx, primary.key_handle, key.as_bytes())?;

        if let Some(parent) = self.get_sealed_path().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(self.get_sealed_path(), sealed)?;

        ctx.flush_context(primary.key_handle.into())
            .map_err(tpm_err)?;
        Ok(())
    }

    #[cfg(not(feature = "tpm"))]
    fn store(&self, _key: &MasterKey) -> Result<()> {
        Err(SecurityError::Tpm("TPM support not compiled in".into()))
    }

    #[cfg(feature = "tpm")]
    fn retrieve(&self) -> Result<MasterKey> {
        let sealed = fs::read(self.get_sealed_path()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SecurityError::MasterKeyNotFound
            } else {
                SecurityError::Io(e)
            }
        })?;

        let mut ctx = self.create_context()?;
        let primary = self.create_primary_key(&mut ctx)?;
        let unsealed = self.unseal_data(&mut ctx, primary.key_handle, &sealed)?;

        ctx.flush_context(primary.key_handle.into())
            .map_err(tpm_err)?;

        MasterKey::from_bytes(&unsealed)
    }

    #[cfg(not(feature = "tpm"))]
    fn retrieve(&self) -> Result<MasterKey> {
        Err(SecurityError::Tpm("TPM support not compiled in".into()))
    }

    fn delete(&self) -> Result<()> {
        let path = self.get_sealed_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    fn exists(&self) -> bool {
        self.get_sealed_path().exists()
    }

    fn is_available(&self) -> bool {
        #[cfg(feature = "tpm")]
        {
            tpm_dev_exists()
        }

        #[cfg(not(feature = "tpm"))]
        {
            false
        }
    }

    fn name(&self) -> &'static str {
        "TPM2 (Linux)"
    }
}
