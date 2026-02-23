use tauri::State;
use uuid::Uuid;

use crate::crypto::{argon2_kdf, xchacha};
use crate::db;
use crate::error::AppError;
use crate::models::feuille::{Feuille, FeuilleData, FeuilleInfo};
use crate::state::AppState;

/// Helper: get the Saladier key from the open cache.
fn get_saladier_key(state: &AppState, saladier_id: &str) -> Result<[u8; 32], AppError> {
    let cache = state
        .open_saladiers_cache()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    cache
        .get(saladier_id)
        .copied()
        .ok_or(AppError::SaladierLocked)
}

/// Create a new Feuille (entry) in a Saladier.
#[tauri::command]
pub async fn create_feuille(
    saladier_id: String,
    data: FeuilleData,
    state: State<'_, AppState>,
) -> Result<FeuilleInfo, AppError> {
    let _ = state.require_session()?;
    let k_s = get_saladier_key(&state, &saladier_id)?;

    let feuille_uuid = Uuid::new_v4().to_string();

    let json_data = serde_json::to_vec(&data)
        .map_err(|e| AppError::Internal(format!("Serialization error: {e}")))?;
    let (nonce, ciphertext) = xchacha::encrypt(&k_s, &json_data)?;

    let feuille = Feuille {
        uuid: feuille_uuid.clone(),
        saladier_id: saladier_id.clone(),
        data_blob: ciphertext,
        nonce,
    };

    let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    db::feuilles::create_feuille(&db_lock, &feuille)?;

    Ok(FeuilleInfo {
        uuid: feuille_uuid,
        saladier_id,
        data,
    })
}

/// Get a decrypted Feuille by UUID.
#[tauri::command]
pub async fn get_feuille(
    uuid: String,
    state: State<'_, AppState>,
) -> Result<FeuilleInfo, AppError> {
    let _ = state.require_session()?;

    let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let feuille = db::feuilles::get_feuille(&db_lock, &uuid)?;

    let k_s = get_saladier_key(&state, &feuille.saladier_id)?;

    let json_data = xchacha::decrypt(&k_s, &feuille.nonce, &feuille.data_blob)?;
    let data: FeuilleData = serde_json::from_slice(&json_data)
        .map_err(|e| AppError::Internal(format!("Deserialization error: {e}")))?;

    Ok(FeuilleInfo {
        uuid: feuille.uuid,
        saladier_id: feuille.saladier_id,
        data,
    })
}

/// List all Feuilles in a Saladier (decrypted).
#[tauri::command]
pub async fn list_feuilles(
    saladier_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<FeuilleInfo>, AppError> {
    let _ = state.require_session()?;
    let k_s = get_saladier_key(&state, &saladier_id)?;

    let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let feuilles = db::feuilles::list_feuilles(&db_lock, &saladier_id)?;

    let mut result = Vec::new();
    for f in feuilles {
        let json_data = xchacha::decrypt(&k_s, &f.nonce, &f.data_blob)?;
        let data: FeuilleData = serde_json::from_slice(&json_data)
            .map_err(|e| AppError::Internal(format!("Deserialization error: {e}")))?;

        result.push(FeuilleInfo {
            uuid: f.uuid,
            saladier_id: f.saladier_id,
            data,
        });
    }

    Ok(result)
}

/// Update a Feuille's data.
#[tauri::command]
pub async fn update_feuille(
    uuid: String,
    data: FeuilleData,
    state: State<'_, AppState>,
) -> Result<FeuilleInfo, AppError> {
    let _ = state.require_session()?;

    let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let existing = db::feuilles::get_feuille(&db_lock, &uuid)?;
    let k_s = get_saladier_key(&state, &existing.saladier_id)?;

    let json_data = serde_json::to_vec(&data)
        .map_err(|e| AppError::Internal(format!("Serialization error: {e}")))?;
    let (nonce, ciphertext) = xchacha::encrypt(&k_s, &json_data)?;

    let updated = Feuille {
        uuid: uuid.clone(),
        saladier_id: existing.saladier_id.clone(),
        data_blob: ciphertext,
        nonce,
    };

    db::feuilles::update_feuille(&db_lock, &updated)?;

    Ok(FeuilleInfo {
        uuid,
        saladier_id: existing.saladier_id,
        data,
    })
}

/// Delete a Feuille after verifying the Saladier password.
#[tauri::command]
pub async fn delete_feuille(
    uuid: String,
    saladier_password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let _ = state.require_session()?;

    // Get the feuille to find its saladier
    let (_feuille, saladier) = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let f = db::feuilles::get_feuille(&db_lock, &uuid)?;
        let s = db::saladiers::get_saladier(&db_lock, &f.saladier_id)?;
        (f, s)
    };

    // Verify the Saladier password cryptographically
    let pwd = saladier_password.into_bytes();
    let salt = saladier.salt_saladier.clone();
    let k_s = tokio::task::spawn_blocking(move || {
        argon2_kdf::derive_key(&pwd, &salt)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    xchacha::decrypt(&k_s, &saladier.verify_nonce, &saladier.verify_enc)
        .map_err(|_| AppError::InvalidCredentials)?;

    // Password verified, proceed with deletion
    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::feuilles::delete_feuille(&db_lock, &uuid)?;
    }

    Ok(())
}
