use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::crypto::{argon2_kdf, xchacha};
use crate::db;
use crate::error::AppError;
use crate::models::saladier::{Saladier, SaladierInfo};
use crate::state::AppState;

#[derive(Serialize)]
pub struct AttemptsInfo {
    pub failed_attempts: u32,
    pub max_failed_attempts: u32,
    pub remaining: Option<u32>,
}

/// Verification token used to validate Saladier passwords cryptographically.
const SALADIER_VERIFY_TOKEN: &[u8] = b"SALADVAULT_SALADIER_OK";

/// Create a new Saladier (vault) with its own password.
/// If `hidden` is true, the Saladier will be invisible in the dashboard (plausible deniability).
#[tauri::command]
pub async fn create_saladier(
    name: String,
    password: String,
    hidden: bool,
    state: State<'_, AppState>,
) -> Result<SaladierInfo, AppError> {
    let (user_id, master_key_bytes) = state.require_session()?;

    let saladier_uuid = Uuid::new_v4().to_string();

    // Generate a salt for this Saladier's own key derivation
    let salt_saladier = argon2_kdf::generate_salt();

    // Derive K_S from the Saladier password
    let pwd = password.into_bytes();
    let salt = salt_saladier;
    let k_s = tokio::task::spawn_blocking(move || {
        argon2_kdf::derive_key(&pwd, &salt)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    // Encrypt a verification token with K_S so we can verify the password later
    let (verify_nonce, verify_enc) = xchacha::encrypt(&k_s, SALADIER_VERIFY_TOKEN)?;

    // Store K_S in cache for immediate use
    {
        let mut cache = state
            .open_saladiers_cache()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        cache.insert(saladier_uuid.clone(), k_s);
    }

    // Encrypt the Saladier name with the master key
    let (nonce, name_enc) = xchacha::encrypt(&master_key_bytes, name.as_bytes())?;

    let saladier = Saladier {
        uuid: saladier_uuid.clone(),
        user_id,
        name_enc,
        salt_saladier: salt.to_vec(),
        nonce,
        verify_enc,
        verify_nonce,
        hidden,
        failed_attempts: 0,
    };

    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::saladiers::create_saladier(&db_lock, &saladier)?;
    }

    Ok(SaladierInfo {
        uuid: saladier_uuid,
        name,
    })
}

/// List all visible (non-hidden) Saladiers for the current user.
/// Hidden Saladiers are excluded for plausible deniability.
#[tauri::command]
pub async fn list_saladiers(
    state: State<'_, AppState>,
) -> Result<Vec<SaladierInfo>, AppError> {
    let (user_id, master_key_bytes) = state.require_session()?;

    let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let saladiers = db::saladiers::list_saladiers(&db_lock, &user_id)?;

    let mut result = Vec::new();
    for s in saladiers {
        let name = xchacha::decrypt(&master_key_bytes, &s.nonce, &s.name_enc)
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .unwrap_or_else(|_| "[Chiffré]".to_string());

        result.push(SaladierInfo {
            uuid: s.uuid,
            name,
        });
    }

    Ok(result)
}

/// Open (unlock) a Saladier using its specific password (Panic Mode).
/// Verifies the password cryptographically by decrypting the verify token.
/// Tracks failed attempts and auto-destroys if max_failed_attempts is exceeded.
#[tauri::command]
pub async fn open_saladier(
    uuid: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let (user_id, _master_key_bytes) = state.require_session()?;

    let saladier = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::saladiers::get_saladier(&db_lock, &uuid)?
    };

    // Derive the Saladier key K_S from the password (in blocking thread)
    let pwd = password.into_bytes();
    let salt = saladier.salt_saladier.clone();
    let k_s = tokio::task::spawn_blocking(move || {
        argon2_kdf::derive_key(&pwd, &salt)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    // Verify the password by decrypting the verification token
    let decrypted = xchacha::decrypt(&k_s, &saladier.verify_nonce, &saladier.verify_enc);

    match decrypted {
        Ok(ref data) if data == SALADIER_VERIFY_TOKEN => {
            // Password verified! Reset failed attempts and store the key
            {
                let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
                db::saladiers::reset_failed_attempts(&db_lock, &uuid)?;
            }

            let mut cache = state
                .open_saladiers_cache()
                .map_err(|e| AppError::Internal(e.to_string()))?;
            cache.insert(uuid, k_s);

            Ok(())
        }
        _ => {
            // Wrong password - increment failed attempts
            let new_count = {
                let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
                db::saladiers::increment_failed_attempts(&db_lock, &uuid)?
            };

            // Check if we should auto-destroy based on user settings
            let settings = {
                let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
                crate::db::settings::get_settings(&db_lock, &user_id)?
            };

            if settings.max_failed_attempts > 0 && new_count >= settings.max_failed_attempts {
                // Auto-destroy the Saladier
                let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
                db::saladiers::delete_saladier(&db_lock, &uuid)?;
            }

            Err(AppError::InvalidCredentials)
        }
    }
}

/// Try to unlock a hidden Saladier by password.
/// Iterates over all hidden Saladiers for the current user and tries to decrypt
/// each one's verify token. Returns the SaladierInfo if found, None otherwise.
/// This does NOT return an error when no match is found (for plausible deniability).
#[tauri::command]
pub async fn unlock_hidden_saladier(
    password: String,
    state: State<'_, AppState>,
) -> Result<Option<SaladierInfo>, AppError> {
    let (user_id, master_key_bytes) = state.require_session()?;

    let hidden_saladiers = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::saladiers::list_hidden_saladiers(&db_lock, &user_id)?
    };

    if hidden_saladiers.is_empty() {
        return Ok(None);
    }

    // Try each hidden Saladier
    for saladier in hidden_saladiers {
        let pwd = password.as_bytes().to_vec();
        let salt = saladier.salt_saladier.clone();
        let k_s = tokio::task::spawn_blocking(move || {
            argon2_kdf::derive_key(&pwd, &salt)
        })
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

        // Try to decrypt the verify token
        let ok = xchacha::decrypt(&k_s, &saladier.verify_nonce, &saladier.verify_enc)
            .map(|d| d == SALADIER_VERIFY_TOKEN)
            .unwrap_or(false);

        if ok {
            // Match found! Cache K_S and return the info
            {
                let mut cache = state
                    .open_saladiers_cache()
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                cache.insert(saladier.uuid.clone(), k_s);
            }

            let name = xchacha::decrypt(&master_key_bytes, &saladier.nonce, &saladier.name_enc)
                .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                .unwrap_or_else(|_| "[Chiffré]".to_string());

            return Ok(Some(SaladierInfo {
                uuid: saladier.uuid,
                name,
            }));
        }
    }

    // No match found - return None (not an error, for plausible deniability)
    Ok(None)
}

/// Get the number of failed attempts and remaining attempts for a Saladier.
#[tauri::command]
pub async fn get_saladier_attempts_info(
    uuid: String,
    state: State<'_, AppState>,
) -> Result<AttemptsInfo, AppError> {
    let (user_id, _) = state.require_session()?;

    let saladier = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::saladiers::get_saladier(&db_lock, &uuid)?
    };

    let settings = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        crate::db::settings::get_settings(&db_lock, &user_id)?
    };

    let remaining = if settings.max_failed_attempts > 0 {
        Some(settings.max_failed_attempts.saturating_sub(saladier.failed_attempts))
    } else {
        None
    };

    Ok(AttemptsInfo {
        failed_attempts: saladier.failed_attempts,
        max_failed_attempts: settings.max_failed_attempts,
        remaining,
    })
}

/// Delete a Saladier after verifying the user's master password.
#[tauri::command]
pub async fn delete_saladier(
    uuid: String,
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let (_user_id, _master_key_bytes) = state.require_session()?;

    // Verify the master password using the verify_master_password helper
    crate::commands::auth::verify_master_password_inner(&master_password, &state).await?;

    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::saladiers::delete_saladier(&db_lock, &uuid)?;
    }

    Ok(())
}
