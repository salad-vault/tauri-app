use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use rusqlite::Connection;
use zeroize::Zeroize;

/// The current authenticated session.
pub struct Session {
    /// The blind index user ID
    pub user_id: String,
    /// The reconstructed master key bytes (zeroized on drop)
    pub master_key_bytes: [u8; 32],
}

impl Session {
    pub fn zeroize(&mut self) {
        self.master_key_bytes.zeroize();
        self.user_id.clear();
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Server authentication tokens.
pub struct ServerTokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// Global application state managed by Tauri.
pub struct AppState {
    /// Database connection (Mutex for thread-safe access)
    pub db: Mutex<Connection>,
    /// Current authenticated session (None if locked)
    pub session: Mutex<Option<Session>>,
    /// Cache of opened Saladier keys: saladier_uuid -> K_S (32 bytes)
    saladier_keys: Mutex<HashMap<String, [u8; 32]>>,
    /// Base directory for application data
    pub data_dir: PathBuf,
    /// Last activity timestamp for auto-lock
    pub last_activity: Mutex<Instant>,
    /// Server JWT tokens (None if not connected)
    pub server_tokens: Mutex<Option<ServerTokens>>,
    /// API server base URL
    pub api_base_url: Mutex<String>,
}

impl AppState {
    pub fn new(db: Connection, data_dir: PathBuf) -> Self {
        Self {
            db: Mutex::new(db),
            session: Mutex::new(None),
            saladier_keys: Mutex::new(HashMap::new()),
            data_dir,
            last_activity: Mutex::new(Instant::now()),
            server_tokens: Mutex::new(None),
            api_base_url: Mutex::new(String::new()),
        }
    }

    /// Path to the device_secret.key file.
    pub fn device_key_path(&self) -> PathBuf {
        self.data_dir.join("device_secret.key")
    }

    /// Path to the SQLite database file.
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("saladvault.db")
    }

    /// Get the current user ID and master key from the session, or return PotagerLocked.
    pub fn require_session(&self) -> Result<(String, [u8; 32]), crate::error::AppError> {
        let session = self
            .session
            .lock()
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
        match session.as_ref() {
            Some(s) => Ok((s.user_id.clone(), s.master_key_bytes)),
            None => Err(crate::error::AppError::PotagerLocked),
        }
    }

    /// Access the cache of opened Saladier keys.
    pub fn open_saladiers_cache(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<String, [u8; 32]>>, String> {
        self.saladier_keys
            .lock()
            .map_err(|e| e.to_string())
    }

    /// Clear all cached Saladier keys (zeroize each one).
    pub fn clear_saladier_keys(&self) {
        if let Ok(mut cache) = self.saladier_keys.lock() {
            for (_, key) in cache.iter_mut() {
                key.zeroize();
            }
            cache.clear();
        }
    }
}
