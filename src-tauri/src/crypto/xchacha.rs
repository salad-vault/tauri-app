use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, XChaCha20Poly1305, XNonce,
};

use crate::error::AppError;

/// Encrypt plaintext using XChaCha20-Poly1305.
/// Returns (nonce, ciphertext). The nonce is 24 bytes.
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), AppError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("Cipher init error: {e}")))?;

    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| AppError::DecryptionFailed)?;

    Ok((nonce.to_vec(), ciphertext))
}

/// Decrypt ciphertext using XChaCha20-Poly1305.
/// The nonce must be 24 bytes.
pub fn decrypt(key: &[u8; 32], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, AppError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("Cipher init error: {e}")))?;

    let nonce = XNonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::DecryptionFailed)?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"Hello, SaladVault!";

        let (nonce, ciphertext) = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key = [42u8; 32];
        let wrong_key = [99u8; 32];
        let plaintext = b"Secret data";

        let (nonce, ciphertext) = encrypt(&key, plaintext).unwrap();
        let result = decrypt(&wrong_key, &nonce, &ciphertext);

        assert!(result.is_err());
    }
}
