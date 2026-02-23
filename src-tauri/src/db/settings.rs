use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::settings::UserSettings;

/// Get settings for a user. Returns default settings if none exist.
pub fn get_settings(conn: &Connection, user_id: &str) -> Result<UserSettings, AppError> {
    let result = conn.query_row(
        "SELECT data FROM settings WHERE user_id = ?1",
        params![user_id],
        |row| {
            let data: String = row.get(0)?;
            Ok(data)
        },
    );

    match result {
        Ok(data) => {
            let settings: UserSettings = serde_json::from_str(&data)
                .unwrap_or_default();
            Ok(settings)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(UserSettings::default()),
        Err(e) => Err(AppError::Database(e)),
    }
}

/// Save settings for a user (upsert).
pub fn save_settings(
    conn: &Connection,
    user_id: &str,
    settings: &UserSettings,
) -> Result<(), AppError> {
    let data = serde_json::to_string(settings)
        .map_err(|e| AppError::Internal(format!("Settings serialization error: {e}")))?;

    conn.execute(
        "INSERT INTO settings (user_id, data) VALUES (?1, ?2)
         ON CONFLICT(user_id) DO UPDATE SET data = excluded.data",
        params![user_id, data],
    )?;

    Ok(())
}
