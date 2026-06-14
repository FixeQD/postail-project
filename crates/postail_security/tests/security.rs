use std::sync::Arc;

use postail_security::{
    SecurityError,
    crypto::{decrypt_with_key, encrypt_with_key},
    manager::{PassphraseSecurityBuilder, SecurityManager},
    master_key::{MasterKey, MASTER_KEY_LENGTH},
    storage::{argon2::Argon2Store, SecretStore, StorageTier},
};
use tempfile::tempdir;

// ============================================================================
// MasterKey tests
// ============================================================================

#[test]
fn master_key_generate_creates_random_key() {
    let key1 = MasterKey::generate();
    let key2 = MasterKey::generate();
    assert_ne!(key1.as_bytes(), key2.as_bytes());
}

#[test]
fn master_key_from_bytes_valid() {
    let bytes = [42u8; MASTER_KEY_LENGTH];
    let key = MasterKey::from_bytes(&bytes).unwrap();
    assert_eq!(key.as_bytes(), &bytes);
}

#[test]
fn master_key_from_bytes_invalid_length() {
    let bytes = [0u8; 16];
    let result = MasterKey::from_bytes(&bytes);
    assert!(matches!(
        result,
        Err(SecurityError::InvalidKeyLength { .. })
    ));
}

#[test]
fn master_key_debug_redacts_key() {
    let key = MasterKey::generate();
    let debug_str = format!("{:?}", key);
    assert!(debug_str.contains("REDACTED"));
}

// ============================================================================
// Crypto tests
// ============================================================================

#[test]
fn crypto_encrypt_decrypt_roundtrip() {
    let key = MasterKey::generate();
    let plaintext = b"secret message for testing purposes";

    let encrypted = encrypt_with_key(&key, plaintext).unwrap();
    let decrypted = decrypt_with_key(&key, &encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn crypto_encrypt_produces_different_output_each_time() {
    let key = MasterKey::generate();
    let plaintext = b"same message";

    let encrypted1 = encrypt_with_key(&key, plaintext).unwrap();
    let encrypted2 = encrypt_with_key(&key, plaintext).unwrap();

    assert_ne!(encrypted1, encrypted2);
}

#[test]
fn crypto_decrypt_with_wrong_key_fails() {
    let key1 = MasterKey::generate();
    let key2 = MasterKey::generate();
    let plaintext = b"secret";

    let encrypted = encrypt_with_key(&key1, plaintext).unwrap();
    let result = decrypt_with_key(&key2, &encrypted);

    assert!(result.is_err());
}

#[test]
fn crypto_decrypt_corrupted_data_fails() {
    let key = MasterKey::generate();
    let plaintext = b"secret";

    let mut encrypted = encrypt_with_key(&key, plaintext).unwrap();
    if let Some(byte) = encrypted.last_mut() {
        *byte ^= 0xFF;
    }

    let result = decrypt_with_key(&key, &encrypted);
    assert!(result.is_err());
}

#[test]
fn crypto_decrypt_too_short_fails() {
    let key = MasterKey::generate();
    let short_data = [0u8; 5];

    let result = decrypt_with_key(&key, &short_data);
    assert!(matches!(result, Err(SecurityError::InvalidNonceLength)));
}

// ============================================================================
// Argon2Store tests
// ============================================================================

#[test]
fn argon2_store_and_retrieve_roundtrip() {
    let dir = tempdir().unwrap();
    let store = Argon2Store::new(dir.path().to_path_buf(), "test-passphrase".into());

    let original_key = MasterKey::generate();
    store.store(&original_key).unwrap();

    let retrieved = store.retrieve().unwrap();
    assert_eq!(original_key.as_bytes(), retrieved.as_bytes());
}

#[test]
fn argon2_wrong_passphrase_fails() {
    let dir = tempdir().unwrap();

    let store1 = Argon2Store::new(dir.path().to_path_buf(), "correct-passphrase".into());
    let original_key = MasterKey::generate();
    store1.store(&original_key).unwrap();

    let store2 = Argon2Store::new(dir.path().to_path_buf(), "wrong-passphrase".into());
    let result = store2.retrieve();

    assert!(matches!(result, Err(SecurityError::InvalidPassphrase)));
}

#[test]
fn argon2_retrieve_nonexistent_fails() {
    let dir = tempdir().unwrap();
    let store = Argon2Store::new(dir.path().to_path_buf(), "passphrase".into());

    let result = store.retrieve();
    assert!(matches!(result, Err(SecurityError::MasterKeyNotFound)));
}

#[test]
fn argon2_delete_removes_file() {
    let dir = tempdir().unwrap();
    let store = Argon2Store::new(dir.path().to_path_buf(), "passphrase".into());

    let key = MasterKey::generate();
    store.store(&key).unwrap();

    let sealed_path = dir.path().join("master_key.sealed");
    assert!(sealed_path.exists());

    store.delete().unwrap();
    assert!(!sealed_path.exists());
}

// ============================================================================
// SecurityManager tests
// ============================================================================

fn create_test_manager() -> (SecurityManager, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let store = Argon2Store::new(dir.path().to_path_buf(), "test-pass".into());
    (
        SecurityManager::with_store(Arc::new(store), StorageTier::Passphrase),
        dir,
    )
}

#[test]
fn manager_initialize_and_unlock() {
    let (mut manager, _dir) = create_test_manager();

    assert!(!manager.is_unlocked());
    manager.initialize().unwrap();
    assert!(manager.is_unlocked());

    manager.lock();
    assert!(!manager.is_unlocked());

    manager.unlock().unwrap();
    assert!(manager.is_unlocked());
}

#[test]
fn manager_encrypt_decrypt_roundtrip() {
    let (mut manager, _dir) = create_test_manager();
    manager.initialize().unwrap();

    let plaintext = b"secret data that needs protection";
    let encrypted = manager.encrypt(plaintext).unwrap();
    let decrypted = manager.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, plaintext);
    assert_ne!(encrypted, plaintext.to_vec());
}

#[test]
fn manager_encrypt_without_unlock_fails() {
    let (manager, _dir) = create_test_manager();
    let result = manager.encrypt(b"test");
    assert!(matches!(result, Err(SecurityError::MasterKeyNotFound)));
}

#[test]
fn manager_double_initialize_fails() {
    let (mut manager, _dir) = create_test_manager();
    manager.initialize().unwrap();

    let result = manager.initialize();
    assert!(matches!(result, Err(SecurityError::MasterKeyAlreadyExists)));
}

#[test]
fn manager_destroy_clears_key() {
    let (mut manager, _dir) = create_test_manager();
    manager.initialize().unwrap();
    assert!(manager.is_unlocked());

    manager.destroy().unwrap();
    assert!(!manager.is_unlocked());
    assert!(!manager.is_initialized());
}

#[test]
fn passphrase_builder_works() {
    let dir = tempdir().unwrap();
    let mut manager =
        PassphraseSecurityBuilder::new(dir.path().to_path_buf(), "my-pass".into()).build();

    manager.initialize().unwrap();
    let encrypted = manager.encrypt(b"hello").unwrap();

    manager.lock();
    manager.unlock().unwrap();

    let decrypted = manager.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, b"hello");
}

pub fn test_manager() -> SecurityManager {
    use std::env::temp_dir;

    let temp_path = temp_dir().join("test_postail_key");
    let store = Argon2Store::new(temp_path, "test_passphrase".to_string());
    let mut manager = SecurityManager::with_store(Arc::new(store), StorageTier::Passphrase);
    let fixed_key = MasterKey::from_bytes(&[0u8; 32]).unwrap();
    manager.initialize_with_key(fixed_key).unwrap();
    manager.unlock().unwrap();
    manager
}
