#![no_main]
use libfuzzer_sys::fuzz_target;
use rust_app_lib::crypto::xchacha;

/// Fuzz decrypt with arbitrary bytes: the function must never panic.
/// It should return Err on invalid/tampered input, never crash.
fuzz_target!(|data: &[u8]| {
    // Need at least 32 (key) + 24 (nonce) + 1 (ciphertext) bytes
    if data.len() < 57 {
        return;
    }
    let key: [u8; 32] = data[..32].try_into().unwrap();
    let nonce = &data[32..56];
    let ciphertext = &data[56..];

    // Must not panic — Err is fine, panic is a bug
    let _ = xchacha::decrypt(&key, nonce, ciphertext);
});
