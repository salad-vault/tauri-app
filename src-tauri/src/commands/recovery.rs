use bip39::Mnemonic;
use tauri::State;

use crate::crypto::keys;
use crate::db;
use crate::error::AppError;
use crate::state::AppState;

/// Generate a 24-word BIP39 recovery phrase from the current device key.
/// This phrase allows the user to mathematically regenerate their device_secret.key.
/// The user MUST print this and store it securely.
#[tauri::command]
pub async fn generate_recovery_phrase(
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let _ = state.require_session()?;

    let device_key_path = state.device_key_path();
    let device_key = keys::load_device_key(&device_key_path)?;

    // BIP39 mnemonic from 32 bytes (256 bits) = 24 words
    let mnemonic = Mnemonic::from_entropy(&device_key)
        .map_err(|e| AppError::Internal(format!("BIP39 error: {e}")))?;

    Ok(mnemonic.to_string())
}

/// Recover the device key from a 24-word BIP39 phrase.
/// Regenerates and saves the device_secret.key file.
#[tauri::command]
pub async fn recover_from_phrase(
    phrase: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mnemonic: Mnemonic = phrase
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid recovery phrase: {e}")))?;

    let entropy = mnemonic.to_entropy();
    if entropy.len() != 32 {
        return Err(AppError::Internal(
            "Recovery phrase does not produce a 32-byte key".to_string(),
        ));
    }

    let mut device_key = [0u8; 32];
    device_key.copy_from_slice(&entropy);

    let device_key_path = state.device_key_path();

    // SV-H5: if a device key already exists, back it up (timestamped) before
    // overwriting. A valid-but-wrong BIP39 phrase would otherwise irreversibly
    // destroy access to the existing local vault — the backup makes recovery
    // reversible. (Verifying the candidate key against the account requires the
    // master password, which is not available in this recovery flow.)
    if device_key_path.exists() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let backup = device_key_path.with_file_name(format!("device_secret.key.bak-{ts}"));
        std::fs::copy(&device_key_path, &backup).map_err(|e| {
            AppError::Internal(format!("Cannot back up existing device key: {e}"))
        })?;
        log::warn!("Existing device key backed up before recovery overwrite");
    }

    keys::save_device_key(&device_key, &device_key_path)?;

    Ok(())
}

/// Check whether the current user has confirmed saving their recovery phrase.
/// Returns true if confirmed, false otherwise.
#[tauri::command]
pub async fn check_recovery_status(
    state: State<'_, AppState>,
) -> Result<bool, AppError> {
    let (user_id, _) = state.require_session()?;

    let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    let user = db::users::get_user(&db_lock, &user_id)?;

    Ok(user.recovery_confirmed)
}

/// Mark the current user as having confirmed saving their recovery phrase.
/// This should only be called after the user has generated and acknowledged the phrase.
#[tauri::command]
pub async fn confirm_recovery_saved(
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let (user_id, _) = state.require_session()?;

    let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
    db::users::set_recovery_confirmed(&db_lock, &user_id)?;

    Ok(())
}
