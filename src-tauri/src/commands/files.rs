use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Maximum size (bytes) for a file read through this command. Imports are small
/// text files (JSON / CSV / XML); this caps abuse and accidental huge reads.
const MAX_READ_BYTES: u64 = 50 * 1024 * 1024; // 50 MB

/// Reject paths that point at SaladVault's own sensitive files, so this generic
/// file-IO command cannot be turned against the app — e.g. to exfiltrate the
/// device key (Ingredient Secret) or corrupt the local database (SV-H6).
fn ensure_path_allowed(state: &AppState, path: &str) -> Result<(), AppError> {
    let requested = std::path::Path::new(path);
    let protected = [state.device_key_path(), state.db_path()];
    let req_canon = std::fs::canonicalize(requested).ok();

    for p in protected.iter() {
        let blocked = match (req_canon.as_ref(), std::fs::canonicalize(p).ok()) {
            // Compare canonicalized paths when both resolve...
            (Some(a), Some(b)) => a == &b,
            // ...otherwise (e.g. the write target does not exist yet) fall back
            // to comparing canonicalized parent dir + file name, then literally.
            _ => requested == p.as_path(),
        };
        if blocked {
            return Err(AppError::Internal(
                "Access to this path is not allowed".to_string(),
            ));
        }
    }
    Ok(())
}

/// Read a text file from disk. Used by the frontend after a file dialog pick,
/// for vault import. Requires an unlocked Potager and forbids reading the app's
/// own sensitive files (SV-H6).
#[tauri::command]
pub async fn read_text_file(path: String, state: State<'_, AppState>) -> Result<String, AppError> {
    let _ = state.require_session()?;
    ensure_path_allowed(&state, &path)?;

    tokio::task::spawn_blocking(move || {
        let meta = std::fs::metadata(&path)
            .map_err(|e| AppError::Internal(format!("Cannot read file: {e}")))?;
        if meta.len() > MAX_READ_BYTES {
            return Err(AppError::Internal("File too large".to_string()));
        }
        std::fs::read_to_string(&path)
            .map_err(|e| AppError::Internal(format!("Cannot read file: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))?
}

/// Write a text file to disk. Used by the frontend after a save dialog, for
/// vault export. Requires an unlocked Potager and forbids overwriting the app's
/// own sensitive files (SV-H6).
#[tauri::command]
pub async fn write_text_file(
    path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let _ = state.require_session()?;
    ensure_path_allowed(&state, &path)?;

    tokio::task::spawn_blocking(move || {
        std::fs::write(&path, content)
            .map_err(|e| AppError::Internal(format!("Cannot write file: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))?
}
