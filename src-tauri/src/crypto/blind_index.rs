use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::AppError;

type HmacSha256 = Hmac<Sha256>;

/// Static pepper compiled into the binary for HMAC-SHA256 blind indexing.
///
/// Threat model: this is a compile-time constant by design. The blind index
/// must be deterministic across all installations so the server can match
/// users without seeing their email. Knowing the pepper only allows offline
/// brute-force enumeration against email dictionaries — it does NOT reveal
/// stored data. Acceptable risk for a local-first desktop app.
const PEPPER: &[u8] = b"SaladVault_BlindIndex_Pepper_v1";

/// Domain-separation salt for email blind indexing.
/// Must be identical across all installations for index consistency.
/// This is NOT a secret — it prevents cross-domain collisions.
pub const EMAIL_BLIND_INDEX_SALT: &[u8] = b"SaladVault_Email_Salt_v1";

/// Compute a blind index for an email address.
/// Uses HMAC-SHA256(pepper, normalize(email) + static_salt) to produce a
/// deterministic, non-reversible identifier.
/// The server never sees the original email.
pub fn compute_blind_index(email: &str, static_salt: &[u8]) -> Result<String, AppError> {
    let normalized = email.trim().to_lowercase();

    let mut mac = HmacSha256::new_from_slice(PEPPER)
        .map_err(|e| AppError::Internal(format!("HMAC init error: {e}")))?;

    mac.update(normalized.as_bytes());
    mac.update(static_salt);

    let result = mac.finalize();
    let hash_bytes = result.into_bytes();

    // Encode as hex string for storage
    Ok(hex::encode(hash_bytes))
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
}
