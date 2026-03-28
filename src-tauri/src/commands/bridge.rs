use tauri::State;

use crate::db;
use crate::error::AppError;
use crate::state::AppState;

/// Generate a 6-digit pairing code for browser extension connection.
/// The code expires after 60 seconds.
#[tauri::command]
pub async fn generate_pairing_code(state: State<'_, AppState>) -> Result<String, AppError> {
    use rand::Rng;
    let code: String = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000));

    {
        let mut pc = state.bridge_pairing_code.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *pc = Some(code.clone());
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
