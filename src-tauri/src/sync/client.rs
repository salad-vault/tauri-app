use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// HTTP client wrapper for the SaladVault API server.
pub struct ApiClient {
    client: Client,
    base_url: String,
}

// ── Request/Response types ──

#[derive(Serialize)]
pub struct RegisterRequest {
    pub blind_id: String,
    pub auth_hash: String,
    pub auth_salt: String,
}

#[derive(Serialize)]
pub struct LoginRequest {
    pub blind_id: String,
    pub auth_hash: String,
}

#[derive(Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct SyncStatusResponse {
    pub version: i64,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct SyncVaultResponse {
    pub vault_blob: String,
    pub version: i64,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct SyncPushRequest {
    pub vault_blob: String,
    pub version: i64,
}

#[derive(Serialize)]
pub struct DeadmanConfigRequest {
    pub enabled: bool,
    pub inactivity_days: u32,
    pub recipient_email: String,
    pub recovery_blob_enc: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct HeartbeatResponse {
    pub last_seen_at: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct DeadmanStatusResponse {
    pub enabled: bool,
    pub inactivity_days: u32,
    pub last_seen_at: String,
}

#[derive(Deserialize)]
pub struct SaltResponse {
    pub auth_salt: String,
}

// ── MFA types ──

#[derive(Deserialize)]
pub struct MfaSetupResponse {
    pub mfa_setup_token: String,
    pub totp_secret_base32: String,
    pub totp_uri: String,
}

#[derive(Serialize)]
pub struct MfaSetupConfirmRequest {
    pub mfa_setup_token: String,
    pub totp_code: String,
}

#[derive(Deserialize)]
pub struct MfaLoginChallengeResponse {
    pub mfa_challenge_token: String,
}

#[derive(Serialize)]
pub struct MfaVerifyRequest {
    pub mfa_challenge_token: String,
    pub totp_code: String,
}

// ── Subscription types ──

#[derive(Deserialize, Serialize, Clone)]
pub struct SubscriptionStatusResponse {
    pub plan: String,
    pub status: String,
    pub trial_end: Option<String>,
    pub current_period_end: Option<String>,
}

#[derive(Deserialize)]
pub struct CheckoutSessionResponse {
    pub checkout_url: String,
}

#[derive(Deserialize)]
pub struct PortalSessionResponse {
    pub portal_url: String,
}

#[derive(Deserialize)]
struct ApiErrorResponse {
    error: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        let enforce_https = base_url.starts_with("https://");

        let client = Client::builder()
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .https_only(enforce_https)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Convert a reqwest network error into a user-visible ServerError.
    fn network_error(e: reqwest::Error) -> AppError {
        if e.is_connect() {
            AppError::ServerError("Impossible de contacter le serveur. Vérifiez l'URL et que le serveur est en ligne.".to_string())
        } else if e.is_timeout() {
            AppError::ServerError("Le serveur ne répond pas (timeout).".to_string())
        } else {
            AppError::ServerError(format!("Erreur réseau : {e}"))
        }
    }

    /// Extract a user-friendly error message from a non-2xx response.
    /// Returns `ServerUnauthorized` for 401 so callers can trigger token refresh.
    async fn extract_error(resp: reqwest::Response) -> AppError {
        let status = resp.status();
        let msg = resp
            .json::<ApiErrorResponse>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("Erreur serveur ({})", status));
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return AppError::ServerUnauthorized;
        }
        AppError::Internal(msg)
    }

    /// Extract error for auth endpoints (login/register).
    /// Always returns the server's error message instead of generic ServerUnauthorized.
    /// Server messages are already sanitized by the API, so they are safe to display.
    async fn extract_auth_error(resp: reqwest::Response) -> AppError {
        let status = resp.status();
        let msg = resp
            .json::<ApiErrorResponse>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("Erreur serveur ({})", status));
        AppError::ServerError(msg)
    }

    // ── Auth ──

    pub async fn register(&self, req: &RegisterRequest) -> Result<MfaSetupResponse, AppError> {
        let resp = self
            .client
            .post(self.url("/auth/register"))
            .json(req)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_auth_error(resp).await)
        }
    }

    pub async fn mfa_setup_confirm(
        &self,
        req: &MfaSetupConfirmRequest,
    ) -> Result<AuthResponse, AppError> {
        let resp = self
            .client
            .post(self.url("/auth/mfa/setup/confirm"))
            .json(req)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_auth_error(resp).await)
        }
    }

    pub async fn get_salt(&self, blind_id: &str) -> Result<SaltResponse, AppError> {
        let resp = self
            .client
            .get(self.url(&format!("/auth/salt/{blind_id}")))
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_auth_error(resp).await)
        }
    }

    pub async fn login(&self, req: &LoginRequest) -> Result<MfaLoginChallengeResponse, AppError> {
        let resp = self
            .client
            .post(self.url("/auth/login"))
            .json(req)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_auth_error(resp).await)
        }
    }

    pub async fn mfa_verify(&self, req: &MfaVerifyRequest) -> Result<AuthResponse, AppError> {
        let resp = self
            .client
            .post(self.url("/auth/mfa/verify"))
            .json(req)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_auth_error(resp).await)
        }
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        let resp = self
            .client
            .post(self.url("/auth/refresh"))
            .json(&RefreshRequest {
                refresh_token: refresh_token.to_string(),
            })
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    pub async fn logout(&self, access_token: &str) -> Result<(), AppError> {
        let resp = self
            .client
            .post(self.url("/auth/logout"))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    // ── Sync ──

    pub async fn sync_status(&self, access_token: &str) -> Result<SyncStatusResponse, AppError> {
        let resp = self
            .client
            .get(self.url("/sync/status"))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    pub async fn sync_pull(&self, access_token: &str) -> Result<SyncVaultResponse, AppError> {
        let resp = self
            .client
            .get(self.url("/sync/vault"))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    pub async fn sync_push(
        &self,
        access_token: &str,
        req: &SyncPushRequest,
    ) -> Result<SyncStatusResponse, AppError> {
        let resp = self
            .client
            .put(self.url("/sync/vault"))
            .bearer_auth(access_token)
            .json(req)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    // ── Dead Man's Switch ──

    pub async fn deadman_heartbeat(
        &self,
        access_token: &str,
    ) -> Result<HeartbeatResponse, AppError> {
        let resp = self
            .client
            .post(self.url("/deadman/heartbeat"))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    #[allow(dead_code)]
    pub async fn deadman_status(
        &self,
        access_token: &str,
    ) -> Result<DeadmanStatusResponse, AppError> {
        let resp = self
            .client
            .get(self.url("/deadman/status"))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    // ── Subscription ──

    pub async fn subscription_status(
        &self,
        access_token: &str,
    ) -> Result<SubscriptionStatusResponse, AppError> {
        let resp = self
            .client
            .get(self.url("/subscription/status"))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    pub async fn subscription_checkout(
        &self,
        access_token: &str,
    ) -> Result<CheckoutSessionResponse, AppError> {
        let resp = self
            .client
            .post(self.url("/subscription/checkout"))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    pub async fn subscription_portal(
        &self,
        access_token: &str,
    ) -> Result<PortalSessionResponse, AppError> {
        let resp = self
            .client
            .post(self.url("/subscription/portal"))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            resp.json().await.map_err(|e| AppError::Internal(e.to_string()))
        } else {
            Err(Self::extract_error(resp).await)
        }
    }

    pub async fn deadman_update_config(
        &self,
        access_token: &str,
        req: &DeadmanConfigRequest,
    ) -> Result<(), AppError> {
        let resp = self
            .client
            .put(self.url("/deadman/config"))
            .bearer_auth(access_token)
            .json(req)
            .send()
            .await
            .map_err(Self::network_error)?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Self::extract_error(resp).await)
        }
    }
}
