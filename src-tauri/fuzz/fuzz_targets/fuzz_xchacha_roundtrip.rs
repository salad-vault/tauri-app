#![no_main]
use libfuzzer_sys::fuzz_target;
use rust_app_lib::crypto::xchacha;

/// Fuzz encrypt→decrypt roundtrip: any plaintext with a random key
/// must decrypt back to the original plaintext.
fuzz_target!(|data: &[u8]| {
    // Use first 32 bytes as key, rest as plaintext
    if data.len() < 32 {
        return;
    }
    let key: [u8; 32] = data[..32].try_into().unwrap();
    let plaintext = &data[32..];

    let (nonce, ciphertext) = match xchacha::encrypt(&key, plaintext) {
        Ok(v) => v,
        Err(_) => return,
    };

    let decrypted = xchacha::decrypt(&key, &nonce, &ciphertext)
        .expect("decrypt must succeed on freshly encrypted data");

    assert_eq!(
        plaintext, &decrypted[..],
        "roundtrip mismatch: plaintext != decrypted"
    );
});
