use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::crypto::{argon2_kdf, blind_index};
use crate::db;
use crate::error::AppError;
use crate::state::AppState;
use crate::commands::device::qr_to_svg;
use crate::sync::client::{
    ApiClient, DeadmanConfigRequest, LoginRequest, MfaSetupConfirmRequest, MfaVerifyRequest,
    RegisterRequest, SyncPushRequest,
};
use crate::sync::{export, import, recovery};

use crate::crypto::blind_index::EMAIL_BLIND_INDEX_SALT;

// ── Response types for the frontend ──

#[derive(Serialize)]
pub struct SyncStatus {
    pub version: i64,
    pub updated_at: String,
}

// ── Helper: get API client ──

fn get_api_client(state: &AppState) -> Result<ApiClient, AppError> {
    let url = state.api_base_url.lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if url.is_empty() {
        return Err(AppError::Internal("URL du serveur non configurée".to_string()));
    }
    Ok(ApiClient::new(&url))
}

fn get_access_token(state: &AppState) -> Result<String, AppError> {
    let tokens = state.server_tokens.lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match tokens.as_ref() {
        Some(t) => Ok(t.access_token.clone()),
        None => Err(AppError::Internal("Non connecté au serveur".to_string())),
    }
}

fn get_refresh_token(state: &AppState) -> Result<String, AppError> {
    let tokens = state.server_tokens.lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match tokens.as_ref() {
        Some(t) => Ok(t.refresh_token.clone()),
        None => Err(AppError::Internal("Non connecté au serveur".to_string())),
    }
}

/// Try to refresh the access token using the stored refresh token.
/// Updates the tokens in state and persists them to disk on success.
async fn try_refresh_token(state: &AppState) -> Result<String, AppError> {
    let refresh_tok = get_refresh_token(state)?;
    let client = get_api_client(state)?;

    let resp = client.refresh_token(&refresh_tok).await?;

    let new_access = resp.access_token.clone();

    let api_url = state.api_base_url.lock()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .clone();

    {
        let mut tokens = state.server_tokens.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *tokens = Some(crate::state::ServerTokens {
            access_token: resp.access_token.clone(),
            refresh_token: resp.refresh_token.clone(),
        });
    }

    // Persist rotated tokens to disk
    if let Ok((user_id, master_key)) = state.require_session() {
        let conn = state.db.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let _ = db::server_auth::save(&conn, &user_id, &master_key, &db::server_auth::ServerAuthData {
            api_url,
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
        });
    }

    Ok(new_access)
}


/// Compute the server auth_hash from the user's master password.
/// This is a separate Argon2id derivation with a different salt,
/// so the server never sees the actual master key material.
/// Runs in a blocking thread to avoid blocking the async runtime.
async fn compute_server_auth(password: String, salt: Vec<u8>) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let derived = argon2_kdf::derive_key(password.as_bytes(), &salt)?;
        Ok(hex::encode(derived))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {e}")))?
}

// ── Auth commands ──

/// MFA setup info returned to the frontend after registration step 1.
#[derive(Serialize)]
pub struct MfaSetupInfo {
    pub mfa_setup_token: String,
    pub totp_secret_base32: String,
    pub totp_uri: String,
    pub qr_svg: String,
}

/// MFA challenge info returned to the frontend after login step 1.
#[derive(Serialize)]
pub struct MfaChallengeInfo {
    pub mfa_challenge_token: String,
}

#[derive(Deserialize)]
pub struct ServerRegisterArgs {
    pub email: String,
    #[serde(rename = "serverPassword")]
    pub server_password: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
}

/// Step 1: Register a new account on the SaladVault server.
/// Returns MFA setup data (QR code, TOTP secret). No tokens yet.
#[tauri::command]
pub async fn server_register(
    state: State<'_, AppState>,
    args: ServerRegisterArgs,
) -> Result<MfaSetupInfo, AppError> {
    let blind_id = blind_index::compute_blind_index(&args.email, EMAIL_BLIND_INDEX_SALT)?;
    let auth_salt = argon2_kdf::generate_salt();
    let auth_hash = compute_server_auth(args.server_password, auth_salt.to_vec()).await?;

    // Save API URL
    {
        let mut url = state.api_base_url.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *url = args.api_url;
    }

    let client = get_api_client(&state)?;
    let b64 = base64::engine::general_purpose::STANDARD;

    let resp = client
        .register(&RegisterRequest {
            blind_id,
            auth_hash,
            auth_salt: b64.encode(auth_salt),
        })
        .await?;

    // Generate QR code SVG from the TOTP URI
    let qr = qrcodegen::QrCode::encode_text(&resp.totp_uri, qrcodegen::QrCodeEcc::Medium)
        .map_err(|e| AppError::Internal(format!("QR code generation error: {e}")))?;
    let qr_svg = qr_to_svg(&qr, 4);

    Ok(MfaSetupInfo {
        mfa_setup_token: resp.mfa_setup_token,
        totp_secret_base32: resp.totp_secret_base32,
        totp_uri: resp.totp_uri,
        qr_svg,
    })
}

#[derive(Deserialize)]
pub struct MfaConfirmArgs {
    #[serde(rename = "mfaSetupToken")]
    pub mfa_setup_token: String,
    #[serde(rename = "totpCode")]
    pub totp_code: String,
}

/// Step 2: Confirm MFA setup with a TOTP code from the authenticator app.
/// Completes registration and stores server tokens (in memory + persisted to disk).
#[tauri::command]
pub async fn server_register_confirm_mfa(
    state: State<'_, AppState>,
    args: MfaConfirmArgs,
) -> Result<(), AppError> {
    let client = get_api_client(&state)?;

    let resp = client
        .mfa_setup_confirm(&MfaSetupConfirmRequest {
            mfa_setup_token: args.mfa_setup_token,
            totp_code: args.totp_code,
        })
        .await?;

    let api_url = state.api_base_url.lock()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .clone();

    // Store tokens in memory
    {
        let mut tokens = state.server_tokens.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *tokens = Some(crate::state::ServerTokens {
            access_token: resp.access_token.clone(),
            refresh_token: resp.refresh_token.clone(),
        });
    }

    // Persist tokens to disk (encrypted with master key)
    if let Ok((user_id, master_key)) = state.require_session() {
        let conn = state.db.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let _ = db::server_auth::save(&conn, &user_id, &master_key, &db::server_auth::ServerAuthData {
            api_url,
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
        });
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct ServerLoginArgs {
    pub email: String,
    #[serde(rename = "serverPassword")]
    pub server_password: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
}

/// Step 1: Log into the SaladVault server.
/// Returns an MFA challenge token. No JWT tokens yet.
#[tauri::command]
pub async fn server_login(
    state: State<'_, AppState>,
    args: ServerLoginArgs,
) -> Result<MfaChallengeInfo, AppError> {
    let blind_id = blind_index::compute_blind_index(&args.email, EMAIL_BLIND_INDEX_SALT)?;

    // Save API URL
    {
        let mut url = state.api_base_url.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *url = args.api_url;
    }

    let client = get_api_client(&state)?;

    // First get the salt from the server
    let salt_resp = client.get_salt(&blind_id).await?;
    let b64 = base64::engine::general_purpose::STANDARD;
    let auth_salt = b64.decode(&salt_resp.auth_salt)
        .map_err(|_| AppError::Internal("Invalid salt from server".to_string()))?;

    // Compute auth_hash with the server's salt
    let auth_hash = compute_server_auth(args.server_password, auth_salt).await?;

    let resp = client
        .login(&LoginRequest {
            blind_id,
            auth_hash,
        })
        .await?;

    Ok(MfaChallengeInfo {
        mfa_challenge_token: resp.mfa_challenge_token,
    })
}

#[derive(Deserialize)]
pub struct MfaVerifyArgs {
    #[serde(rename = "mfaChallengeToken")]
    pub mfa_challenge_token: String,
    #[serde(rename = "totpCode")]
    pub totp_code: String,
}

/// Step 2: Verify TOTP code to complete login.
/// Stores server tokens on success (in memory + persisted to disk).
#[tauri::command]
pub async fn server_login_verify_mfa(
    state: State<'_, AppState>,
    args: MfaVerifyArgs,
) -> Result<(), AppError> {
    let client = get_api_client(&state)?;

    let resp = client
        .mfa_verify(&MfaVerifyRequest {
            mfa_challenge_token: args.mfa_challenge_token,
            totp_code: args.totp_code,
        })
        .await?;

    let api_url = state.api_base_url.lock()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .clone();

    // Store tokens in memory
    {
        let mut tokens = state.server_tokens.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *tokens = Some(crate::state::ServerTokens {
            access_token: resp.access_token.clone(),
            refresh_token: resp.refresh_token.clone(),
        });
    }

    // Persist tokens to disk (encrypted with master key)
    if let Ok((user_id, master_key)) = state.require_session() {
        let conn = state.db.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let _ = db::server_auth::save(&conn, &user_id, &master_key, &db::server_auth::ServerAuthData {
            api_url,
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
        });
    }

    // Best-effort heartbeat to update last_seen_at on the server after login
    if let Ok(token) = get_access_token(&state) {
        let _ = client.deadman_heartbeat(&token).await;
    }

    Ok(())
}

/// Log out from the SaladVault server.
#[tauri::command]
pub async fn server_logout(state: State<'_, AppState>) -> Result<(), AppError> {
    // Best-effort logout — ignore errors (token may already be expired)
    if let Ok(token) = get_access_token(&state) {
        if let Ok(client) = get_api_client(&state) {
            let _ = client.logout(&token).await;
        }
    }

    // Clear tokens from memory
    {
        let mut tokens = state.server_tokens.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *tokens = None;
    }

    // Delete persisted tokens from disk
    if let Ok((user_id, _)) = state.require_session() {
        let conn = state.db.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let _ = db::server_auth::delete(&conn, &user_id);
    }

    Ok(())
}

/// Check if connected to the server.
#[tauri::command]
pub async fn server_is_connected(state: State<'_, AppState>) -> Result<bool, AppError> {
    let tokens = state.server_tokens.lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(tokens.is_some())
}

// ── Sync commands ──

/// Get the current sync status from the server.
#[tauri::command]
pub async fn sync_status(state: State<'_, AppState>) -> Result<SyncStatus, AppError> {
    let token = get_access_token(&state)?;
    let client = get_api_client(&state)?;

    let resp = match client.sync_status(&token).await {
        Err(AppError::ServerUnauthorized) => {
            let new_token = try_refresh_token(&state).await?;
            client.sync_status(&new_token).await?
        }
        other => other?,
    };

    Ok(SyncStatus {
        version: resp.version,
        updated_at: resp.updated_at,
    })
}

/// Push the local database to the server (encrypted).
#[tauri::command]
pub async fn sync_push(state: State<'_, AppState>) -> Result<SyncStatus, AppError> {
    let mut token = get_access_token(&state)?;
    let client = get_api_client(&state)?;

    // Get current server version (with refresh retry)
    let server_status = match client.sync_status(&token).await {
        Err(AppError::ServerUnauthorized) => {
            token = try_refresh_token(&state).await?;
            client.sync_status(&token).await?
        }
        other => other?,
    };
    let new_version = server_status.version + 1;

    // Export local DB as encrypted blob
    let (_, master_key) = state.require_session()?;
    let vault_blob = {
        let conn = state.db.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        export::export_vault(&conn, &master_key)?
    };

    let push_req = SyncPushRequest {
        vault_blob,
        version: new_version,
    };

    let resp = match client.sync_push(&token, &push_req).await {
        Err(AppError::ServerUnauthorized) => {
            let new_token = try_refresh_token(&state).await?;
            client.sync_push(&new_token, &push_req).await?
        }
        other => other?,
    };

    Ok(SyncStatus {
        version: resp.version,
        updated_at: resp.updated_at,
    })
}

/// Pull the vault from the server and replace local data.
#[tauri::command]
pub async fn sync_pull(state: State<'_, AppState>) -> Result<SyncStatus, AppError> {
    let token = get_access_token(&state)?;
    let client = get_api_client(&state)?;

    let resp = match client.sync_pull(&token).await {
        Err(AppError::ServerUnauthorized) => {
            let new_token = try_refresh_token(&state).await?;
            client.sync_pull(&new_token).await?
        }
        other => other?,
    };

    // Decrypt and import
    let (_, master_key) = state.require_session()?;
    {
        let conn = state.db.lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        import::import_vault(&conn, &master_key, &resp.vault_blob)?;
    }

    Ok(SyncStatus {
        version: resp.version,
        updated_at: resp.updated_at,
    })
}

// ── Dead Man's Switch commands ──

#[derive(Serialize)]
pub struct DeadmanStatus {
    pub enabled: bool,
    pub inactivity_days: u32,
    pub last_seen_at: String,
}

/// Get the Dead Man's Switch status from the server.
#[tauri::command]
pub async fn deadman_status(state: State<'_, AppState>) -> Result<DeadmanStatus, AppError> {
    let token = get_access_token(&state)?;
    let client = get_api_client(&state)?;

    let resp = match client.deadman_status(&token).await {
        Err(AppError::ServerUnauthorized) => {
            let new_token = try_refresh_token(&state).await?;
            client.deadman_status(&new_token).await?
        }
        other => other?,
    };

    Ok(DeadmanStatus {
        enabled: resp.enabled,
        inactivity_days: resp.inactivity_days,
        last_seen_at: resp.last_seen_at,
    })
}

/// Send a heartbeat to the server.
#[tauri::command]
pub async fn deadman_heartbeat(state: State<'_, AppState>) -> Result<(), AppError> {
    let token = get_access_token(&state)?;
    let client = get_api_client(&state)?;

    match client.deadman_heartbeat(&token).await {
        Err(AppError::ServerUnauthorized) => {
            let new_token = try_refresh_token(&state).await?;
            client.deadman_heartbeat(&new_token).await?;
        }
        other => { other?; }
    };

    Ok(())
}

#[derive(Deserialize)]
pub struct DeadmanConfigArgs {
    pub enabled: bool,
    pub days: u32,
    #[serde(rename = "recipientEmail")]
    pub recipient_email: String,
    #[serde(rename = "recoveryBlob")]
    pub recovery_blob: Option<String>,
}

/// Update the Dead Man's Switch configuration on the server.
#[tauri::command]
pub async fn deadman_update_config(
    state: State<'_, AppState>,
    args: DeadmanConfigArgs,
) -> Result<(), AppError> {
    let token = get_access_token(&state)?;
    let client = get_api_client(&state)?;

    let req = DeadmanConfigRequest {
        enabled: args.enabled,
        inactivity_days: args.days,
        recipient_email: args.recipient_email,
        recovery_blob_enc: args.recovery_blob,
    };

    match client.deadman_update_config(&token, &req).await {
        Err(AppError::ServerUnauthorized) => {
            let new_token = try_refresh_token(&state).await?;
            client.deadman_update_config(&new_token, &req).await?;
        }
        other => { other?; }
    };

    Ok(())
}

// ── Subscription commands ──

/// Get the current subscription status.
#[tauri::command]
pub async fn subscription_status(
    state: State<'_, AppState>,
) -> Result<crate::sync::client::SubscriptionStatusResponse, AppError> {
    let token = get_access_token(&state)?;
    let client = get_api_client(&state)?;

    match client.subscription_status(&token).await {
        Err(AppError::ServerUnauthorized) => {
            let new_token = try_refresh_token(&state).await?;
            client.subscription_status(&new_token).await
        }
        other => other,
    }
}

/// Create a Stripe Checkout session and return the URL.
#[tauri::command]
pub async fn subscription_checkout(
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let token = get_access_token(&state)?;
    let client = get_api_client(&state)?;

    let resp = match client.subscription_checkout(&token).await {
        Err(AppError::ServerUnauthorized) => {
            let new_token = try_refresh_token(&state).await?;
            client.subscription_checkout(&new_token).await?
        }
        other => other?,
    };

    Ok(resp.checkout_url)
}

/// Create a Stripe Customer Portal session and return the URL.
#[tauri::command]
pub async fn subscription_portal(
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let token = get_access_token(&state)?;
    let client = get_api_client(&state)?;

    let resp = match client.subscription_portal(&token).await {
        Err(AppError::ServerUnauthorized) => {
            let new_token = try_refresh_token(&state).await?;
            client.subscription_portal(&new_token).await?
        }
        other => other?,
    };

    Ok(resp.portal_url)
}

// ── Recovery Kit commands ──

#[derive(Deserialize)]
pub struct GenerateRecoveryKitArgs {
    #[serde(rename = "recoveryPassword")]
    pub recovery_password: String,
}

/// Generate a recovery kit blob encrypted with the given recovery password.
/// Returns the base64-encoded blob. The frontend is responsible for uploading
/// it to the server via `deadman_update_config`.
#[tauri::command]
pub async fn generate_recovery_kit(
    state: State<'_, AppState>,
    args: GenerateRecoveryKitArgs,
) -> Result<String, AppError> {
    if args.recovery_password.len() < 8 {
        return Err(AppError::Internal(
            "Le mot de passe de secours doit contenir au moins 8 caractères".to_string(),
        ));
    }

    let (_, master_key) = state.require_session()?;
    let conn = state.db.lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let blob = recovery::generate_recovery_blob(&conn, &master_key, &args.recovery_password)?;

    Ok(blob)
}
