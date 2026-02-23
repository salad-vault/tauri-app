use serde::{Deserialize, Serialize};

/// Represents a user in the database.
/// The `id` is a blind index (HMAC-SHA256 hash of the email) -- never the email in plaintext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Blind index: HMAC-SHA256(pepper, email + static_salt)
    pub id: String,
    /// Random salt used for Argon2id key derivation
    pub salt_master: Vec<u8>,
    /// Encrypted cloud key part (Dual-Lock: server-side secret, encrypted)
    pub k_cloud_enc: Vec<u8>,
    /// Whether the user has confirmed saving their recovery phrase
    pub recovery_confirmed: bool,
}
