use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::saladier::Saladier;

/// Helper to build a Saladier from a row.
fn saladier_from_row(row: &rusqlite::Row) -> rusqlite::Result<Saladier> {
    let hidden_int: i32 = row.get(7)?;
    let failed_attempts: i32 = row.get(8)?;
    Ok(Saladier {
        uuid: row.get(0)?,
        user_id: row.get(1)?,
        name_enc: row.get(2)?,
        salt_saladier: row.get(3)?,
        nonce: row.get(4)?,
        verify_enc: row.get(5)?,
        verify_nonce: row.get(6)?,
        hidden: hidden_int != 0,
        failed_attempts: failed_attempts as u32,
    })
}

const SELECT_COLS: &str = "uuid, user_id, name_enc, salt_saladier, nonce, verify_enc, verify_nonce, hidden, failed_attempts";

/// Insert a new Saladier into the database.
pub fn create_saladier(conn: &Connection, saladier: &Saladier) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO saladiers (uuid, user_id, name_enc, salt_saladier, nonce, verify_enc, verify_nonce, hidden, failed_attempts) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            saladier.uuid,
            saladier.user_id,
            saladier.name_enc,
            saladier.salt_saladier,
            saladier.nonce,
            saladier.verify_enc,
            saladier.verify_nonce,
            saladier.hidden as i32,
            saladier.failed_attempts as i32,
        ],
    )?;
    Ok(())
}

/// Get a Saladier by UUID.
pub fn get_saladier(conn: &Connection, uuid: &str) -> Result<Saladier, AppError> {
    let sql = format!("SELECT {} FROM saladiers WHERE uuid = ?1", SELECT_COLS);
    conn.query_row(&sql, params![uuid], saladier_from_row)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::SaladierNotFound,
            other => AppError::Database(other),
        })
}

/// List all visible (non-hidden) Saladiers for a given user.
pub fn list_saladiers(conn: &Connection, user_id: &str) -> Result<Vec<Saladier>, AppError> {
    let sql = format!(
        "SELECT {} FROM saladiers WHERE user_id = ?1 AND hidden = 0",
        SELECT_COLS
    );
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params![user_id], saladier_from_row)?;

    let mut saladiers = Vec::new();
    for row in rows {
        saladiers.push(row?);
    }

    Ok(saladiers)
}

/// List all hidden Saladiers for a given user (for hidden search).
pub fn list_hidden_saladiers(conn: &Connection, user_id: &str) -> Result<Vec<Saladier>, AppError> {
    let sql = format!(
        "SELECT {} FROM saladiers WHERE user_id = ?1 AND hidden = 1",
        SELECT_COLS
    );
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params![user_id], saladier_from_row)?;

    let mut saladiers = Vec::new();
    for row in rows {
        saladiers.push(row?);
    }

    Ok(saladiers)
}

/// List ALL Saladiers for a given user (visible + hidden).
pub fn list_all_saladiers(conn: &Connection, user_id: &str) -> Result<Vec<Saladier>, AppError> {
    let sql = format!(
        "SELECT {} FROM saladiers WHERE user_id = ?1",
        SELECT_COLS
    );
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params![user_id], saladier_from_row)?;

    let mut saladiers = Vec::new();
    for row in rows {
        saladiers.push(row?);
    }

    Ok(saladiers)
}

/// Update the encrypted name and nonce of a Saladier (for re-encryption).
pub fn update_saladier_name_enc(
    conn: &Connection,
    uuid: &str,
    name_enc: &[u8],
    nonce: &[u8],
) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE saladiers SET name_enc = ?1, nonce = ?2 WHERE uuid = ?3",
        params![name_enc, nonce, uuid],
    )?;
    if affected == 0 {
        return Err(AppError::SaladierNotFound);
    }
    Ok(())
}

/// Increment the failed_attempts counter for a Saladier.
/// Returns the new count.
pub fn increment_failed_attempts(conn: &Connection, uuid: &str) -> Result<u32, AppError> {
    conn.execute(
        "UPDATE saladiers SET failed_attempts = failed_attempts + 1 WHERE uuid = ?1",
        params![uuid],
    )?;
    let count: i32 = conn.query_row(
        "SELECT failed_attempts FROM saladiers WHERE uuid = ?1",
        params![uuid],
        |row| row.get(0),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::SaladierNotFound,
        other => AppError::Database(other),
    })?;
    Ok(count as u32)
}

/// Reset the failed_attempts counter for a Saladier.
pub fn reset_failed_attempts(conn: &Connection, uuid: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE saladiers SET failed_attempts = 0 WHERE uuid = ?1",
        params![uuid],
    )?;
    Ok(())
}

/// Delete a Saladier by UUID.
pub fn delete_saladier(conn: &Connection, uuid: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM saladiers WHERE uuid = ?1", params![uuid])?;
    if affected == 0 {
        return Err(AppError::SaladierNotFound);
    }
    Ok(())
}
