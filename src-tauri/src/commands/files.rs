use crate::error::AppError;

/// Read a text file from disk. Used by the frontend after a file dialog pick.
#[tauri::command]
pub async fn read_text_file(path: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        std::fs::read_to_string(&path)
            .map_err(|e| AppError::Internal(format!("Cannot read file: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))?
}

/// Write a text file to disk. Used by the frontend after a save dialog.
#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> Result<(), AppError> {
    tokio::task::spawn_blocking(move || {
        std::fs::write(&path, content)
            .map_err(|e| AppError::Internal(format!("Cannot write file: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))?
}
