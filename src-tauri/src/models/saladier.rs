use serde::{Deserialize, Serialize};

/// Represents a Saladier (vault/container) in the database.
/// Each Saladier has its own independent encryption key derived from its own password.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Saladier {
    /// Unique identifier (UUID v4)
    pub uuid: String,
    /// Owner user ID (blind index)
    pub user_id: String,
    /// Encrypted name of the Saladier
    pub name_enc: Vec<u8>,
    /// Salt for deriving the Saladier-specific key via Argon2id
    pub salt_saladier: Vec<u8>,
    /// Nonce used for encrypting the name
    pub nonce: Vec<u8>,
    /// Encrypted verification token (to verify Saladier password)
    pub verify_enc: Vec<u8>,
    /// Nonce for verify_enc
    pub verify_nonce: Vec<u8>,
    /// Whether this Saladier is hidden (invisible in the UI for plausible deniability)
    pub hidden: bool,
    /// Number of failed password attempts
    pub failed_attempts: u32,
}

/// Decrypted view of a Saladier, safe to send to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaladierInfo {
    pub uuid: String,
    pub name: String,
}
