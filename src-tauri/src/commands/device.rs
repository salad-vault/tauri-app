use tauri::State;

use crate::crypto::{keys, xchacha};
use crate::db;
use crate::error::AppError;
use crate::state::AppState;

/// Convert a QrCode to an SVG string.
pub fn qr_to_svg(qr: &qrcodegen::QrCode, border: i32) -> String {
    let size = qr.size();
    let dim = size + border * 2;
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {dim} {dim}" shape-rendering="crispEdges">"#
    );
    svg.push_str(&format!(
        r##"<rect width="{dim}" height="{dim}" fill="#ffffff"/><path d=""##
    ));
    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x, y) {
                svg.push_str(&format!("M{},{}h1v1h-1z", x + border, y + border));
            }
        }
    }
    svg.push_str(r##"" fill="#000000"/></svg>"##);
    svg
}

/// Initialize a new device key if one doesn't already exist.
/// Returns true if a new key was created, false if one already exists.
#[tauri::command]
pub async fn init_device_key(state: State<'_, AppState>) -> Result<bool, AppError> {
    let path = state.device_key_path();
    if keys::device_key_exists(&path) {
        return Ok(false);
    }

    let key = keys::generate_device_key();
    keys::save_device_key(&key, &path)?;
    Ok(true)
}

/// Check if a device key file exists on this device.
#[tauri::command]
pub async fn check_device_key(state: State<'_, AppState>) -> Result<bool, AppError> {
    let path = state.device_key_path();
    Ok(keys::device_key_exists(&path))
}

/// Get the path to the device_secret.key file.
#[tauri::command]
pub async fn get_device_key_path(state: State<'_, AppState>) -> Result<String, AppError> {
    let path = state.device_key_path();
    Ok(path.to_string_lossy().to_string())
}

/// Move the device key to a new location (e.g., USB drive for cold storage).
/// Uses the Tauri file dialog for path selection on the frontend.
#[tauri::command]
pub async fn move_device_key(
    new_path: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let current_path = state.device_key_path();

    if !current_path.exists() {
        return Err(AppError::KeyFileNotFound);
    }

    let new_path = std::path::PathBuf::from(&new_path);

    // Ensure the target directory exists
    if let Some(parent) = new_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Copy the file to the new location
    std::fs::copy(&current_path, &new_path)?;

    // Verify the copy is successful by reading both
    let original = std::fs::read(&current_path)?;
    let copied = std::fs::read(&new_path)?;

    if original != copied {
        // Remove the failed copy
        let _ = std::fs::remove_file(&new_path);
        return Err(AppError::Internal("Copy verification failed".to_string()));
    }

    // Remove the original
    std::fs::remove_file(&current_path)?;

    Ok(new_path.to_string_lossy().to_string())
}

/// Export the device key as a base64 string for QR code display.
#[tauri::command]
pub async fn export_device_key_qrcode(state: State<'_, AppState>) -> Result<String, AppError> {
    let path = state.device_key_path();
    let key = keys::load_device_key(&path)?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        key,
    ))
}

/// Export the device key as a QR code SVG string.
#[tauri::command]
pub async fn generate_device_key_qr_svg(state: State<'_, AppState>) -> Result<String, AppError> {
    let path = state.device_key_path();
    let key = keys::load_device_key(&path)?;
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key);
    let qr = qrcodegen::QrCode::encode_text(&b64, qrcodegen::QrCodeEcc::Medium)
        .map_err(|e| AppError::Internal(format!("QR code generation error: {e}")))?;
    Ok(qr_to_svg(&qr, 4))
}

/// Regenerate the device key.
/// 1. Generate a new device_secret.key
/// 2. Re-derive the master key with the new device key
/// 3. Re-encrypt k_cloud_enc
/// 4. Re-encrypt all Saladier names
/// 5. Save the new file
/// 6. Reset recovery_confirmed = false
#[tauri::command]
pub async fn regenerate_device_key(
    master_password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    // Verify the master password first
    crate::commands::auth::verify_master_password_inner(&master_password, &state).await?;

    let (user_id, old_master_key) = state.require_session()?;

    let device_key_path = state.device_key_path();

    // Load user data
    let user = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::users::get_user(&db_lock, &user_id)?
    };

    // Generate new device key
    let new_device_key = keys::generate_device_key();

    // Re-derive the master key with the new device key
    let pwd = master_password.into_bytes();
    let dk = new_device_key;
    let salt = user.salt_master.clone();
    let new_master_key = tokio::task::spawn_blocking(move || {
        keys::reconstruct_master_key(&pwd, &dk, &salt)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))??;

    // Re-encrypt k_cloud_enc with the new master key
    let verification_data = b"SALADVAULT_VERIFIED";
    let (nonce, ciphertext) = xchacha::encrypt(new_master_key.as_bytes(), verification_data)?;
    let mut new_k_cloud_enc = nonce;
    new_k_cloud_enc.extend_from_slice(&ciphertext);

    // Re-encrypt all Saladier names
    let saladiers = {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        db::saladiers::list_all_saladiers(&db_lock, &user_id)?
    };

    // Decrypt with old key, re-encrypt with new key
    let mut re_encrypted: Vec<(String, Vec<u8>, Vec<u8>)> = Vec::new();
    for s in &saladiers {
        let name_bytes = xchacha::decrypt(&old_master_key, &s.nonce, &s.name_enc)
            .unwrap_or_else(|_| b"[Error]".to_vec());
        let (new_nonce, new_name_enc) = xchacha::encrypt(new_master_key.as_bytes(), &name_bytes)?;
        re_encrypted.push((s.uuid.clone(), new_name_enc, new_nonce));
    }

    // Apply all changes in a single DB transaction
    {
        let db_lock = state.db.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        // Update user's k_cloud_enc
        db_lock.execute(
            "UPDATE users SET k_cloud_enc = ?1, recovery_confirmed = 0 WHERE id = ?2",
            rusqlite::params![new_k_cloud_enc, user_id],
        )?;

        // Update all Saladier names
        for (uuid, name_enc, nonce) in &re_encrypted {
            db::saladiers::update_saladier_name_enc(&db_lock, uuid, name_enc, nonce)?;
        }
    }

    // Save the new device key
    keys::save_device_key(&new_device_key, &device_key_path)?;

    // Update session with new master key
    {
        let mut session = state.session.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        *session = Some(crate::state::Session {
            user_id,
            master_key_bytes: *new_master_key.as_bytes(),
        });
    }

    Ok(())
}
