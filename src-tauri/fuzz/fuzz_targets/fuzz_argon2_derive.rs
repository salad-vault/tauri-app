#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use rust_app_lib::crypto::argon2_kdf;

#[derive(Arbitrary, Debug)]
struct ArgonInput {
    password: Vec<u8>,
    salt: Vec<u8>,
}

/// Fuzz Argon2id key derivation: must never panic.
/// Also verifies determinism: same input → same output.
fuzz_target!(|input: ArgonInput| {
    // Argon2id requires salt >= 8 bytes
    if input.salt.len() < 8 {
        return;
    }

    // Limit password size to avoid OOM with Argon2 (64MB per call)
    if input.password.len() > 128 {
        return;
    }

    let result1 = argon2_kdf::derive_key(&input.password, &input.salt);
    let result2 = argon2_kdf::derive_key(&input.password, &input.salt);

    match (result1, result2) {
        (Ok(k1), Ok(k2)) => assert_eq!(k1, k2, "Argon2id must be deterministic"),
        (Err(_), Err(_)) => {} // both fail consistently — ok
        _ => panic!("Argon2id inconsistent: one succeeded, one failed"),
    }
});
