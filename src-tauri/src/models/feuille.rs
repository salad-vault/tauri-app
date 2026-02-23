use serde::{Deserialize, Serialize};

/// Represents a Feuille (entry) in the database.
/// The data_blob is an encrypted JSON blob containing the actual credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feuille {
    /// Unique identifier (UUID v4)
    pub uuid: String,
    /// Parent Saladier UUID
    pub saladier_id: String,
    /// Encrypted JSON blob containing FeuilleData
    pub data_blob: Vec<u8>,
    /// Nonce used for XChaCha20-Poly1305 encryption
    pub nonce: Vec<u8>,
}

/// The plaintext content of a Feuille, serialized to JSON before encryption.
/// This structure is NEVER stored in plaintext -- always encrypted as data_blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeuilleData {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
}

/// Decrypted view of a Feuille, safe to send to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeuilleInfo {
    pub uuid: String,
    pub saladier_id: String,
    pub data: FeuilleData,
}
