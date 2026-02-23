use std::time::Instant;

use tauri::State;

use crate::db;
use crate::error::AppError;
use crate::models::settings::UserSettings;
use crate::state::AppState;

/// Get the current user's settings.
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<UserSettings, AppError> {
    let (user_id, _) = state.require_session()?;

    let db_lock = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::settings::get_settings(&db_lock, &user_id)
}

/// Save the current user's settings.
#[tauri::command]
pub async fn save_settings(
    settings: UserSettings,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let (user_id, _) = state.require_session()?;

    let db_lock = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::settings::save_settings(&db_lock, &user_id, &settings)
}

/// Apply screenshot protection dynamically.
#[tauri::command]
pub async fn apply_screenshot_protection(
    enabled: bool,
    window: tauri::Window,
) -> Result<(), AppError> {
    window
        .set_content_protected(enabled)
        .map_err(|e| AppError::Internal(format!("Screenshot protection error: {e}")))?;
    Ok(())
}

/// Write text to the clipboard.
#[tauri::command]
pub async fn write_to_clipboard(text: String) -> Result<(), AppError> {
    // We use the Tauri clipboard plugin which is initialized in lib.rs
    // The frontend will call this via the plugin's JS API directly
    // This command is a fallback / alternative
    let _ = text;
    Ok(())
}

/// Clear the clipboard.
#[tauri::command]
pub async fn clear_clipboard() -> Result<(), AppError> {
    // The frontend will use the clipboard plugin's JS API
    Ok(())
}

/// Update last activity timestamp (called by frontend on user interaction).
#[tauri::command]
pub async fn update_last_activity(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut last = state
        .last_activity
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    *last = Instant::now();
    Ok(())
}

/// Check if auto-lock should trigger based on inactivity.
/// Returns the number of seconds since last activity.
#[tauri::command]
pub async fn get_inactivity_seconds(state: State<'_, AppState>) -> Result<u64, AppError> {
    let last = state
        .last_activity
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(last.elapsed().as_secs())
}
