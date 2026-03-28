pub mod bridge;
pub mod feuilles;
pub mod saladiers;
pub mod schema;
pub mod server_auth;
pub mod settings;
pub mod users;

use rusqlite::Connection;
use std::path::Path;

use crate::error::AppError;

/// Open (or create) the application database at the given path.
///
/// Note: The database itself is not encrypted at the file level.
/// All sensitive data (names, passwords, entries) is encrypted at the application level
/// using XChaCha20-Poly1305 before being stored in the database.
/// This provides Zero-Knowledge security: even if the DB file is stolen,
/// all data_blob, name_enc fields are encrypted and unreadable.
pub fn open_database(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;

    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL")?;

    // Enable foreign keys
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // Initialize the schema
    schema::initialize(&conn)?;

    Ok(conn)
}

#[cfg(test)]
pub fn open_test_database() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    schema::initialize(&conn)?;
    Ok(conn)
}
