use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::crypto::{argon2_kdf, xchacha};
use crate::error::AppError;
use crate::sync::export;

/// Inner payload of the recovery blob.
/// Contains the master key + the full vault data so the recipient
/// can import everything into a fresh SaladVault instance.
#[derive(Serialize, Deserialize)]
struct RecoveryPayload {
    /// The master key (hex-encoded) — allows decrypting all vault entries.
    master_key_hex: String,
    /// The full vault export (raw DB rows, individually encrypted with master key).
    vault: export::SyncPayload,
}

/// Generate a self-contained recovery kit blob.
///
/// The blob includes the master key + full vault data, encrypted with
/// a key derived from the recovery password (Argon2id + XChaCha20-Poly1305).
///
/// Format: base64( salt_32 || nonce_24 || ciphertext )
///
/// The recipient decrypts with:
///   1. recovery_key = Argon2id(recovery_password, salt)
///   2. plaintext_json = XChaCha20-Poly1305_decrypt(recovery_key, nonce, ciphertext)
///   3. plaintext_json contains { master_key_hex, vault: SyncPayload }
pub fn generate_recovery_blob(
    conn: &rusqlite::Connection,
    master_key: &[u8; 32],
    recovery_password: &str,
) -> Result<String, AppError> {
    let b64 = base64::engine::general_purpose::STANDARD;

    // 1. Export the raw vault data (DB rows as-is)
    let raw_json = export::export_vault_raw(conn)?;
    let vault: export::SyncPayload = serde_json::from_slice(&raw_json)
        .map_err(|e| AppError::Internal(format!("Deserialization error: {e}")))?;

    // 2. Build recovery payload with master key included
    let recovery_payload = RecoveryPayload {
        master_key_hex: hex::encode(master_key),
        vault,
    };
    let payload_json = serde_json::to_vec(&recovery_payload)
        .map_err(|e| AppError::Internal(format!("Serialization error: {e}")))?;

    // 3. Derive a recovery key from the password
    let salt = argon2_kdf::generate_salt();
    let recovery_key = argon2_kdf::derive_key(recovery_password.as_bytes(), &salt)?;

    // 4. Encrypt the payload with the recovery key
    let (nonce, ciphertext) = xchacha::encrypt(&recovery_key, &payload_json)?;

    // 5. Pack: salt (32) || nonce (24) || ciphertext
    let mut packed = Vec::with_capacity(32 + nonce.len() + ciphertext.len());
    packed.extend_from_slice(&salt);
    packed.extend_from_slice(&nonce);
    packed.extend_from_slice(&ciphertext);

    Ok(b64.encode(&packed))
}
