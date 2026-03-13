use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::AppError;

type HmacSha256 = Hmac<Sha256>;

/// Compile-time seed for blind index PEPPER derivation.
/// Used directly for server-side blind indexing (cross-device deterministic)
/// and as HKDF input material for local blind indexing (device-specific).
const PEPPER_SEED: &[u8] = b"SaladVault_BlindIndex_Pepper_v1";

/// Domain-separation salt for email blind indexing.
/// Must be identical across all installations for index consistency.
/// This is NOT a secret — it prevents cross-domain collisions.
pub const EMAIL_BLIND_INDEX_SALT: &[u8] = b"SaladVault_Email_Salt_v1";

/// Derive a device-specific PEPPER for local blind indexing.
/// Uses HKDF(device_key, PEPPER_SEED) to produce a 32-byte pepper
/// that is unique per device, preventing offline enumeration without
/// physical access to the device_key file.
fn derive_local_pepper(device_key: &[u8; 32]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(device_key), PEPPER_SEED);
    let mut pepper = [0u8; 32];
    hk.expand(b"SaladVault_LocalBlindIndex_v1", &mut pepper)
        .expect("HKDF expand for local pepper should not fail with 32-byte output");
    pepper
}

/// Compute a blind index using the provided pepper.
fn compute_blind_index_with_pepper(
    email: &str,
    static_salt: &[u8],
    pepper: &[u8],
) -> Result<String, AppError> {
    let normalized = email.trim().to_lowercase();

    let mut mac = HmacSha256::new_from_slice(pepper)
        .map_err(|e| AppError::Internal(format!("HMAC init error: {e}")))?;

    mac.update(normalized.as_bytes());
    mac.update(static_salt);

    let result = mac.finalize();
    let hash_bytes = result.into_bytes();

    Ok(hex::encode(hash_bytes))
}

/// Compute a local blind index for an email address using a device-specific pepper.
/// The pepper is derived from the device_key via HKDF, making the index
/// non-computable without physical access to the device_key file.
pub fn compute_local_blind_index(
    email: &str,
    static_salt: &[u8],
    device_key: &[u8; 32],
) -> Result<String, AppError> {
    let pepper = derive_local_pepper(device_key);
    compute_blind_index_with_pepper(email, static_salt, &pepper)
}

/// Compute a server-side blind index for an email address.
/// Uses the compile-time PEPPER_SEED directly so the result is deterministic
/// across all installations (required for server-side user matching).
pub fn compute_blind_index(email: &str, static_salt: &[u8]) -> Result<String, AppError> {
    compute_blind_index_with_pepper(email, static_salt, PEPPER_SEED)
}

/// Simple hex encoding (no external dep needed)
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blind_index_deterministic() {
        let salt = b"test_salt";
        let idx1 = compute_blind_index("user@example.com", salt).unwrap();
        let idx2 = compute_blind_index("user@example.com", salt).unwrap();
        assert_eq!(idx1, idx2);
    }

    #[test]
    fn test_blind_index_case_insensitive() {
        let salt = b"test_salt";
        let idx1 = compute_blind_index("User@Example.COM", salt).unwrap();
        let idx2 = compute_blind_index("user@example.com", salt).unwrap();
        assert_eq!(idx1, idx2);
    }

    #[test]
    fn test_blind_index_different_emails() {
        let salt = b"test_salt";
        let idx1 = compute_blind_index("alice@example.com", salt).unwrap();
        let idx2 = compute_blind_index("bob@example.com", salt).unwrap();
        assert_ne!(idx1, idx2);
    }

    #[test]
    fn test_local_blind_index_deterministic() {
        let salt = b"test_salt";
        let device_key = [42u8; 32];
        let idx1 = compute_local_blind_index("user@example.com", salt, &device_key).unwrap();
        let idx2 = compute_local_blind_index("user@example.com", salt, &device_key).unwrap();
        assert_eq!(idx1, idx2);
    }

    #[test]
    fn test_local_blind_index_differs_from_server() {
        let salt = b"test_salt";
        let device_key = [42u8; 32];
        let local = compute_local_blind_index("user@example.com", salt, &device_key).unwrap();
        let server = compute_blind_index("user@example.com", salt).unwrap();
        assert_ne!(local, server, "Local and server blind indexes must differ");
    }

    #[test]
    fn test_local_blind_index_different_device_keys() {
        let salt = b"test_salt";
        let dk1 = [42u8; 32];
        let dk2 = [99u8; 32];
        let idx1 = compute_local_blind_index("user@example.com", salt, &dk1).unwrap();
        let idx2 = compute_local_blind_index("user@example.com", salt, &dk2).unwrap();
        assert_ne!(idx1, idx2, "Different device keys must produce different indexes");
    }
}
