use rusqlite::{params, Connection};

use crate::error::AppError;

pub fn get_bridge_token(conn: &Connection) -> Result<Option<String>, AppError> {
    let result = conn.query_row(
        "SELECT value FROM bridge_config WHERE key = 'bridge_token'",
        [],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn set_bridge_token(conn: &Connection, token: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO bridge_config (key, value) VALUES ('bridge_token', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![token],
    )?;
    Ok(())
}

pub fn delete_bridge_token(conn: &Connection) -> Result<(), AppError> {
    conn.execute("DELETE FROM bridge_config WHERE key = 'bridge_token'", [])?;
    Ok(())
}
