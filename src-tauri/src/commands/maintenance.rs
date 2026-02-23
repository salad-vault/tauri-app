use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

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
