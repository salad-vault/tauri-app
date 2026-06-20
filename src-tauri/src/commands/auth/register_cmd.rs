use tauri::State;
use zeroize::Zeroizing;

use crate::crypto::blind_index::EMAIL_BLIND_INDEX_SALT;
use crate::crypto::{argon2_kdf, blind_index, keys, xchacha};
use crate::db;
use crate::error::AppError;
use crate::models::user::User;
use crate::state::AppState;

/// Register a new user account (Potager).
#[tauri::command]
pub async fn register(
    email: String,
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    // Load existing device key or generate a new one BEFORE computing the
    // blind index, so the local PEPPER can be derived from the device key.
    let device_key_path = state.device_key_path();
    let (device_key, is_new_key) = match keys::load_device_key(&device_key_path) {
        Ok(dk) => (dk, false),
        Err(AppError::KeyFileNotFound) => (keys::generate_device_key(), true),
        Err(e) => return Err(e),
    };

    let user_id =
        blind_index::compute_local_blind_index(&email, EMAIL_BLIND_INDEX_SALT, &device_key)?;

    // Check if user already exists BEFORE saving a new device key.
    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        if db::users::get_user(&db_lock, &user_id).is_ok() {
            return Err(AppError::UserAlreadyExists);
        }
    }

    let salt_master = argon2_kdf::generate_salt();
    let salt_sync = argon2_kdf::generate_salt();

    // Derive master key and sync key in parallel.
    // SV-M3: Zeroizing wipes the password copies on drop (incl. after move).
    let pwd_bytes = Zeroizing::new(master_password.into_bytes());
    let pwd_for_sync = pwd_bytes.clone();
    let dk = device_key;
    let sm = salt_master;
    let ss = salt_sync;
    let mk_handle = tokio::task::spawn_blocking(move || keys::reconstruct_master_key(&pwd_bytes, &dk, &sm));
    let sk_handle = tokio::task::spawn_blocking(move || keys::derive_sync_key(&pwd_for_sync, &ss));
    let master_key: keys::MasterKey = mk_handle.await.map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;
    let sync_key: keys::SyncKey = sk_handle.await.map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    let verification_data = b"SALADVAULT_VERIFIED";
    let (nonce, ciphertext) = xchacha::encrypt(master_key.as_bytes(), verification_data)?;

    let mut k_cloud_enc = nonce;
    k_cloud_enc.extend_from_slice(&ciphertext);

    let user = User {
        id: user_id.clone(),
        salt_master: salt_master.to_vec(),
        k_cloud_enc,
        recovery_confirmed: false,
        salt_sync: Some(salt_sync.to_vec()),
    };

    // Persist device key only if it was freshly generated
    if is_new_key {
        keys::save_device_key(&device_key, &device_key_path)?;
    }

    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::users::create_user(&db_lock, &user)?;
    }

    {
        let mut session = state.session.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        *session = Some(crate::state::Session {
            user_id,
            master_key_bytes: *master_key.as_bytes(),
            sync_key_bytes: Some(*sync_key.as_bytes()),
        });
    }

    Ok(())
}
