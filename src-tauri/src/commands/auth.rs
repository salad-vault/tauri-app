use std::path::PathBuf;

use tauri::State;

use crate::crypto::{argon2_kdf, blind_index, keys, xchacha};
use crate::db;
use crate::error::AppError;
use crate::models::user::User;
use crate::state::AppState;

use crate::crypto::blind_index::EMAIL_BLIND_INDEX_SALT;

// ── Pre-change backup helpers ───────────────────────────────────────────────

/// Create a backup of the database before a destructive operation.
/// Uses `VACUUM INTO` for a clean, WAL-merged copy.
/// Returns the path to the backup file.
fn create_pre_change_backup(
    conn: &rusqlite::Connection,
    data_dir: &std::path::Path,
) -> Result<PathBuf, AppError> {
    let backups_dir = data_dir.join("backups");
    std::fs::create_dir_all(&backups_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_path = backups_dir.join(format!("saladvault_pre_pwchange_{timestamp}.db"));

    conn.execute("VACUUM INTO ?1", rusqlite::params![backup_path.to_string_lossy().as_ref()])?;

    Ok(backup_path)
}

/// Remove a backup file after a successful operation. Best-effort.
fn remove_backup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

/// Remove old backup files, keeping at most `keep` recent ones.
fn cleanup_old_backups(data_dir: &std::path::Path, keep: usize) {
    let backups_dir = data_dir.join("backups");
    if let Ok(entries) = std::fs::read_dir(&backups_dir) {
        let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        files.sort_by_key(|e| {
            std::cmp::Reverse(
                e.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            )
        });
        for old in files.into_iter().skip(keep) {
            let _ = std::fs::remove_file(old.path());
        }
    }
}

/// Register a new user account (Potager).
#[tauri::command]
pub async fn register(
    email: String,
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let user_id = blind_index::compute_blind_index(&email, EMAIL_BLIND_INDEX_SALT)?;

    // Check if user already exists BEFORE generating a new device key.
    // Otherwise, save_device_key would overwrite the existing key and
    // permanently break the unlock for the existing account.
    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if db::users::get_user(&db_lock, &user_id).is_ok() {
            return Err(AppError::UserAlreadyExists);
        }
    }

    let salt_master = argon2_kdf::generate_salt();
    let device_key = keys::generate_device_key();

    // Reconstruct master key in a blocking thread
    let pwd = master_password.into_bytes();
    let dk = device_key;
    let sm = salt_master;
    let master_key = tokio::task::spawn_blocking(move || {
        keys::reconstruct_master_key(&pwd, &dk, &sm)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    let verification_data = b"SALADVAULT_VERIFIED";
    let (nonce, ciphertext) = xchacha::encrypt(master_key.as_bytes(), verification_data)?;

    let mut k_cloud_enc = nonce;
    k_cloud_enc.extend_from_slice(&ciphertext);

    let user = User {
        id: user_id.clone(),
        salt_master: salt_master.to_vec(),
        k_cloud_enc,
        recovery_confirmed: false,
    };

    let device_key_path = state.device_key_path();
    keys::save_device_key(&device_key, &device_key_path)?;

    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::users::create_user(&db_lock, &user)?;
    }

    {
        let mut session = state.session.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        *session = Some(crate::state::Session {
            user_id,
            master_key_bytes: *master_key.as_bytes(),
        });
    }

    Ok(())
}

/// Unlock the Potager (authenticate with master password).
#[tauri::command]
pub async fn unlock(
    email: String,
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let user_id = blind_index::compute_blind_index(&email, EMAIL_BLIND_INDEX_SALT)?;

    let device_key_path = state.device_key_path();
    let device_key = keys::load_device_key(&device_key_path)?;

    // Scope the db_lock so it is dropped before any .await
    let user = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::users::get_user(&db_lock, &user_id)?
    };

    // Reconstruct master key in a blocking thread (Argon2id is CPU-intensive)
    let pwd = master_password.into_bytes();
    let dk = device_key;
    let salt = user.salt_master.clone();
    let master_key = tokio::task::spawn_blocking(move || {
        keys::reconstruct_master_key(&pwd, &dk, &salt)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    // Verify by decrypting k_cloud_enc
    if user.k_cloud_enc.len() < 24 {
        return Err(AppError::InvalidCredentials);
    }
    let (nonce, ciphertext) = user.k_cloud_enc.split_at(24);
    let decrypted = xchacha::decrypt(master_key.as_bytes(), nonce, ciphertext)
        .map_err(|_| AppError::InvalidCredentials)?;

    if decrypted != b"SALADVAULT_VERIFIED" {
        return Err(AppError::InvalidCredentials);
    }

    {
        let mut session = state.session.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        *session = Some(crate::state::Session {
            user_id: user_id.clone(),
            master_key_bytes: *master_key.as_bytes(),
        });
    }

    // Restore persisted server tokens (if any)
    {
        let conn = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if let Ok(Some(auth_data)) = db::server_auth::load(&conn, &user_id, master_key.as_bytes()) {
            let mut url = state.api_base_url.lock()
                .map_err(|e| AppError::Internal(e.to_string()))?;
            *url = auth_data.api_url;
            drop(url);

            let mut tokens = state.server_tokens.lock()
                .map_err(|e| AppError::Internal(e.to_string()))?;
            *tokens = Some(crate::state::ServerTokens {
                access_token: auth_data.access_token,
                refresh_token: auth_data.refresh_token,
            });
        }
    }

    Ok(())
}

/// Lock the Potager.
#[tauri::command]
pub async fn lock(state: State<'_, AppState>) -> Result<(), AppError> {
    state.clear_saladier_keys();

    let mut session = state.session.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(ref mut s) = *session {
        s.zeroize();
    }
    *session = None;
    Ok(())
}

/// Check if the Potager is currently unlocked.
#[tauri::command]
pub async fn is_unlocked(state: State<'_, AppState>) -> Result<bool, AppError> {
    let session = state.session.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(session.is_some())
}

/// Verify the master password matches the current session.
pub async fn verify_master_password_inner(
    master_password: &str,
    state: &AppState,
) -> Result<(), AppError> {
    let (user_id, _) = state.require_session()?;

    let device_key_path = state.device_key_path();
    let device_key = keys::load_device_key(&device_key_path)?;

    let user = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::users::get_user(&db_lock, &user_id)?
    };

    let pwd = master_password.as_bytes().to_vec();
    let dk = device_key;
    let salt = user.salt_master.clone();
    let master_key = tokio::task::spawn_blocking(move || {
        keys::reconstruct_master_key(&pwd, &dk, &salt)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    if user.k_cloud_enc.len() < 24 {
        return Err(AppError::InvalidCredentials);
    }
    let (nonce, ciphertext) = user.k_cloud_enc.split_at(24);
    xchacha::decrypt(master_key.as_bytes(), nonce, ciphertext)
        .map_err(|_| AppError::InvalidCredentials)?;

    Ok(())
}

/// Tauri command to verify the master password.
#[tauri::command]
pub async fn verify_master_password(
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    verify_master_password_inner(&master_password, &state).await
}

/// Change the master password:
/// 1. Verify old password
/// 2. Generate new salt_master
/// 3. Derive new master key with HKDF
/// 4. Re-encrypt k_cloud_enc
/// 5. Re-encrypt all Saladier names
/// 6. Backup DB, then update atomically inside a transaction
///
/// Note: The device key (Ingredient Secret) does not change when the password
/// changes. The BIP39 recovery phrase remains valid. `recovery_confirmed` is
/// intentionally left untouched.
#[tauri::command]
pub async fn change_master_password(
    current_password: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    // Verify the current password
    verify_master_password_inner(&current_password, &state).await?;

    let (user_id, old_master_key) = state.require_session()?;

    let device_key_path = state.device_key_path();
    let device_key = keys::load_device_key(&device_key_path)?;

    // Generate new salt
    let new_salt = argon2_kdf::generate_salt();

    // Derive new master key
    let pwd = new_password.into_bytes();
    let dk = device_key;
    let salt = new_salt;
    let new_master_key = tokio::task::spawn_blocking(move || {
        keys::reconstruct_master_key(&pwd, &dk, &salt)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    // Re-encrypt k_cloud_enc
    let verification_data = b"SALADVAULT_VERIFIED";
    let (nonce, ciphertext) = xchacha::encrypt(new_master_key.as_bytes(), verification_data)?;
    let mut new_k_cloud_enc = nonce;
    new_k_cloud_enc.extend_from_slice(&ciphertext);

    // Re-encrypt all Saladier names
    let saladiers = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::saladiers::list_all_saladiers(&db_lock, &user_id)?
    };

    let mut re_encrypted: Vec<(String, Vec<u8>, Vec<u8>)> = Vec::new();
    for s in &saladiers {
        let name_bytes = xchacha::decrypt(&old_master_key, &s.nonce, &s.name_enc)
            .unwrap_or_else(|_| b"[Error]".to_vec());
        let (new_nonce, new_name_enc) = xchacha::encrypt(new_master_key.as_bytes(), &name_bytes)?;
        re_encrypted.push((s.uuid.clone(), new_name_enc, new_nonce));
    }

    // Apply all changes atomically with a pre-change backup
    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        // Backup first (abort on failure — vault is still untouched)
        let backup_path = create_pre_change_backup(&db_lock, &state.data_dir)?;

        // Begin transaction (auto-ROLLBACK on drop if not committed)
        let tx = db_lock
            .unchecked_transaction()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let commit_result = (|| -> Result<(), AppError> {
            tx.execute(
                "UPDATE users SET salt_master = ?1, k_cloud_enc = ?2 WHERE id = ?3",
                rusqlite::params![new_salt.to_vec(), new_k_cloud_enc, user_id],
            )?;

            for (uuid, name_enc, nonce) in &re_encrypted {
                db::saladiers::update_saladier_name_enc(&tx, uuid, name_enc, nonce)?;
            }

            tx.commit()
                .map_err(|e| AppError::Internal(e.to_string()))?;

            Ok(())
        })();

        match commit_result {
            Ok(()) => {
                // Success: remove backup, clean up old ones
                remove_backup(&backup_path);
                cleanup_old_backups(&state.data_dir, 3);
            }
            Err(e) => {
                // Transaction rolled back automatically on drop.
                // Backup file is preserved for manual recovery.
                return Err(e);
            }
        }
    }

    // Update session
    {
        let mut session = state.session.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        *session = Some(crate::state::Session {
            user_id,
            master_key_bytes: *new_master_key.as_bytes(),
        });
    }

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::crypto::xchacha;
    use crate::db;
    use crate::models::saladier::Saladier;
    use crate::models::user::User;

    /// Verify that when a saladier update fails inside a transaction,
    /// the user row update is also rolled back (atomicity guarantee).
    #[test]
    fn test_change_password_transaction_rollback() {
        let conn = db::open_test_database().unwrap();

        let salt = [1u8; 32];
        let fake_master_key = [42u8; 32];

        // Create verification token
        let (nonce, ct) = xchacha::encrypt(&fake_master_key, b"SALADVAULT_VERIFIED").unwrap();
        let mut k_cloud_enc = nonce;
        k_cloud_enc.extend_from_slice(&ct);

        let user = User {
            id: "test_user".to_string(),
            salt_master: salt.to_vec(),
            k_cloud_enc: k_cloud_enc.clone(),
            recovery_confirmed: false,
        };
        db::users::create_user(&conn, &user).unwrap();

        // Create a real saladier
        let (s_nonce, s_name_enc) = xchacha::encrypt(&fake_master_key, b"My Vault").unwrap();
        let saladier = Saladier {
            uuid: "sal-1".to_string(),
            user_id: "test_user".to_string(),
            name_enc: s_name_enc.clone(),
            salt_saladier: vec![0u8; 32],
            nonce: s_nonce.clone(),
            verify_enc: vec![0u8; 16],
            verify_nonce: vec![0u8; 24],
            hidden: false,
            failed_attempts: 0,
        };
        db::saladiers::create_saladier(&conn, &saladier).unwrap();

        // Start a transaction: update user, then try to update a non-existent saladier
        let tx = conn.unchecked_transaction().unwrap();

        let new_salt = [99u8; 32];
        tx.execute(
            "UPDATE users SET salt_master = ?1 WHERE id = ?2",
            rusqlite::params![new_salt.to_vec(), "test_user"],
        )
        .unwrap();

        // This should fail: non-existent saladier UUID -> SaladierNotFound
        let result =
            db::saladiers::update_saladier_name_enc(&tx, "nonexistent-uuid", &[0], &[0]);
        assert!(result.is_err());

        // Drop tx without commit -> automatic ROLLBACK
        drop(tx);

        // Verify: user row must be unchanged (rollback worked)
        let user_after = db::users::get_user(&conn, "test_user").unwrap();
        assert_eq!(
            user_after.salt_master,
            salt.to_vec(),
            "User salt should be unchanged after rollback"
        );
        assert_eq!(
            user_after.k_cloud_enc, k_cloud_enc,
            "k_cloud_enc should be unchanged after rollback"
        );

        // Verify: saladier name_enc must also be unchanged
        let saladiers = db::saladiers::list_all_saladiers(&conn, "test_user").unwrap();
        assert_eq!(saladiers.len(), 1);
        assert_eq!(
            saladiers[0].name_enc, s_name_enc,
            "Saladier name_enc should be unchanged after rollback"
        );
    }

    /// Verify that a successful transaction commits all changes.
    #[test]
    fn test_change_password_transaction_commit() {
        let conn = db::open_test_database().unwrap();

        let salt = [1u8; 32];
        let fake_master_key = [42u8; 32];

        let (nonce, ct) = xchacha::encrypt(&fake_master_key, b"SALADVAULT_VERIFIED").unwrap();
        let mut k_cloud_enc = nonce;
        k_cloud_enc.extend_from_slice(&ct);

        let user = User {
            id: "test_user".to_string(),
            salt_master: salt.to_vec(),
            k_cloud_enc,
            recovery_confirmed: false,
        };
        db::users::create_user(&conn, &user).unwrap();

        let (s_nonce, s_name_enc) = xchacha::encrypt(&fake_master_key, b"My Vault").unwrap();
        let saladier = Saladier {
            uuid: "sal-1".to_string(),
            user_id: "test_user".to_string(),
            name_enc: s_name_enc,
            salt_saladier: vec![0u8; 32],
            nonce: s_nonce,
            verify_enc: vec![0u8; 16],
            verify_nonce: vec![0u8; 24],
            hidden: false,
            failed_attempts: 0,
        };
        db::saladiers::create_saladier(&conn, &saladier).unwrap();

        // Successful transaction: update user + update saladier
        let new_salt = [99u8; 32];
        let new_key = [77u8; 32];
        let (new_nonce, new_ct) = xchacha::encrypt(&new_key, b"SALADVAULT_VERIFIED").unwrap();
        let mut new_k_cloud_enc = new_nonce;
        new_k_cloud_enc.extend_from_slice(&new_ct);

        let (new_s_nonce, new_s_name_enc) = xchacha::encrypt(&new_key, b"My Vault").unwrap();

        let tx = conn.unchecked_transaction().unwrap();
        tx.execute(
            "UPDATE users SET salt_master = ?1, k_cloud_enc = ?2 WHERE id = ?3",
            rusqlite::params![new_salt.to_vec(), new_k_cloud_enc, "test_user"],
        )
        .unwrap();
        db::saladiers::update_saladier_name_enc(&tx, "sal-1", &new_s_name_enc, &new_s_nonce)
            .unwrap();
        tx.commit().unwrap();

        // Verify: all changes persisted
        let user_after = db::users::get_user(&conn, "test_user").unwrap();
        assert_eq!(
            user_after.salt_master,
            new_salt.to_vec(),
            "User salt should be updated after commit"
        );

        let saladiers = db::saladiers::list_all_saladiers(&conn, "test_user").unwrap();
        assert_eq!(
            saladiers[0].name_enc, new_s_name_enc,
            "Saladier name should be updated after commit"
        );
    }
}
