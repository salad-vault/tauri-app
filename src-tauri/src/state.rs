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
    /// The sync key bytes — device-independent, derived from password + salt_sync only.
    /// None if the user has no salt_sync yet (pre-migration).
    pub sync_key_bytes: Option<[u8; 32]>,
}

impl Session {
    pub fn zeroize(&mut self) {
        self.master_key_bytes.zeroize();
        if let Some(ref mut sk) = self.sync_key_bytes {
            sk.zeroize();
        }
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
    /// Active pairing code for the browser extension, with its creation time so
    /// it can be expired after a real TTL (SV-H1). `None` when no code is active.
    pub bridge_pairing_code: Mutex<Option<(String, Instant)>>,
    /// Persistent bridge token for authenticated extension connections
    pub bridge_token: Mutex<Option<String>>,
    /// Server-enforced auto-lock timeout in seconds (0 = disabled). Mirrors the
    /// user's setting; a background task locks the session when idle beyond it,
    /// so a frozen/dead webview cannot leave the vault unlocked (SV-M6).
    pub auto_lock_secs: Mutex<u64>,
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
            bridge_pairing_code: Mutex::new(None),
            bridge_token: Mutex::new(None),
            // Default to 5 minutes (matches AutoLockTimeout::After5Min) until the
            // user's real setting is loaded.
            auto_lock_secs: Mutex::new(300),
        }
    }

    /// Update the server-enforced auto-lock timeout (seconds; 0 disables it).
    pub fn set_auto_lock_secs(&self, secs: u64) {
        if let Ok(mut g) = self.auto_lock_secs.lock() {
            *g = secs;
        }
    }

    /// Zeroize and drop the current session and clear the Saladier key cache.
    /// Shared by the `lock` command and the auto-lock task (SV-M6).
    pub fn lock_now(&self) {
        self.clear_saladier_keys();
        if let Ok(mut session) = self.session.lock() {
            if let Some(ref mut s) = *session {
                s.zeroize();
            }
            *session = None;
        }
    }

    /// SV-M6: if a session is open and inactivity exceeds the configured
    /// timeout, lock it. Returns true if it locked. Enforced server-side so a
    /// frozen or dead webview cannot leave the vault unlocked indefinitely.
    pub fn auto_lock_if_idle(&self) -> bool {
        let timeout = match self.auto_lock_secs.lock() {
            Ok(g) => *g,
            Err(_) => return false,
        };
        if timeout == 0 {
            return false; // disabled (Never / inactivity off)
        }
        let session_open = self.session.lock().map(|g| g.is_some()).unwrap_or(false);
        if !session_open {
            return false;
        }
        let idle = match self.last_activity.lock() {
            Ok(g) => g.elapsed().as_secs(),
            Err(_) => return false,
        };
        if idle >= timeout {
            self.lock_now();
            true
        } else {
            false
        }
    }

    /// Path to the device_secret.key file.
    pub fn device_key_path(&self) -> PathBuf {
        self.data_dir.join("device_secret.key")
    }

    /// Path to the SQLite database file.
    #[allow(dead_code)]
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

    /// Get the sync key from the session. Returns an error if not available.
    pub fn require_sync_key(&self) -> Result<[u8; 32], crate::error::AppError> {
        let session = self
            .session
            .lock()
            .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
        match session.as_ref().and_then(|s| s.sync_key_bytes) {
            Some(sk) => Ok(sk),
            None => Err(crate::error::AppError::Internal(
                "Sync key not available".to_string(),
            )),
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
