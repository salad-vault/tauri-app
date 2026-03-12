use tauri::State;

use crate::crypto::{argon2_kdf, blind_index, keys, xchacha};
use crate::db;
use crate::error::AppError;
use crate::models::user::User;
use crate::state::AppState;

use crate::crypto::blind_index::EMAIL_BLIND_INDEX_SALT;

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
/// 6. Update DB
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

    // Apply all changes atomically
    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        db_lock.execute(
            "UPDATE users SET salt_master = ?1, k_cloud_enc = ?2 WHERE id = ?3",
            rusqlite::params![new_salt.to_vec(), new_k_cloud_enc, user_id],
        )?;

        for (uuid, name_enc, nonce) in &re_encrypted {
            db::saladiers::update_saladier_name_enc(&db_lock, uuid, name_enc, nonce)?;
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
