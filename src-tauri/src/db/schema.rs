use rusqlite::{Connection, OptionalExtension};
use rusqlite_migration::{Migrations, M};

use crate::error::AppError;

// Versioned migrations. Append new ones at the end — never insert in the middle
// and never edit a past migration. `rusqlite_migration` tracks progress via
// PRAGMA user_version.
fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // M1: initial schema
        M::up(
            "CREATE TABLE IF NOT EXISTS users (
                id                  TEXT PRIMARY KEY,
                salt_master         BLOB NOT NULL,
                k_cloud_enc         BLOB NOT NULL
            );
            CREATE TABLE IF NOT EXISTS saladiers (
                uuid            TEXT PRIMARY KEY,
                user_id         TEXT NOT NULL,
                name_enc        BLOB NOT NULL,
                salt_saladier   BLOB NOT NULL,
                nonce           BLOB NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS feuilles (
                uuid            TEXT PRIMARY KEY,
                saladier_id     TEXT NOT NULL,
                data_blob       BLOB NOT NULL,
                nonce           BLOB NOT NULL,
                FOREIGN KEY (saladier_id) REFERENCES saladiers(uuid) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS settings (
                user_id TEXT PRIMARY KEY,
                data    TEXT NOT NULL DEFAULT '{}',
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS bridge_config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS server_auth (
                user_id      TEXT PRIMARY KEY,
                api_url      TEXT NOT NULL,
                tokens_enc   BLOB NOT NULL,
                tokens_nonce BLOB NOT NULL,
                saved_at     TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );",
        ),
        // M2: saladier verification ciphertext (used to confirm password before decrypting entries)
        M::up(
            "ALTER TABLE saladiers ADD COLUMN verify_enc BLOB NOT NULL DEFAULT X'';
             ALTER TABLE saladiers ADD COLUMN verify_nonce BLOB NOT NULL DEFAULT X'';",
        ),
        // M3: hidden saladiers + attempts counter for panic mode / brute-force protection
        M::up(
            "ALTER TABLE saladiers ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE saladiers ADD COLUMN failed_attempts INTEGER NOT NULL DEFAULT 0;",
        ),
        // M4: track recovery phrase confirmation (nag screen gate)
        M::up("ALTER TABLE users ADD COLUMN recovery_confirmed INTEGER NOT NULL DEFAULT 0;"),
        // M5: dedicated salt for cross-device sync key derivation
        M::up("ALTER TABLE users ADD COLUMN salt_sync BLOB;"),
    ])
}

/// Run all pending migrations on the given connection.
/// Idempotent: already-applied migrations are skipped via `user_version`.
pub fn initialize(conn: &mut Connection) -> Result<(), AppError> {
    bootstrap_legacy_version(conn)?;
    migrations()
        .to_latest(conn)
        .map_err(|e| AppError::Internal(format!("Migration error: {e}")))?;
    Ok(())
}

/// Pre-0.1.4 builds created tables with their final columns inline and applied
/// `ALTER TABLE` statements with silently-swallowed errors, but never stamped
/// `PRAGMA user_version`. Without this bootstrap, `to_latest` on such a DB
/// would re-run M2 and fail with "duplicate column name: verify_enc", panicking
/// the app on launch.
fn bootstrap_legacy_version(conn: &Connection) -> Result<(), AppError> {
    let current: i64 =
        conn.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
            row.get(0)
        })?;
    if current != 0 {
        return Ok(());
    }

    let has_users = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'users'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !has_users {
        return Ok(());
    }

    let has_column = |table: &str, column: &str| -> Result<bool, AppError> {
        Ok(conn
            .query_row(
                "SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2",
                rusqlite::params![table, column],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    };

    let legacy_version: i64 = if has_column("users", "salt_sync")? {
        5
    } else if has_column("users", "recovery_confirmed")? {
        4
    } else if has_column("saladiers", "failed_attempts")? {
        3
    } else if has_column("saladiers", "verify_enc")? {
        2
    } else {
        1
    };

    conn.pragma_update(None, "user_version", legacy_version)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid() {
        assert!(migrations().validate().is_ok());
    }

    #[test]
    fn migrations_apply_to_fresh_db() {
        let mut conn = Connection::open_in_memory().unwrap();
        initialize(&mut conn).unwrap();

        // All tables should exist after migration
        for table in [
            "users",
            "saladiers",
            "feuilles",
            "settings",
            "bridge_config",
            "server_auth",
        ] {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "table {table} missing");
        }

        // user_version should match the number of migrations
        let version: i64 = conn
            .query_row("SELECT user_version FROM pragma_user_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 5);
    }

    /// Reproduces the upgrade path from a pre-0.1.4 install: tables created
    /// with all final columns inline and `user_version` never stamped. Without
    /// `bootstrap_legacy_version`, the first `to_latest` call panics with
    /// "duplicate column name: verify_enc" when it replays M2.
    #[test]
    fn initialize_handles_legacy_db_without_user_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE users (
                id                  TEXT PRIMARY KEY,
                salt_master         BLOB NOT NULL,
                k_cloud_enc         BLOB NOT NULL,
                recovery_confirmed  INTEGER NOT NULL DEFAULT 0,
                salt_sync           BLOB
            );
            CREATE TABLE saladiers (
                uuid            TEXT PRIMARY KEY,
                user_id         TEXT NOT NULL,
                name_enc        BLOB NOT NULL,
                salt_saladier   BLOB NOT NULL,
                nonce           BLOB NOT NULL,
                verify_enc      BLOB NOT NULL DEFAULT X'',
                verify_nonce    BLOB NOT NULL DEFAULT X'',
                hidden          INTEGER NOT NULL DEFAULT 0,
                failed_attempts INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            CREATE TABLE feuilles (
                uuid        TEXT PRIMARY KEY,
                saladier_id TEXT NOT NULL,
                data_blob   BLOB NOT NULL,
                nonce       BLOB NOT NULL,
                FOREIGN KEY (saladier_id) REFERENCES saladiers(uuid) ON DELETE CASCADE
            );
            CREATE TABLE settings (
                user_id TEXT PRIMARY KEY,
                data    TEXT NOT NULL DEFAULT '{}',
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );
            CREATE TABLE bridge_config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE server_auth (
                user_id      TEXT PRIMARY KEY,
                api_url      TEXT NOT NULL,
                tokens_enc   BLOB NOT NULL,
                tokens_nonce BLOB NOT NULL,
                saved_at     TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );",
        )
        .unwrap();

        let pre_version: i64 = conn
            .query_row("SELECT user_version FROM pragma_user_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(pre_version, 0, "legacy DBs start at user_version = 0");

        initialize(&mut conn).expect("legacy DB must upgrade without errors");

        let post_version: i64 = conn
            .query_row("SELECT user_version FROM pragma_user_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(post_version, 5);
    }

    /// Older legacy DB that only reached the M2-equivalent schema before the
    /// user upgraded. `bootstrap_legacy_version` should stamp version 2, then
    /// `to_latest` applies M3–M5 on top.
    #[test]
    fn initialize_handles_partial_legacy_db() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE users (
                id          TEXT PRIMARY KEY,
                salt_master BLOB NOT NULL,
                k_cloud_enc BLOB NOT NULL
            );
            CREATE TABLE saladiers (
                uuid          TEXT PRIMARY KEY,
                user_id       TEXT NOT NULL,
                name_enc      BLOB NOT NULL,
                salt_saladier BLOB NOT NULL,
                nonce         BLOB NOT NULL,
                verify_enc    BLOB NOT NULL DEFAULT X'',
                verify_nonce  BLOB NOT NULL DEFAULT X''
            );
            CREATE TABLE feuilles (
                uuid        TEXT PRIMARY KEY,
                saladier_id TEXT NOT NULL,
                data_blob   BLOB NOT NULL,
                nonce       BLOB NOT NULL
            );
            CREATE TABLE settings (user_id TEXT PRIMARY KEY, data TEXT NOT NULL DEFAULT '{}');
            CREATE TABLE bridge_config (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE server_auth (
                user_id      TEXT PRIMARY KEY,
                api_url      TEXT NOT NULL,
                tokens_enc   BLOB NOT NULL,
                tokens_nonce BLOB NOT NULL,
                saved_at     TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();

        initialize(&mut conn).expect("partial legacy DB must upgrade cleanly");

        let post_version: i64 = conn
            .query_row("SELECT user_version FROM pragma_user_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(post_version, 5);

        // M3–M5 columns should have been applied on top of the stamped base.
        let has_failed_attempts: bool = conn
            .query_row(
                "SELECT 1 FROM pragma_table_info('saladiers') WHERE name = 'failed_attempts'",
                [],
                |_| Ok(()),
            )
            .optional()
            .unwrap()
            .is_some();
        assert!(has_failed_attempts, "M3 column must be applied after bootstrap");
    }
}
