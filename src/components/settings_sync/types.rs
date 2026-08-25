use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(super) struct ServerAuthArgs {
    pub email: String,
    #[serde(rename = "serverPassword")]
    pub server_password: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
}

#[derive(Serialize)]
pub(super) struct MfaConfirmArgs {
    #[serde(rename = "mfaSetupToken")]
    pub mfa_setup_token: String,
    #[serde(rename = "totpCode")]
    pub totp_code: String,
}

#[derive(Serialize)]
pub(super) struct MfaVerifyArgs {
    #[serde(rename = "mfaChallengeToken")]
    pub mfa_challenge_token: String,
    #[serde(rename = "totpCode")]
    pub totp_code: String,
}

#[derive(Deserialize)]
pub(super) struct SyncStatus {
    pub version: i64,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub(super) struct MfaSetupInfo {
    pub mfa_setup_token: String,
    pub totp_secret_base32: String,
    #[allow(dead_code)]
    pub totp_uri: String,
    pub qr_svg: String,
}

#[derive(Deserialize)]
pub(super) struct MfaChallengeInfo {
    pub mfa_challenge_token: String,
}

#[derive(Serialize)]
pub(super) struct SendVerificationArgs {
    pub email: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
}

#[derive(Serialize)]
pub(super) struct VerifyCodeArgs {
    pub email: String,
    pub code: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
}

#[derive(Serialize)]
pub(super) struct DeleteAccountArgs {
    #[serde(rename = "totpCode")]
    pub totp_code: String,
}

#[derive(Deserialize)]
pub(super) struct DeadmanStatus {
    pub enabled: bool,
    pub inactivity_days: u32,
    pub last_seen_at: String,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum MfaPhase {
    None,
    EmailVerification,
    Setup,
    Challenge,
}
