use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::feuille::Feuille;

/// Insert a new Feuille (entry) into the database.
pub fn create_feuille(conn: &Connection, feuille: &Feuille) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO feuilles (uuid, saladier_id, data_blob, nonce) VALUES (?1, ?2, ?3, ?4)",
        params![
            feuille.uuid,
            feuille.saladier_id,
            feuille.data_blob,
            feuille.nonce,
        ],
    )?;
    Ok(())
}

/// Get a Feuille by UUID.
pub fn get_feuille(conn: &Connection, uuid: &str) -> Result<Feuille, AppError> {
    conn.query_row(
        "SELECT uuid, saladier_id, data_blob, nonce FROM feuilles WHERE uuid = ?1",
        params![uuid],
        |row| {
            Ok(Feuille {
                uuid: row.get(0)?,
                saladier_id: row.get(1)?,
                data_blob: row.get(2)?,
                nonce: row.get(3)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::FeuilleNotFound,
        other => AppError::Database(other),
    })
}

/// List all Feuilles in a given Saladier.
pub fn list_feuilles(conn: &Connection, saladier_id: &str) -> Result<Vec<Feuille>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT uuid, saladier_id, data_blob, nonce FROM feuilles WHERE saladier_id = ?1",
    )?;

    let rows = stmt.query_map(params![saladier_id], |row| {
        Ok(Feuille {
            uuid: row.get(0)?,
            saladier_id: row.get(1)?,
            data_blob: row.get(2)?,
            nonce: row.get(3)?,
        })
    })?;

    let mut feuilles = Vec::new();
    for row in rows {
        feuilles.push(row?);
    }

    Ok(feuilles)
}

/// Update a Feuille's encrypted data.
pub fn update_feuille(conn: &Connection, feuille: &Feuille) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE feuilles SET data_blob = ?1, nonce = ?2 WHERE uuid = ?3",
        params![feuille.data_blob, feuille.nonce, feuille.uuid],
    )?;
    if affected == 0 {
        return Err(AppError::FeuilleNotFound);
    }
    Ok(())
}

/// Delete a Feuille by UUID.
pub fn delete_feuille(conn: &Connection, uuid: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM feuilles WHERE uuid = ?1", params![uuid])?;
    if affected == 0 {
        return Err(AppError::FeuilleNotFound);
    }
    Ok(())
}
