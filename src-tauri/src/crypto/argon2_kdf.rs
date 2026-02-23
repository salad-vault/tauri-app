use argon2::{Algorithm, Argon2, Params, Version};

use crate::error::AppError;

/// OWASP recommended Argon2id parameters
const MEMORY_COST_KB: u32 = 65536; // 64 MB
const TIME_COST: u32 = 3;
const PARALLELISM: u32 = 4;
const OUTPUT_LEN: usize = 32;

/// Derive a 32-byte key from a password and salt using Argon2id
/// with OWASP recommended parameters (m=64MB, t=3, p=4).
pub fn derive_key(password: &[u8], salt: &[u8]) -> Result<[u8; 32], AppError> {
    let params = Params::new(MEMORY_COST_KB, TIME_COST, PARALLELISM, Some(OUTPUT_LEN))
        .map_err(|e| AppError::Internal(format!("Argon2 params error: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; OUTPUT_LEN];
    argon2
        .hash_password_into(password, salt, &mut output)
        .map_err(|e| AppError::Internal(format!("Argon2 hash error: {e}")))?;

    Ok(output)
}

/// Generate a random 32-byte salt
pub fn generate_salt() -> [u8; 32] {
    use rand::RngCore;
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let password = b"test_password";
        let salt = [0u8; 32];
        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_salt() {
        let password = b"test_password";
        let salt1 = [0u8; 32];
        let salt2 = [1u8; 32];
        let key1 = derive_key(password, &salt1).unwrap();
        let key2 = derive_key(password, &salt2).unwrap();
        assert_ne!(key1, key2);
    }
}
