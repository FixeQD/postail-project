//! Tests for BIP39 recovery key subsystem:
//!   - phrase generation (12-word, valid BIP39 English)
//!   - key derivation determinism and isolation
//!   - RecoveryStore create / unlock / wrong-phrase / missing-file
//!   - pending-phrase store / get / clear / verify flow

use std::collections::HashSet;

use postail_project_lib::security::{
    master_key::{MasterKey, MASTER_KEY_LENGTH},
    recovery::{
        clear_pending_phrase, derive_recovery_key, generate_phrase, get_pending_phrase,
        store_pending_phrase, verify_pending_phrase, RecoveryStore,
    },
};
use tempfile::tempdir;

// ── generate_phrase ───────────────────────────────────────────────────────────

#[test]
fn phrase_has_12_words() {
    let phrase = generate_phrase();
    assert_eq!(phrase.split_whitespace().count(), 12);
}

#[test]
fn phrase_words_are_lowercase_ascii() {
    let phrase = generate_phrase();
    for word in phrase.split_whitespace() {
        assert!(word.chars().all(|c| c.is_ascii_lowercase()), "word not lowercase ascii: {word}");
    }
}

#[test]
fn two_generated_phrases_are_different() {
    let p1 = generate_phrase();
    let p2 = generate_phrase();
    // Astronomically unlikely to be equal, but possible — repeat a few times for safety
    let phrases: HashSet<_> = (0..5).map(|_| generate_phrase()).collect();
    assert!(phrases.len() > 1, "All generated phrases were identical");
    let _ = (p1, p2);
}

// ── derive_recovery_key ───────────────────────────────────────────────────────

#[test]
fn derived_key_is_32_bytes() {
    let phrase = generate_phrase();
    let key = derive_recovery_key(&phrase).unwrap();
    assert_eq!(key.as_bytes().len(), MASTER_KEY_LENGTH);
}

#[test]
fn derivation_is_deterministic() {
    let phrase = generate_phrase();
    let k1 = derive_recovery_key(&phrase).unwrap();
    let k2 = derive_recovery_key(&phrase).unwrap();
    assert_eq!(k1.as_bytes(), k2.as_bytes());
}

#[test]
fn different_phrases_produce_different_keys() {
    let p1 = generate_phrase();
    let p2 = generate_phrase();
    // Guard against the astronomically unlikely collision
    if p1 == p2 {
        return;
    }
    let k1 = derive_recovery_key(&p1).unwrap();
    let k2 = derive_recovery_key(&p2).unwrap();
    assert_ne!(k1.as_bytes(), k2.as_bytes());
}

#[test]
fn derivation_trims_whitespace() {
    let phrase = generate_phrase();
    let k1 = derive_recovery_key(&phrase).unwrap();
    let k2 = derive_recovery_key(&format!("  {}  ", phrase)).unwrap();
    assert_eq!(k1.as_bytes(), k2.as_bytes());
}

#[test]
fn invalid_phrase_returns_error() {
    let result = derive_recovery_key("not a valid bip39 phrase at all ever never");
    assert!(result.is_err(), "Should have returned Err for invalid phrase");
}

#[test]
fn empty_phrase_returns_error() {
    assert!(derive_recovery_key("").is_err());
}

// ── RecoveryStore create / unlock ─────────────────────────────────────────────

fn make_store() -> (RecoveryStore, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let store = RecoveryStore::new(dir.path().to_path_buf());
    (store, dir)
}

#[test]
fn recovery_store_does_not_exist_initially() {
    let (store, _dir) = make_store();
    assert!(!store.exists());
}

#[test]
fn recovery_store_exists_after_create() {
    let phrase = generate_phrase();
    let master_key = MasterKey::generate();
    let (store, _dir) = make_store();

    store.create(&master_key, &phrase).unwrap();
    assert!(store.exists());
}

#[test]
fn recovery_store_unlock_roundtrip() {
    let phrase = generate_phrase();
    let master_key = MasterKey::generate();
    let original_bytes = *master_key.as_bytes();

    let (store, _dir) = make_store();
    store.create(&master_key, &phrase).unwrap();

    let recovered = store.unlock(&phrase).unwrap();
    assert_eq!(recovered.as_bytes(), &original_bytes);
}

#[test]
fn recovery_store_wrong_phrase_fails() {
    let phrase = generate_phrase();
    let wrong_phrase = generate_phrase();

    // Retry in the unlikely event they matched
    if phrase == wrong_phrase {
        return;
    }

    let master_key = MasterKey::generate();
    let (store, _dir) = make_store();
    store.create(&master_key, &phrase).unwrap();

    let result = store.unlock(&wrong_phrase);
    assert!(result.is_err(), "Unlocking with wrong phrase should fail");
}

#[test]
fn recovery_store_unlock_missing_file_fails() {
    let (store, _dir) = make_store();
    let phrase = generate_phrase();
    let result = store.unlock(&phrase);
    assert!(result.is_err());
}

#[test]
fn recovery_store_create_overwrites() {
    let phrase = generate_phrase();
    let key1 = MasterKey::generate();
    let key2 = MasterKey::generate();

    let (store, _dir) = make_store();
    store.create(&key1, &phrase).unwrap();
    store.create(&key2, &phrase).unwrap(); // overwrite

    let recovered = store.unlock(&phrase).unwrap();
    assert_eq!(recovered.as_bytes(), key2.as_bytes());
}

// ── pending-phrase API ────────────────────────────────────────────────────────

// NOTE: these tests are inherently sequential because they share global state
// (PENDING_PHRASE). Run with --test-threads=1 if flaky.

#[test]
fn pending_phrase_lifecycle() {
    clear_pending_phrase();

    // Initially absent
    assert_eq!(get_pending_phrase(), None);

    // Store
    store_pending_phrase("word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12".to_string());
    assert!(get_pending_phrase().is_some());

    // Clear
    clear_pending_phrase();
    assert_eq!(get_pending_phrase(), None);
}

#[test]
fn verify_pending_phrase_correct_words() {
    clear_pending_phrase();
    let phrase = "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima";
    store_pending_phrase(phrase.to_string());

    // Verify indices 0, 2, 4 match correctly
    let result = verify_pending_phrase(
        &[0, 2, 4],
        &["alpha".to_string(), "charlie".to_string(), "echo".to_string()],
    )
    .unwrap();
    assert!(result);

    clear_pending_phrase();
}

#[test]
fn verify_pending_phrase_wrong_word() {
    clear_pending_phrase();
    let phrase = "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima";
    store_pending_phrase(phrase.to_string());

    let result = verify_pending_phrase(
        &[0],
        &["bravo".to_string()], // wrong: index 0 is "alpha"
    )
    .unwrap();
    assert!(!result);

    clear_pending_phrase();
}

#[test]
fn verify_pending_phrase_out_of_bounds_index() {
    clear_pending_phrase();
    let phrase = "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima";
    store_pending_phrase(phrase.to_string());

    let result = verify_pending_phrase(
        &[99], // out of range
        &["alpha".to_string()],
    )
    .unwrap();
    assert!(!result);

    clear_pending_phrase();
}

#[test]
fn verify_pending_phrase_no_pending_phrase() {
    clear_pending_phrase();
    // No phrase stored → should return Ok(false)
    let result = verify_pending_phrase(&[0], &["word".to_string()]).unwrap();
    assert!(!result);
}

#[test]
fn verify_pending_phrase_case_insensitive_word_match() {
    clear_pending_phrase();
    let phrase = "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima";
    store_pending_phrase(phrase.to_string());

    // trim().to_lowercase() in verify_pending_phrase means "  ALPHA  " should match
    let result = verify_pending_phrase(&[0], &["  ALPHA  ".to_string()]).unwrap();
    assert!(result);

    clear_pending_phrase();
}
