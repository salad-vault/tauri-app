use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::models::user::User;

/// Insert a new user into the database.
pub fn create_user(conn: &Connection, user: &User) -> Result<(), AppError> {
    let existing = get_user(conn, &user.id);
    if existing.is_ok() {
        return Err(AppError::UserAlreadyExists);
    }

    conn.execute(
        "INSERT INTO users (id, salt_master, k_cloud_enc, recovery_confirmed, salt_sync) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![user.id, user.salt_master, user.k_cloud_enc, user.recovery_confirmed as i32, user.salt_sync],
    )?;

    Ok(())
}

/// Get a user by their blind index ID.
pub fn get_user(conn: &Connection, user_id: &str) -> Result<User, AppError> {
    conn.query_row(
        "SELECT id, salt_master, k_cloud_enc, recovery_confirmed, salt_sync FROM users WHERE id = ?1",
        params![user_id],
        |row| {
            let confirmed: i32 = row.get(3)?;
            Ok(User {
                id: row.get(0)?,
                salt_master: row.get(1)?,
                k_cloud_enc: row.get(2)?,
                recovery_confirmed: confirmed != 0,
                salt_sync: row.get(4)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::UserNotFound,
        other => AppError::Database(other),
    })
}

/// Update the sync salt for a user (migration for existing users).
pub fn set_salt_sync(conn: &Connection, user_id: &str, salt_sync: &[u8]) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE users SET salt_sync = ?1 WHERE id = ?2",
        params![salt_sync, user_id],
    )?;
    if affected == 0 {
        return Err(AppError::UserNotFound);
    }
    Ok(())
}

/// Update the recovery_confirmed flag for a user.
pub fn set_recovery_confirmed(conn: &Connection, user_id: &str) -> Result<(), AppError> {
    let affected = conn.execute(
        "UPDATE users SET recovery_confirmed = 1 WHERE id = ?1",
        params![user_id],
    )?;
    if affected == 0 {
        return Err(AppError::UserNotFound);
    }
    Ok(())
}

/// Delete a user by their blind index ID.
#[allow(dead_code)]
pub fn delete_user(conn: &Connection, user_id: &str) -> Result<(), AppError> {
    let affected = conn.execute("DELETE FROM users WHERE id = ?1", params![user_id])?;
    if affected == 0 {
        return Err(AppError::UserNotFound);
    }
    Ok(())
}
