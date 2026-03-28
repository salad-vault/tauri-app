use rusqlite::Connection;

use crate::error::AppError;

/// Initialize the database schema with the required tables.
/// Uses IF NOT EXISTS to be idempotent.
pub fn initialize(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "
        -- Users table: never stores personal data in plaintext.
        -- id is the blind index (HMAC-SHA256 hash of the email).
        CREATE TABLE IF NOT EXISTS users (
            id                  TEXT PRIMARY KEY,
            salt_master         BLOB NOT NULL,
            k_cloud_enc         BLOB NOT NULL,
            recovery_confirmed  INTEGER NOT NULL DEFAULT 0
        );

        -- Saladiers (Vaults): each has independent encryption.
        CREATE TABLE IF NOT EXISTS saladiers (
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

        -- Feuilles (Entries): encrypted JSON blobs inside a Saladier.
        CREATE TABLE IF NOT EXISTS feuilles (
            uuid            TEXT PRIMARY KEY,
            saladier_id     TEXT NOT NULL,
            data_blob       BLOB NOT NULL,
            nonce           BLOB NOT NULL,
            FOREIGN KEY (saladier_id) REFERENCES saladiers(uuid) ON DELETE CASCADE
        );

        -- Settings: one JSON blob per user for all preferences.
        CREATE TABLE IF NOT EXISTS settings (
            user_id TEXT PRIMARY KEY,
            data    TEXT NOT NULL DEFAULT '{}',
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        -- Bridge config: persistent key-value for browser extension pairing.
        CREATE TABLE IF NOT EXISTS bridge_config (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- Server auth: persisted encrypted tokens for auto-reconnect.
        -- tokens_enc is encrypted with the user's master key (XChaCha20-Poly1305).
        CREATE TABLE IF NOT EXISTS server_auth (
            user_id      TEXT PRIMARY KEY,
            api_url      TEXT NOT NULL,
            tokens_enc   BLOB NOT NULL,
            tokens_nonce BLOB NOT NULL,
            saved_at     TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        ",
    )?;

    // Migrations for existing DBs (silently ignored if columns already exist)
    let _ = conn.execute_batch(
        "ALTER TABLE saladiers ADD COLUMN verify_enc BLOB NOT NULL DEFAULT X'';",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE saladiers ADD COLUMN verify_nonce BLOB NOT NULL DEFAULT X'';",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE saladiers ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE saladiers ADD COLUMN failed_attempts INTEGER NOT NULL DEFAULT 0;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE users ADD COLUMN recovery_confirmed INTEGER NOT NULL DEFAULT 0;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE users ADD COLUMN salt_sync BLOB;",
    );

    Ok(())
}
