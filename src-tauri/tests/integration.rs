//! Integration tests for the SaladVault crypto pipeline.
//!
//! Tests the full Dual-Lock cycle:
//!   generate device key -> save/load -> reconstruct master key -> encrypt -> decrypt
//! Also validates blind index consistency across modules.

use rust_app_lib::crypto::{argon2_kdf, blind_index, keys, xchacha};
use tempfile::TempDir;

/// Full Dual-Lock cycle: key generation, persistence, master key reconstruction,
/// and XChaCha20-Poly1305 encrypt/decrypt roundtrip.
#[test]
fn test_dual_lock_full_cycle() {
    let tmp = TempDir::new().unwrap();
    let key_path = tmp.path().join("device_secret.key");

    // 1. Generate and save the device key
    let device_key = keys::generate_device_key();
    keys::save_device_key(&device_key, &key_path).unwrap();

    // 2. Load it back and verify identity
    let loaded_key = keys::load_device_key(&key_path).unwrap();
    assert_eq!(device_key, loaded_key);

    // 3. Reconstruct master key via Argon2id + HKDF
    let password = b"MyStr0ngP@ssw0rd!";
    let salt = argon2_kdf::generate_salt();
    let master_key = keys::reconstruct_master_key(password, &device_key, &salt).unwrap();

    // 4. Encrypt some data (simulating a Feuille)
    let plaintext = b"username: alice\npassword: s3cret_v4lue";
    let (nonce, ciphertext) = xchacha::encrypt(master_key.as_bytes(), plaintext).unwrap();

    // 5. Decrypt and verify roundtrip
    let decrypted = xchacha::decrypt(master_key.as_bytes(), &nonce, &ciphertext).unwrap();
    assert_eq!(decrypted, plaintext);

    // 6. Wrong key must fail decryption
    let wrong_key = [0xFFu8; 32];
    assert!(xchacha::decrypt(&wrong_key, &nonce, &ciphertext).is_err());
}

/// Verify that reconstructing the master key is deterministic:
/// same inputs always produce the same key.
#[test]
fn test_master_key_deterministic_reconstruction() {
    let password = b"deterministic_test";
    let device_key = [0xABu8; 32];
    let salt = [0xCDu8; 32];

    let mk1 = keys::reconstruct_master_key(password, &device_key, &salt).unwrap();
    let mk2 = keys::reconstruct_master_key(password, &device_key, &salt).unwrap();
    assert_eq!(mk1.as_bytes(), mk2.as_bytes());
}

/// Changing any single input to the Dual-Lock must produce a different master key.
#[test]
fn test_dual_lock_sensitivity() {
    let password = b"base_password";
    let device_key = [0x42u8; 32];
    let salt = [0x00u8; 32];

    let base = keys::reconstruct_master_key(password, &device_key, &salt).unwrap();

    // Different password
    let diff_pwd = keys::reconstruct_master_key(b"other_password", &device_key, &salt).unwrap();
    assert_ne!(base.as_bytes(), diff_pwd.as_bytes());

    // Different device key
    let mut other_dk = [0x42u8; 32];
    other_dk[0] = 0x43;
    let diff_dk = keys::reconstruct_master_key(password, &other_dk, &salt).unwrap();
    assert_ne!(base.as_bytes(), diff_dk.as_bytes());

    // Different salt
    let mut other_salt = [0x00u8; 32];
    other_salt[0] = 0x01;
    let diff_salt = keys::reconstruct_master_key(password, &device_key, &other_salt).unwrap();
    assert_ne!(base.as_bytes(), diff_salt.as_bytes());
}

/// Blind index must be consistent when using the canonical EMAIL_BLIND_INDEX_SALT.
#[test]
fn test_blind_index_consistency_with_canonical_salt() {
    let email = "user@example.com";

    let idx1 = blind_index::compute_blind_index(email, blind_index::EMAIL_BLIND_INDEX_SALT).unwrap();
    let idx2 = blind_index::compute_blind_index(email, blind_index::EMAIL_BLIND_INDEX_SALT).unwrap();
    assert_eq!(idx1, idx2);

    // Case insensitive
    let idx3 = blind_index::compute_blind_index("User@Example.COM", blind_index::EMAIL_BLIND_INDEX_SALT).unwrap();
    assert_eq!(idx1, idx3);

    // Different emails produce different indices
    let idx4 = blind_index::compute_blind_index("other@example.com", blind_index::EMAIL_BLIND_INDEX_SALT).unwrap();
    assert_ne!(idx1, idx4);
}

/// Verification token roundtrip: encrypt a known token, decrypt, compare.
/// This mirrors the registration/unlock flow.
#[test]
fn test_verification_token_roundtrip() {
    let password = b"test_verification";
    let device_key = keys::generate_device_key();
    let salt = argon2_kdf::generate_salt();

    let master_key = keys::reconstruct_master_key(password, &device_key, &salt).unwrap();

    // Encrypt verification token (same pattern as register command)
    let token = b"SALADVAULT_VERIFIED";
    let (nonce, ciphertext) = xchacha::encrypt(master_key.as_bytes(), token).unwrap();

    // Simulate unlock: reconstruct key with same inputs, decrypt, verify
    let mk2 = keys::reconstruct_master_key(password, &device_key, &salt).unwrap();
    let decrypted = xchacha::decrypt(mk2.as_bytes(), &nonce, &ciphertext).unwrap();
    assert_eq!(decrypted, token);
}

/// Device key file must have exactly 32 bytes; corrupted files should error.
#[test]
fn test_device_key_invalid_size_rejected() {
    let tmp = TempDir::new().unwrap();
    let key_path = tmp.path().join("bad.key");

    // Write a key that is too short
    std::fs::write(&key_path, &[0u8; 16]).unwrap();
    assert!(keys::load_device_key(&key_path).is_err());

    // Write a key that is too long
    std::fs::write(&key_path, &[0u8; 64]).unwrap();
    assert!(keys::load_device_key(&key_path).is_err());
}

/// Missing device key file should return KeyFileNotFound.
#[test]
fn test_device_key_missing_file() {
    let tmp = TempDir::new().unwrap();
    let key_path = tmp.path().join("nonexistent.key");
    let err = keys::load_device_key(&key_path).unwrap_err();
    assert!(format!("{err:?}").contains("KeyFileNotFound"));
}
