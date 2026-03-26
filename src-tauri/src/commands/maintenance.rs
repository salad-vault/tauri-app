use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Check if a newer version is available. Returns the new version string or null.
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<Option<String>, AppError> {
    use tauri_plugin_updater::UpdaterExt;
    let update = app
        .updater()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .check()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(update.map(|u| u.version))
}

/// Download and install the available update, then restart.
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), AppError> {
    use tauri_plugin_updater::UpdaterExt;
    let update = app
        .updater()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .check()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(update) = update {
        update
            .download_and_install(|_, _| {}, || {})
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}

/// Optimize the database storage by running VACUUM.
#[tauri::command]
pub async fn vacuum_database(state: State<'_, AppState>) -> Result<(), AppError> {
    let _ = state.require_session()?;

    let db_lock = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    db_lock.execute_batch("VACUUM")?;
    Ok(())
}

/// Check database integrity using PRAGMA integrity_check.
#[tauri::command]
pub async fn check_integrity(state: State<'_, AppState>) -> Result<String, AppError> {
    let _ = state.require_session()?;

    let db_lock = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let result: String = db_lock.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    Ok(result)
}
