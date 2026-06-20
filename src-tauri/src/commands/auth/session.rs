use tauri::State;
use zeroize::Zeroizing;

use crate::crypto::blind_index::EMAIL_BLIND_INDEX_SALT;
use crate::crypto::{argon2_kdf, blind_index, keys, xchacha};
use crate::db;
use crate::error::AppError;
use crate::state::AppState;

/// Unlock the Potager (authenticate with master password).
#[tauri::command]
pub async fn unlock(
    email: String,
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let device_key_path = state.device_key_path();
    let device_key = keys::load_device_key(&device_key_path)?;

    let user_id =
        blind_index::compute_local_blind_index(&email, EMAIL_BLIND_INDEX_SALT, &device_key)?;

    // Scope the db_lock so it is dropped before any .await
    let (user_id, user) = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        match db::users::get_user(&db_lock, &user_id) {
            Ok(u) => (user_id, u),
            Err(_) => {
                // Transparent migration: try legacy blind index (hardcoded PEPPER).
                // Accounts created before the HKDF PEPPER derivation used the
                // compile-time constant. If found, migrate user_id atomically.
                let legacy_id =
                    blind_index::compute_blind_index(&email, EMAIL_BLIND_INDEX_SALT)?;
                let u = db::users::get_user(&db_lock, &legacy_id)?;
                let new_id = blind_index::compute_local_blind_index(
                    &email,
                    EMAIL_BLIND_INDEX_SALT,
                    &device_key,
                )?;
                migrate_user_id(&db_lock, &legacy_id, &new_id)?;
                (new_id, u)
            }
        }
    };

    // Reconstruct master key and sync key in parallel.
    // SV-M3: wrap the password copies in Zeroizing so the heap buffers are wiped
    // on drop (including after being moved into the spawn_blocking closures).
    let pwd_bytes = Zeroizing::new(master_password.into_bytes());
    let pwd_for_sync = pwd_bytes.clone();
    let dk = device_key;
    let salt = user.salt_master.clone();

    // Resolve salt_sync (generate if missing for migration)
    let salt_sync_vec = match user.salt_sync.clone() {
        Some(s) => s,
        None => {
            let new_salt = argon2_kdf::generate_salt();
            let conn = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
            db::users::set_salt_sync(&conn, &user_id, &new_salt)?;
            new_salt.to_vec()
        }
    };
    let ss = salt_sync_vec;

    let mk_handle = tokio::task::spawn_blocking(move || keys::reconstruct_master_key(&pwd_bytes, &dk, &salt));
    let sk_handle = tokio::task::spawn_blocking(move || keys::derive_sync_key(&pwd_for_sync, &ss));
    let master_key: keys::MasterKey = mk_handle.await.map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;
    let sync_key: keys::SyncKey = sk_handle.await.map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

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
            sync_key_bytes: Some(*sync_key.as_bytes()),
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
/// Shared by `verify_master_password`, `change_master_password`, and destructive saladier ops.
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

    let pwd = Zeroizing::new(master_password.as_bytes().to_vec()); // SV-M3
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

/// Migrate a user's primary key from the legacy blind index to the new
/// device-key-derived blind index. Updates all FK references atomically.
fn migrate_user_id(
    conn: &rusqlite::Connection,
    old_id: &str,
    new_id: &str,
) -> Result<(), AppError> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tx.execute(
        "UPDATE users SET id = ?1 WHERE id = ?2",
        rusqlite::params![new_id, old_id],
    )?;
    tx.execute(
        "UPDATE saladiers SET user_id = ?1 WHERE user_id = ?2",
        rusqlite::params![new_id, old_id],
    )?;
    tx.execute(
        "UPDATE settings SET user_id = ?1 WHERE user_id = ?2",
        rusqlite::params![new_id, old_id],
    )?;
    tx.execute(
        "UPDATE server_auth SET user_id = ?1 WHERE user_id = ?2",
        rusqlite::params![new_id, old_id],
    )?;

    tx.commit()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}
