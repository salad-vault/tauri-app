#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use rust_app_lib::crypto::blind_index;

#[derive(Arbitrary, Debug)]
struct BlindIndexInput {
    email: String,
    salt: Vec<u8>,
    device_key: [u8; 32],
}

/// Fuzz blind index computation: must never panic.
/// Verifies:
/// - Determinism (same input → same output)
/// - Case insensitivity (email normalized to lowercase)
/// - Server vs local indexes differ (different pepper sources)
fuzz_target!(|input: BlindIndexInput| {
    // Server blind index: deterministic
    let server1 = blind_index::compute_blind_index(&input.email, &input.salt);
    let server2 = blind_index::compute_blind_index(&input.email, &input.salt);
    match (&server1, &server2) {
        (Ok(a), Ok(b)) => assert_eq!(a, b, "server blind index must be deterministic"),
        _ => {}
    }

    // Case insensitivity
    let upper = blind_index::compute_blind_index(&input.email.to_uppercase(), &input.salt);
    let lower = blind_index::compute_blind_index(&input.email.to_lowercase(), &input.salt);
    match (&upper, &lower) {
        (Ok(a), Ok(b)) => assert_eq!(a, b, "blind index must be case-insensitive"),
        _ => {}
    }

    // Local blind index: deterministic
    let local1 = blind_index::compute_local_blind_index(
        &input.email, &input.salt, &input.device_key,
    );
    let local2 = blind_index::compute_local_blind_index(
        &input.email, &input.salt, &input.device_key,
    );
    match (&local1, &local2) {
        (Ok(a), Ok(b)) => assert_eq!(a, b, "local blind index must be deterministic"),
        _ => {}
    }

    // Server and local must differ (different peppers)
    if let (Ok(s), Ok(l)) = (&server1, &local1) {
        assert_ne!(s, l, "server and local blind indexes must differ");
    }
});
