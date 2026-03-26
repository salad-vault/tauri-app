#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use rust_app_lib::crypto::keys;

#[derive(Arbitrary, Debug)]
struct MasterKeyInput {
    password: Vec<u8>,
    device_key: [u8; 32],
    salt: Vec<u8>,
}

/// Fuzz master key reconstruction (Dual-Lock protocol):
/// Argon2id(password, salt) → HKDF(device_key, derived) → 32-byte key.
///
/// Verifies:
/// - Never panics on arbitrary input
/// - Determinism: same inputs → same key
/// - Sensitivity: different device_key → different key
fuzz_target!(|input: MasterKeyInput| {
    // Argon2 requires salt >= 8 bytes; limit password to avoid OOM
    if input.salt.len() < 8 || input.password.len() > 128 {
        return;
    }

    let mk1 = keys::reconstruct_master_key(
        &input.password, &input.device_key, &input.salt,
    );
    let mk2 = keys::reconstruct_master_key(
        &input.password, &input.device_key, &input.salt,
    );

    match (&mk1, &mk2) {
        (Ok(k1), Ok(k2)) => {
            assert_eq!(
                k1.as_bytes(), k2.as_bytes(),
                "master key must be deterministic"
            );
        }
        (Err(_), Err(_)) => {} // both fail consistently
        _ => panic!("master key inconsistent: one succeeded, one failed"),
    }

    // If first succeeded, verify sensitivity to device_key
    if let Ok(k1) = &mk1 {
        let mut alt_dk = input.device_key;
        alt_dk[0] ^= 0xFF; // flip one byte

        if let Ok(k_alt) = keys::reconstruct_master_key(
            &input.password, &alt_dk, &input.salt,
        ) {
            assert_ne!(
                k1.as_bytes(), k_alt.as_bytes(),
                "different device_key must produce different master key"
            );
        }
    }
});
