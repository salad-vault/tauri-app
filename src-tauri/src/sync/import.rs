use base64::Engine;
use rusqlite::Connection;

use crate::crypto::xchacha;
use crate::error::AppError;
use crate::sync::export::SyncPayload;

/// Import a vault blob received from the server.
/// The blob is base64(nonce || ciphertext) encrypted with the master key.
///
/// Strategy: replace the entire local database content with the server data.
/// This is the "accept server" strategy used when the user chooses to overwrite local.
pub fn import_vault(
    conn: &Connection,
    master_key: &[u8; 32],
    vault_blob_b64: &str,
) -> Result<(), AppError> {
    let b64 = base64::engine::general_purpose::STANDARD;

    // Decode base64
    let packed = b64
        .decode(vault_blob_b64)
        .map_err(|_| AppError::Internal("Invalid base64 vault blob".to_string()))?;

    if packed.len() < 24 {
        return Err(AppError::Internal("Vault blob too short".to_string()));
    }

    // Split nonce (24 bytes) and ciphertext
    let nonce = &packed[..24];
    let ciphertext = &packed[24..];

    // Decrypt
    let json_bytes = xchacha::decrypt(master_key, nonce, ciphertext)?;

    // Deserialize
    let payload: SyncPayload = serde_json::from_slice(&json_bytes)
        .map_err(|e| AppError::Internal(format!("Deserialization error: {e}")))?;

    // Replace local data in a transaction
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Clear existing data (order matters for foreign keys)
    tx.execute_batch(
        "DELETE FROM feuilles;
         DELETE FROM saladiers;
         DELETE FROM settings;
         DELETE FROM users;",
    )?;

    // Insert users
    for user in &payload.users {
        let salt = b64.decode(&user.salt_master).unwrap_or_default();
        let k_cloud = b64.decode(&user.k_cloud_enc).unwrap_or_default();
        let salt_sync: Option<Vec<u8>> = user.salt_sync.as_ref()
            .and_then(|s| b64.decode(s).ok());
        tx.execute(
            "INSERT INTO users (id, salt_master, k_cloud_enc, recovery_confirmed, salt_sync) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![user.id, salt, k_cloud, user.recovery_confirmed, salt_sync],
        )?;
    }

    // Insert saladiers
    for s in &payload.saladiers {
        let name_enc = b64.decode(&s.name_enc).unwrap_or_default();
        let salt = b64.decode(&s.salt_saladier).unwrap_or_default();
        let nonce = b64.decode(&s.nonce).unwrap_or_default();
        let verify_enc = b64.decode(&s.verify_enc).unwrap_or_default();
        let verify_nonce = b64.decode(&s.verify_nonce).unwrap_or_default();
        tx.execute(
            "INSERT INTO saladiers (uuid, user_id, name_enc, salt_saladier, nonce, verify_enc, verify_nonce, hidden, failed_attempts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                s.uuid, s.user_id, name_enc, salt, nonce, verify_enc, verify_nonce, s.hidden, s.failed_attempts
            ],
        )?;
    }

    // Insert feuilles
    for f in &payload.feuilles {
        let data_blob = b64.decode(&f.data_blob).unwrap_or_default();
        let nonce = b64.decode(&f.nonce).unwrap_or_default();
        tx.execute(
            "INSERT INTO feuilles (uuid, saladier_id, data_blob, nonce) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![f.uuid, f.saladier_id, data_blob, nonce],
        )?;
    }

    // Insert settings
    for s in &payload.settings {
        tx.execute(
            "INSERT INTO settings (user_id, data) VALUES (?1, ?2)
             ON CONFLICT(user_id) DO UPDATE SET data = excluded.data",
            rusqlite::params![s.user_id, s.data],
        )?;
    }

    tx.commit()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}
