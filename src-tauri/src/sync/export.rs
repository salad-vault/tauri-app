use base64::Engine;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::crypto::xchacha;
use crate::error::AppError;

/// Serializable representation of the entire local database.
/// All sensitive fields are already encrypted blobs — this struct
/// captures the raw DB rows as-is.
#[derive(Serialize, Deserialize)]
pub struct SyncPayload {
    pub users: Vec<UserRow>,
    pub saladiers: Vec<SaladierRow>,
    pub feuilles: Vec<FeuilleRow>,
    pub settings: Vec<SettingsRow>,
}

#[derive(Serialize, Deserialize)]
pub struct UserRow {
    pub id: String,
    pub salt_master: String,   // base64
    pub k_cloud_enc: String,   // base64
    pub recovery_confirmed: i32,
    #[serde(default)]
    pub salt_sync: Option<String>, // base64, for multi-device sync
}

#[derive(Serialize, Deserialize)]
pub struct SaladierRow {
    pub uuid: String,
    pub user_id: String,
    pub name_enc: String,      // base64
    pub salt_saladier: String, // base64
    pub nonce: String,         // base64
    pub verify_enc: String,    // base64
    pub verify_nonce: String,  // base64
    pub hidden: i32,
    pub failed_attempts: i32,
}

#[derive(Serialize, Deserialize)]
pub struct FeuilleRow {
    pub uuid: String,
    pub saladier_id: String,
    pub data_blob: String,     // base64
    pub nonce: String,         // base64
}

#[derive(Serialize, Deserialize)]
pub struct SettingsRow {
    pub user_id: String,
    pub data: String,          // JSON string
}

/// Collect all local DB rows into a SyncPayload.
fn collect_payload(conn: &Connection) -> Result<SyncPayload, AppError> {
    let b64 = base64::engine::general_purpose::STANDARD;

    // Users
    let mut stmt = conn.prepare("SELECT id, salt_master, k_cloud_enc, recovery_confirmed, salt_sync FROM users")?;
    let users: Vec<UserRow> = stmt
        .query_map([], |row| {
            let salt: Vec<u8> = row.get(1)?;
            let k_cloud: Vec<u8> = row.get(2)?;
            let salt_sync: Option<Vec<u8>> = row.get(4)?;
            Ok(UserRow {
                id: row.get(0)?,
                salt_master: b64.encode(&salt),
                k_cloud_enc: b64.encode(&k_cloud),
                recovery_confirmed: row.get(3)?,
                salt_sync: salt_sync.map(|s| b64.encode(&s)),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Saladiers
    let mut stmt = conn.prepare(
        "SELECT uuid, user_id, name_enc, salt_saladier, nonce, verify_enc, verify_nonce, hidden, failed_attempts FROM saladiers",
    )?;
    let saladiers: Vec<SaladierRow> = stmt
        .query_map([], |row| {
            let name_enc: Vec<u8> = row.get(2)?;
            let salt: Vec<u8> = row.get(3)?;
            let nonce: Vec<u8> = row.get(4)?;
            let verify_enc: Vec<u8> = row.get(5)?;
            let verify_nonce: Vec<u8> = row.get(6)?;
            Ok(SaladierRow {
                uuid: row.get(0)?,
                user_id: row.get(1)?,
                name_enc: b64.encode(&name_enc),
                salt_saladier: b64.encode(&salt),
                nonce: b64.encode(&nonce),
                verify_enc: b64.encode(&verify_enc),
                verify_nonce: b64.encode(&verify_nonce),
                hidden: row.get(7)?,
                failed_attempts: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Feuilles
    let mut stmt = conn.prepare("SELECT uuid, saladier_id, data_blob, nonce FROM feuilles")?;
    let feuilles: Vec<FeuilleRow> = stmt
        .query_map([], |row| {
            let data_blob: Vec<u8> = row.get(2)?;
            let nonce: Vec<u8> = row.get(3)?;
            Ok(FeuilleRow {
                uuid: row.get(0)?,
                saladier_id: row.get(1)?,
                data_blob: b64.encode(&data_blob),
                nonce: b64.encode(&nonce),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Settings
    let mut stmt = conn.prepare("SELECT user_id, data FROM settings")?;
    let settings: Vec<SettingsRow> = stmt
        .query_map([], |row| {
            Ok(SettingsRow {
                user_id: row.get(0)?,
                data: row.get(1)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(SyncPayload {
        users,
        saladiers,
        feuilles,
        settings,
    })
}

fn payload_to_json(payload: &SyncPayload) -> Result<Vec<u8>, AppError> {
    serde_json::to_vec(payload)
        .map_err(|e| AppError::Internal(format!("Serialization error: {e}")))
}

/// Export the entire local database as a JSON payload,
/// then encrypt it with the master key.
/// Returns a base64-encoded encrypted blob.
pub fn export_vault(conn: &Connection, master_key: &[u8; 32]) -> Result<String, AppError> {
    let b64 = base64::engine::general_purpose::STANDARD;
    let payload = collect_payload(conn)?;
    let json_bytes = payload_to_json(&payload)?;

    // Encrypt with master key
    let (nonce, ciphertext) = xchacha::encrypt(master_key, &json_bytes)?;

    // Pack: nonce (24 bytes) || ciphertext
    let mut packed = Vec::with_capacity(nonce.len() + ciphertext.len());
    packed.extend_from_slice(&nonce);
    packed.extend_from_slice(&ciphertext);

    Ok(b64.encode(&packed))
}

/// Export the entire local database as raw JSON bytes (not encrypted).
/// Used by the recovery kit generator which applies its own encryption.
pub fn export_vault_raw(conn: &Connection) -> Result<Vec<u8>, AppError> {
    let payload = collect_payload(conn)?;
    payload_to_json(&payload)
}
