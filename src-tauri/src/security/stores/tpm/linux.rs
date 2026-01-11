#[cfg(feature = "tpm")]
use tss_esapi::{
    attributes::SessionAttributesBuilder,
    constants::SessionType,
    handles::KeyHandle,
    interface_types::{
        algorithm::{HashingAlgorithm, PublicAlgorithm},
        resource_handles::Hierarchy,
        session_handles::AuthSession,
    },
    structures::{
        CreatePrimaryKeyResult, Data, Digest, MaxBuffer, Public, PublicBuilder,
        SymmetricDefinitionObject,
    },
    tcti_ldr::{DeviceConfig, TctiNameConf},
    Context,
};

use std::fs;
use std::path::PathBuf;

use crate::error::{Result, SecurityError};
use crate::security::master_key::MasterKey;
use crate::security::stores::SecretStore;

const SEALED_FILE_NAME: &str = "master_key.tpm";

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
                TctiNameConf::Device(DeviceConfig::from("/dev/tpmrm0"))
            } else if std::path::Path::new("/dev/tpm0").exists() {
                TctiNameConf::Device(DeviceConfig::from("/dev/tpm0"))
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

    #[cfg(feature = "tpm")]
    fn create_context(&self) -> Result<Context> {
        Context::new(self.tcti.clone()).map_err(|e| SecurityError::Tpm(e.to_string()))
    }

    #[cfg(feature = "tpm")]
    fn create_primary_key(&self, ctx: &mut Context) -> Result<CreatePrimaryKeyResult> {
        let session = ctx
            .start_auth_session(
                None,
                None,
                None,
                SessionType::Hmac,
                SymmetricDefinitionObject::AES_256_CFB,
                HashingAlgorithm::Sha256,
            )
            .map_err(|e| SecurityError::Tpm(e.to_string()))?
            .ok_or_else(|| SecurityError::Tpm("failed to create auth session".into()))?;

        let session_attrs = SessionAttributesBuilder::new()
            .with_decrypt(true)
            .with_encrypt(true)
            .build();

        ctx.tr_sess_set_attributes(session, session_attrs.0, session_attrs.1)
            .map_err(|e| SecurityError::Tpm(e.to_string()))?;

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
                    .map_err(|e| SecurityError::Tpm(e.to_string()))?,
            )
            .with_symmetric_cipher_parameters(
                tss_esapi::structures::SymmetricCipherParameters::new(
                    SymmetricDefinitionObject::AES_256_CFB,
                ),
            )
            .with_symmetric_cipher_unique_identifier(Default::default())
            .build()
            .map_err(|e| SecurityError::Tpm(e.to_string()))?;

        ctx.execute_with_session(Some(session), |ctx| {
            ctx.create_primary(Hierarchy::Owner, primary_public, None, None, None, None)
        })
        .map_err(|e| SecurityError::Tpm(e.to_string()))
    }

    #[cfg(feature = "tpm")]
    fn seal_data(&self, ctx: &mut Context, primary: KeyHandle, data: &[u8]) -> Result<Vec<u8>> {
        let session = ctx
            .start_auth_session(
                None,
                None,
                None,
                SessionType::Hmac,
                SymmetricDefinitionObject::AES_256_CFB,
                HashingAlgorithm::Sha256,
            )
            .map_err(|e| SecurityError::Tpm(e.to_string()))?
            .ok_or_else(|| SecurityError::Tpm("failed to create auth session".into()))?;

        let seal_public = PublicBuilder::new()
            .with_public_algorithm(PublicAlgorithm::KeyedHash)
            .with_name_hashing_algorithm(HashingAlgorithm::Sha256)
            .with_object_attributes(
                tss_esapi::attributes::ObjectAttributesBuilder::new()
                    .with_fixed_tpm(true)
                    .with_fixed_parent(true)
                    .with_user_with_auth(true)
                    .build()
                    .map_err(|e| SecurityError::Tpm(e.to_string()))?,
            )
            .with_keyed_hash_parameters(tss_esapi::structures::KeyedHashParameters::new(
                tss_esapi::structures::KeyedHashScheme::Null,
            ))
            .with_keyed_hash_unique_identifier(Digest::default())
            .build()
            .map_err(|e| SecurityError::Tpm(e.to_string()))?;

        let sensitive_data =
            MaxBuffer::try_from(data.to_vec()).map_err(|e| SecurityError::Tpm(e.to_string()))?;

        let result = ctx
            .execute_with_session(Some(session), |ctx| {
                ctx.create(
                    primary,
                    seal_public,
                    None,
                    Some(sensitive_data.into()),
                    None,
                    None,
                )
            })
            .map_err(|e| SecurityError::Tpm(e.to_string()))?;

        let mut blob = Vec::new();
        let priv_bytes = result.out_private.to_vec();
        let pub_bytes: Vec<u8> = result
            .out_public
            .try_into()
            .map_err(|e: tss_esapi::Error| SecurityError::Tpm(e.to_string()))?;

        blob.extend_from_slice(&(priv_bytes.len() as u32).to_le_bytes());
        blob.extend_from_slice(&priv_bytes);
        blob.extend_from_slice(&(pub_bytes.len() as u32).to_le_bytes());
        blob.extend_from_slice(&pub_bytes);

        Ok(blob)
    }

    #[cfg(feature = "tpm")]
    fn unseal_data(&self, ctx: &mut Context, primary: KeyHandle, blob: &[u8]) -> Result<Vec<u8>> {
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

        let private = tss_esapi::structures::Private::try_from(priv_bytes.to_vec())
            .map_err(|e| SecurityError::Tpm(e.to_string()))?;
        let public = Public::try_from(pub_bytes).map_err(|e| SecurityError::Tpm(e.to_string()))?;

        let session = ctx
            .start_auth_session(
                None,
                None,
                None,
                SessionType::Hmac,
                SymmetricDefinitionObject::AES_256_CFB,
                HashingAlgorithm::Sha256,
            )
            .map_err(|e| SecurityError::Tpm(e.to_string()))?
            .ok_or_else(|| SecurityError::Tpm("failed to create auth session".into()))?;

        let sealed_handle = ctx
            .execute_with_session(Some(session), |ctx| ctx.load(primary, private, public))
            .map_err(|e| SecurityError::Tpm(e.to_string()))?;

        let unsealed = ctx
            .execute_with_session(Some(session), |ctx| ctx.unseal(sealed_handle.into()))
            .map_err(|e| SecurityError::Tpm(e.to_string()))?;

        Ok(unsealed.to_vec())
    }
}

impl SecretStore for LinuxTpmStore {
    #[cfg(feature = "tpm")]
    fn store(&self, key: &MasterKey) -> Result<()> {
        let mut ctx = self.create_context()?;
        let primary_result = self.create_primary_key(&mut ctx)?;
        let sealed = self.seal_data(&mut ctx, primary_result.key_handle, key.as_bytes())?;

        if let Some(parent) = self.get_sealed_path().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(self.get_sealed_path(), sealed)?;

        ctx.flush_context(primary_result.key_handle.into())
            .map_err(|e| SecurityError::Tpm(e.to_string()))?;

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
        let primary_result = self.create_primary_key(&mut ctx)?;
        let unsealed = self.unseal_data(&mut ctx, primary_result.key_handle, &sealed)?;

        ctx.flush_context(primary_result.key_handle.into())
            .map_err(|e| SecurityError::Tpm(e.to_string()))?;

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

    fn is_available(&self) -> bool {
        #[cfg(feature = "tpm")]
        {
            self.create_context().is_ok()
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

fn default_storage_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("postail")
        .join("security")
}
