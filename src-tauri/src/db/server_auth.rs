use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::crypto::xchacha;
use crate::error::AppError;

/// Plaintext token data (before encryption / after decryption).
#[derive(Serialize, Deserialize)]
pub struct ServerAuthData {
    pub api_url: String,
    pub access_token: String,
    pub refresh_token: String,
}

/// Save server auth data encrypted with the user's master key.
/// Replaces any existing entry for this user.
pub fn save(
    conn: &Connection,
    user_id: &str,
    master_key: &[u8; 32],
    data: &ServerAuthData,
) -> Result<(), AppError> {
    let json = serde_json::to_vec(data)
        .map_err(|e| AppError::Internal(format!("JSON serialize error: {e}")))?;

    let (nonce, ciphertext) = xchacha::encrypt(master_key, &json)?;

    conn.execute(
        "INSERT INTO server_auth (user_id, api_url, tokens_enc, tokens_nonce, saved_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))
         ON CONFLICT(user_id) DO UPDATE SET
             api_url = excluded.api_url,
             tokens_enc = excluded.tokens_enc,
             tokens_nonce = excluded.tokens_nonce,
             saved_at = excluded.saved_at",
        params![user_id, data.api_url, ciphertext, nonce],
    )?;

    Ok(())
}

/// Load and decrypt server auth data for the given user.
/// Returns None if no saved data exists.
pub fn load(
    conn: &Connection,
    user_id: &str,
    master_key: &[u8; 32],
) -> Result<Option<ServerAuthData>, AppError> {
    let result = conn.query_row(
        "SELECT tokens_enc, tokens_nonce FROM server_auth WHERE user_id = ?1",
        params![user_id],
        |row| {
            let tokens_enc: Vec<u8> = row.get(0)?;
            let tokens_nonce: Vec<u8> = row.get(1)?;
            Ok((tokens_enc, tokens_nonce))
        },
    );

    let (tokens_enc, tokens_nonce) = match result {
        Ok(data) => data,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(AppError::Database(e)),
    };

    let json = xchacha::decrypt(master_key, &tokens_nonce, &tokens_enc)?;

    let data: ServerAuthData = serde_json::from_slice(&json)
        .map_err(|e| AppError::Internal(format!("JSON deserialize error: {e}")))?;

    Ok(Some(data))
}

/// Delete saved server auth data for the given user.
pub fn delete(conn: &Connection, user_id: &str) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM server_auth WHERE user_id = ?1",
        params![user_id],
    )?;
    Ok(())
}
