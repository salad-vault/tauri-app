use tauri::State;

use crate::db;
use crate::error::AppError;
use crate::state::AppState;

/// Generate a short-lived, high-entropy pairing code for the browser extension
/// (SV-H1). The code expires after `bridge::PAIRING_TTL_SECS` real seconds.
#[tauri::command]
pub async fn generate_pairing_code(state: State<'_, AppState>) -> Result<String, AppError> {
    use rand::Rng;
    // 8 characters over an unambiguous 30-symbol alphabet (no 0/O/1/I/L/U)
    // ≈ 2^39 combinations — far beyond the previous 6-digit (10^6) space.
    // Generated with a CSPRNG; matched case-insensitively at pairing time.
    const ALPHABET: &[u8] = b"23456789ABCDEFGHJKMNPQRSTVWXYZ";
    let mut rng = rand::thread_rng();
    let code: String = (0..8)
        .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
        .collect();

    {
        let mut pc = state
            .bridge_pairing_code
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *pc = Some((code.clone(), std::time::Instant::now()));
    }

    Ok(code)
}

/// Check bridge status: is it paired? is a connection active?
#[tauri::command]
pub async fn get_bridge_status(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    let has_token = state.bridge_token.lock()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .is_some();
    Ok(serde_json::json!({
        "paired": has_token,
        "port": crate::bridge::BRIDGE_PORT,
    }))
}

/// Revoke the bridge token, disconnecting the extension.
#[tauri::command]
pub async fn revoke_bridge_token(state: State<'_, AppState>) -> Result<(), AppError> {
    {
        let mut t = state.bridge_token.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *t = None;
    }
    let conn = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    db::bridge::delete_bridge_token(&conn)?;
    Ok(())
}
