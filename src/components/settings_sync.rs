use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

// ── Types ──

#[derive(Serialize)]
struct ServerAuthArgs {
    email: String,
    #[serde(rename = "serverPassword")]
    server_password: String,
    #[serde(rename = "apiUrl")]
    api_url: String,
}

#[derive(Serialize)]
struct MfaConfirmArgs {
    #[serde(rename = "mfaSetupToken")]
    mfa_setup_token: String,
    #[serde(rename = "totpCode")]
    totp_code: String,
}

#[derive(Serialize)]
struct MfaVerifyArgs {
    #[serde(rename = "mfaChallengeToken")]
    mfa_challenge_token: String,
    #[serde(rename = "totpCode")]
    totp_code: String,
}

#[derive(Deserialize)]
struct SyncStatus {
    version: i64,
    updated_at: String,
}

#[derive(Deserialize)]
struct MfaSetupInfo {
    mfa_setup_token: String,
    totp_secret_base32: String,
    #[allow(dead_code)]
    totp_uri: String,
    qr_svg: String,
}

#[derive(Deserialize)]
struct MfaChallengeInfo {
    mfa_challenge_token: String,
}

#[derive(Serialize)]
struct SendVerificationArgs {
    email: String,
    #[serde(rename = "apiUrl")]
    api_url: String,
}

#[derive(Serialize)]
struct VerifyCodeArgs {
    email: String,
    code: String,
    #[serde(rename = "apiUrl")]
    api_url: String,
}

#[derive(Serialize)]
struct DeleteAccountArgs {
    #[serde(rename = "totpCode")]
    totp_code: String,
}

#[derive(Clone, Copy, PartialEq)]
enum MfaPhase {
    None,
    EmailVerification,
    Setup,
    Challenge,
}

// ── Component ──

#[component]
pub fn SettingsSync() -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let (connected, set_connected) = signal(false);
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (api_url, set_api_url) = signal("https://api.saladvault.app".to_string());
    let (error_msg, set_error_msg) = signal(String::new());
    let (success_msg, set_success_msg) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (sync_version, set_sync_version) = signal(0i64);
    let (sync_updated, set_sync_updated) = signal(String::new());
    let (show_register, set_show_register) = signal(false);

    // Account deletion state
    let (show_delete_confirm, set_show_delete_confirm) = signal(false);
    let (delete_totp, set_delete_totp) = signal(String::new());
    let (delete_loading, set_delete_loading) = signal(false);

    // Dead Man's Switch state
    let (dm_enabled, set_dm_enabled) = signal(false);
    let (dm_days, set_dm_days) = signal(90u32);
    let (dm_email, set_dm_email) = signal(String::new());
    let (dm_recovery_pwd, set_dm_recovery_pwd) = signal(String::new());
    let (dm_last_seen, set_dm_last_seen) = signal(String::new());
    let (dm_loading, set_dm_loading) = signal(false);

    // MFA state
    let (mfa_phase, set_mfa_phase) = signal(MfaPhase::None);
    let (mfa_setup_token, set_mfa_setup_token) = signal(String::new());
    let (mfa_challenge_token, set_mfa_challenge_token) = signal(String::new());
    let (mfa_qr_svg, set_mfa_qr_svg) = signal(String::new());
    let (mfa_secret_b32, set_mfa_secret_b32) = signal(String::new());
    let (totp_code, set_totp_code) = signal(String::new());

    // Check connection on mount
    {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            if let Ok(result) = invoke("server_is_connected", args).await {
                if let Some(val) = result.as_bool() {
                    set_connected.set(val);
                    if val {
                        load_sync_status(set_sync_version, set_sync_updated).await;
                        load_deadman_status(set_dm_enabled, set_dm_days, set_dm_last_seen).await;
                    }
                }
            }
        });
    }

    // ── Login step 1: credentials → MFA challenge ──
    let handle_login = move |_| {
        set_loading.set(true);
        set_error_msg.set(String::new());
        set_success_msg.set(String::new());
        let e = email.get_untracked();
        let p = password.get_untracked();
        let u = api_url.get_untracked();

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&ServerAuthArgs {
                email: e,
                server_password: p,
                api_url: u,
            })
            .unwrap();
            match invoke("server_login", args).await {
                Ok(result) => {
                    if let Ok(info) = serde_wasm_bindgen::from_value::<MfaChallengeInfo>(result) {
                        set_mfa_challenge_token.set(info.mfa_challenge_token);
                        set_mfa_phase.set(MfaPhase::Challenge);
                        set_totp_code.set(String::new());
                    }
                }
                Err(err) => {
                    set_error_msg.set(
                        err.as_string()
                            .unwrap_or_else(|| t("sync.connection_error", lang.get()).to_string()),
                    );
                }
            }
            set_loading.set(false);
        });
    };

    // ── Login step 2: verify TOTP ──
    let handle_mfa_verify = move |_| {
        set_loading.set(true);
        set_error_msg.set(String::new());
        let token = mfa_challenge_token.get_untracked();
        let code = totp_code.get_untracked();

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&MfaVerifyArgs {
                mfa_challenge_token: token,
                totp_code: code,
            })
            .unwrap();
            match invoke("server_login_verify_mfa", args).await {
                Ok(_) => {
                    set_connected.set(true);
                    set_mfa_phase.set(MfaPhase::None);
                    set_success_msg.set(t("sync.connected", lang.get()).to_string());
                    set_password.set(String::new());
                    set_totp_code.set(String::new());
                    load_sync_status(set_sync_version, set_sync_updated).await;
                }
                Err(err) => {
                    set_error_msg.set(
                        err.as_string()
                            .unwrap_or_else(|| t("sync.invalid_mfa", lang.get()).to_string()),
                    );
                }
            }
            set_loading.set(false);
        });
    };

    // ── Register step 1: send email verification code ──
    let handle_register = move |_| {
        set_loading.set(true);
        set_error_msg.set(String::new());
        set_success_msg.set(String::new());
        let e = email.get_untracked();
        let u = api_url.get_untracked();

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&SendVerificationArgs {
                email: e,
                api_url: u,
            })
            .unwrap();
            match invoke("server_send_verification", args).await {
                Ok(_) => {
                    set_mfa_phase.set(MfaPhase::EmailVerification);
                    set_totp_code.set(String::new());
                }
                Err(err) => {
                    set_error_msg.set(
                        err.as_string()
                            .unwrap_or_else(|| t("sync.register_error", lang.get()).to_string()),
                    );
                }
            }
            set_loading.set(false);
        });
    };

    // ── Register step 1b: verify email code, then register ──
    let handle_email_verify = move |_| {
        set_loading.set(true);
        set_error_msg.set(String::new());
        let e = email.get_untracked();
        let code = totp_code.get_untracked();
        let u = api_url.get_untracked();
        let p = password.get_untracked();

        spawn_local(async move {
            // Step 1: verify the email code
            let verify_args = serde_wasm_bindgen::to_value(&VerifyCodeArgs {
                email: e.clone(),
                code,
                api_url: u.clone(),
            })
            .unwrap();
            match invoke("server_verify_code", verify_args).await {
                Ok(_) => {
                    // Step 2: proceed with actual registration
                    let register_args = serde_wasm_bindgen::to_value(&ServerAuthArgs {
                        email: e,
                        server_password: p,
                        api_url: u,
                    })
                    .unwrap();
                    match invoke("server_register", register_args).await {
                        Ok(result) => {
                            if let Ok(info) = serde_wasm_bindgen::from_value::<MfaSetupInfo>(result) {
                                set_mfa_setup_token.set(info.mfa_setup_token);
                                set_mfa_qr_svg.set(info.qr_svg);
                                set_mfa_secret_b32.set(info.totp_secret_base32);
                                set_mfa_phase.set(MfaPhase::Setup);
                                set_totp_code.set(String::new());
                            }
                        }
                        Err(err) => {
                            set_error_msg.set(
                                err.as_string()
                                    .unwrap_or_else(|| t("sync.register_error", lang.get()).to_string()),
                            );
                        }
                    }
                }
                Err(err) => {
                    set_error_msg.set(
                        err.as_string()
                            .unwrap_or_else(|| t("sync.invalid_mfa", lang.get()).to_string()),
                    );
                }
            }
            set_loading.set(false);
        });
    };

    // ── Account deletion handler ──
    let handle_delete_account = move |_| {
        set_delete_loading.set(true);
        set_error_msg.set(String::new());
        let code = delete_totp.get_untracked();

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&DeleteAccountArgs {
                totp_code: code,
            })
            .unwrap();
            match invoke("server_delete_account", args).await {
                Ok(_) => {
                    set_connected.set(false);
                    set_show_delete_confirm.set(false);
                    set_delete_totp.set(String::new());
                    set_success_msg.set(t("sync.delete_account_success", lang.get()).to_string());
                }
                Err(err) => {
                    set_error_msg.set(
                        err.as_string()
                            .unwrap_or_else(|| t("sync.invalid_mfa", lang.get()).to_string()),
                    );
                }
            }
            set_delete_loading.set(false);
        });
    };

    // ── Register step 2: confirm TOTP ──
    let handle_mfa_confirm = move |_| {
        set_loading.set(true);
        set_error_msg.set(String::new());
        let token = mfa_setup_token.get_untracked();
        let code = totp_code.get_untracked();

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&MfaConfirmArgs {
                mfa_setup_token: token,
                totp_code: code,
            })
            .unwrap();
            match invoke("server_register_confirm_mfa", args).await {
                Ok(_) => {
                    set_connected.set(true);
                    set_mfa_phase.set(MfaPhase::None);
                    set_success_msg.set(t("sync.account_created", lang.get()).to_string());
                    set_password.set(String::new());
                    set_totp_code.set(String::new());
                    set_show_register.set(false);
                }
                Err(err) => {
                    set_error_msg.set(
                        err.as_string()
                            .unwrap_or_else(|| t("sync.invalid_mfa", lang.get()).to_string()),
                    );
                }
            }
            set_loading.set(false);
        });
    };

    let handle_logout = move |_| {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let _ = invoke("server_logout", args).await;
            set_connected.set(false);
            set_success_msg.set(t("sync.disconnected", lang.get()).to_string());
        });
    };

    let handle_push = move |_| {
        set_loading.set(true);
        set_error_msg.set(String::new());
        set_success_msg.set(String::new());
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            match invoke("sync_push", args).await {
                Ok(result) => {
                    if let Ok(status) = serde_wasm_bindgen::from_value::<SyncStatus>(result) {
                        set_sync_version.set(status.version);
                        set_sync_updated.set(status.updated_at);
                    }
                    set_success_msg.set(t("sync.push_success", lang.get()).to_string());
                }
                Err(err) => {
                    set_error_msg.set(
                        err.as_string()
                            .unwrap_or_else(|| t("sync.sync_error", lang.get()).to_string()),
                    );
                }
            }
            set_loading.set(false);
        });
    };

    let handle_pull = move |_| {
        set_loading.set(true);
        set_error_msg.set(String::new());
        set_success_msg.set(String::new());
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            match invoke("sync_pull", args).await {
                Ok(result) => {
                    if let Ok(status) = serde_wasm_bindgen::from_value::<SyncStatus>(result) {
                        set_sync_version.set(status.version);
                        set_sync_updated.set(status.updated_at);
                    }
                    set_success_msg.set(t("sync.pull_success", lang.get()).to_string());
                }
                Err(err) => {
                    set_error_msg.set(
                        err.as_string()
                            .unwrap_or_else(|| t("sync.sync_error", lang.get()).to_string()),
                    );
                }
            }
            set_loading.set(false);
        });
    };

    // ── Dead Man's Switch save ──
    let handle_dm_save = move |_| {
        let enabled = dm_enabled.get_untracked();
        let days = dm_days.get_untracked();
        let email = dm_email.get_untracked();
        let pwd = dm_recovery_pwd.get_untracked();
        set_dm_loading.set(true);
        set_error_msg.set(String::new());
        set_success_msg.set(String::new());

        spawn_local(async move {
            // Step 1: generate recovery kit if a password is provided
            let blob: Option<String> = if !pwd.is_empty() {
                #[derive(Serialize)]
                struct GenArgs {
                    #[serde(rename = "recoveryPassword")]
                    recovery_password: String,
                }
                let args = serde_wasm_bindgen::to_value(&GenArgs { recovery_password: pwd }).unwrap();
                match invoke("generate_recovery_kit", args).await {
                    Ok(val) => val.as_string(),
                    Err(_) => None,
                }
            } else {
                None
            };

            // Step 2: send config to server
            #[derive(Serialize)]
            struct DmArgs {
                enabled: bool,
                days: u32,
                #[serde(rename = "recipientEmail")]
                recipient_email: String,
                #[serde(rename = "recoveryBlob")]
                recovery_blob: Option<String>,
            }
            let args = serde_wasm_bindgen::to_value(&DmArgs {
                enabled,
                days,
                recipient_email: email,
                recovery_blob: blob,
            }).unwrap();
            match invoke("deadman_update_config", args).await {
                Ok(_) => set_success_msg.set(t("general.deadman_saved", lang.get()).to_string()),
                Err(err) => set_error_msg.set(
                    err.as_string().unwrap_or_else(|| t("sync.sync_error", lang.get()).to_string())
                ),
            }
            set_dm_loading.set(false);
        });
    };

    let handle_cancel_mfa = move |_| {
        set_mfa_phase.set(MfaPhase::None);
        set_totp_code.set(String::new());
        set_error_msg.set(String::new());
    };

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("sync.section_title", lang.get())}</h2>
            <p class="settings-section-desc">{move || t("sync.section_desc", lang.get())}</p>

            // Messages
            {move || {
                let err = error_msg.get();
                let ok = success_msg.get();
                if !err.is_empty() {
                    view! { <div class="error-msg">{err}</div> }.into_any()
                } else if !ok.is_empty() {
                    view! { <div class="info-msg">{ok}</div> }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            {move || {
                if connected.get() {
                    // Connected state
                    view! {
                        <div class="settings-group">
                            <h3>{move || t("sync.status", lang.get())}</h3>
                            <div class="settings-row">
                                <label>{move || t("sync.connection", lang.get())}</label>
                                <span class="badge badge-success">{move || t("sync.connected_badge", lang.get())}</span>
                            </div>
                            <div class="settings-row">
                                <label>{move || t("sync.server_version", lang.get())}</label>
                                <span>{move || sync_version.get().to_string()}</span>
                            </div>
                            <div class="settings-row">
                                <label>{move || t("sync.last_sync", lang.get())}</label>
                                <span>{move || {
                                    let u = sync_updated.get();
                                    if u.is_empty() { t("sec.never", lang.get()).to_string() } else { u }
                                }}</span>
                            </div>
                            <div class="import-buttons">
                                <button
                                    class="btn btn-primary btn-sm"
                                    on:click=handle_push
                                    disabled=move || loading.get()
                                >
                                    {move || if loading.get() { t("sync.sending", lang.get()) } else { t("sync.push", lang.get()) }}
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    on:click=handle_pull
                                    disabled=move || loading.get()
                                >
                                    {move || if loading.get() { t("sync.receiving", lang.get()) } else { t("sync.pull", lang.get()) }}
                                </button>
                            </div>
                            <p class="settings-hint settings-hint-warn">{move || t("sync.pull_warning", lang.get())}</p>
                        </div>
                        // ── Dead Man's Switch ──
                        <div class="settings-group">
                            <h3>{move || t("general.deadman_title", lang.get())}</h3>
                            <p class="settings-hint">{move || t("general.deadman_desc", lang.get())}</p>
                            <div class="settings-row">
                                <label>{move || t("general.deadman_enable", lang.get())}</label>
                                <input
                                    type="checkbox"
                                    class="settings-toggle"
                                    prop:checked=move || dm_enabled.get()
                                    on:change=move |ev| set_dm_enabled.set(event_target_checked(&ev))
                                />
                            </div>
                            <div class="settings-row">
                                <label>{move || t("general.deadman_days", lang.get())}</label>
                                <select
                                    class="settings-select"
                                    on:change=move |ev| {
                                        let val: u32 = event_target_value(&ev).parse().unwrap_or(90);
                                        set_dm_days.set(val);
                                    }
                                >
                                    <option value="30" selected=move || dm_days.get() == 30>"30 jours"</option>
                                    <option value="60" selected=move || dm_days.get() == 60>"60 jours"</option>
                                    <option value="90" selected=move || dm_days.get() == 90>"90 jours"</option>
                                    <option value="180" selected=move || dm_days.get() == 180>"180 jours"</option>
                                    <option value="365" selected=move || dm_days.get() == 365>"1 an"</option>
                                </select>
                            </div>
                            <div class="settings-row">
                                <label>{move || t("general.deadman_email", lang.get())}</label>
                                <input
                                    type="email"
                                    class="settings-input"
                                    placeholder="contact@example.com"
                                    prop:value=move || dm_email.get()
                                    on:input=move |ev| set_dm_email.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="settings-row">
                                <label>{move || t("general.kit_password", lang.get())}</label>
                                <input
                                    type="password"
                                    class="settings-input"
                                    placeholder=move || t("general.kit_password_placeholder", lang.get())
                                    prop:value=move || dm_recovery_pwd.get()
                                    on:input=move |ev| set_dm_recovery_pwd.set(event_target_value(&ev))
                                />
                            </div>
                            <p class="settings-hint">{move || t("general.deadman_recovery_hint", lang.get())}</p>
                            {move || {
                                let ls = dm_last_seen.get();
                                if !ls.is_empty() {
                                    view! {
                                        <div class="settings-row">
                                            <label>{move || t("general.deadman_last_seen", lang.get())}</label>
                                            <span class="settings-value-muted">{ls}</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }
                            }}
                            <button
                                class="btn btn-primary btn-sm"
                                on:click=handle_dm_save
                                disabled=move || dm_loading.get()
                            >
                                {move || if dm_loading.get() {
                                    t("general.deadman_saving", lang.get())
                                } else {
                                    t("general.deadman_save", lang.get())
                                }}
                            </button>
                        </div>
                        <div class="settings-group">
                            <button class="btn btn-ghost btn-danger btn-sm" on:click=handle_logout>
                                {move || t("sync.logout", lang.get())}
                            </button>
                        </div>

                        // Danger zone — account deletion
                        <div class="settings-group settings-danger-zone">
                            <h3>{move || t("sync.danger_zone", lang.get())}</h3>
                            {move || {
                                if show_delete_confirm.get() {
                                    view! {
                                        <p class="settings-hint settings-warning">
                                            {move || t("sync.delete_account_warning", lang.get())}
                                        </p>
                                        <label class="settings-label">{move || t("sync.delete_account_mfa", lang.get())}</label>
                                        <input
                                            type="text"
                                            class="settings-input"
                                            maxlength="6"
                                            placeholder="000000"
                                            prop:value=move || delete_totp.get()
                                            on:input=move |ev| set_delete_totp.set(event_target_value(&ev))
                                        />
                                        <div class="settings-row" style="gap: 0.5rem; margin-top: 0.5rem;">
                                            <button
                                                class="btn btn-ghost btn-danger btn-sm"
                                                disabled=move || delete_loading.get() || delete_totp.get().len() != 6
                                                on:click=handle_delete_account
                                            >
                                                {move || if delete_loading.get() { t("sync.deleting_account", lang.get()).to_string() } else { t("sync.delete_account", lang.get()).to_string() }}
                                            </button>
                                            <button
                                                class="btn btn-ghost btn-sm"
                                                on:click=move |_| {
                                                    set_show_delete_confirm.set(false);
                                                    set_delete_totp.set(String::new());
                                                }
                                            >
                                                {move || t("sync.cancel", lang.get())}
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button
                                            class="btn btn-ghost btn-danger btn-sm"
                                            on:click=move |_| set_show_delete_confirm.set(true)
                                        >
                                            {move || t("sync.delete_account", lang.get())}
                                        </button>
                                    }.into_any()
                                }
                            }}
                        </div>
                    }.into_any()
                } else if mfa_phase.get() == MfaPhase::EmailVerification {
                    // Email verification phase (registration step 1b)
                    view! {
                        <div class="settings-group">
                            <h3>{move || t("sync.email_verification_title", lang.get())}</h3>
                            <p class="settings-hint">{move || t("sync.email_verification_hint", lang.get())}</p>
                            <label class="settings-label">{move || t("sync.email_verification_code", lang.get())}</label>
                            <input
                                type="text"
                                class="settings-input"
                                maxlength="6"
                                placeholder="000000"
                                prop:value=move || totp_code.get()
                                on:input=move |ev| set_totp_code.set(event_target_value(&ev))
                            />
                            <div class="settings-row" style="gap: 0.5rem; margin-top: 0.5rem;">
                                <button
                                    class="btn btn-primary btn-sm"
                                    disabled=move || loading.get() || totp_code.get().len() != 6
                                    on:click=handle_email_verify
                                >
                                    {move || if loading.get() { t("sync.email_verifying", lang.get()).to_string() } else { t("sync.email_verify_btn", lang.get()).to_string() }}
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    on:click=move |_| {
                                        set_mfa_phase.set(MfaPhase::None);
                                        set_totp_code.set(String::new());
                                    }
                                >
                                    {move || t("sync.cancel", lang.get())}
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else if mfa_phase.get() == MfaPhase::Setup {
                    // MFA Setup phase (registration step 2)
                    view! {
                        <div class="settings-group">
                            <h3>{move || t("sync.mfa_setup_title", lang.get())}</h3>
                            <p class="settings-hint">{move || t("sync.mfa_setup_hint", lang.get())}</p>
                            <div class="settings-row">
                                <label>{move || t("sync.manual_key", lang.get())}</label>
                                <code class="mfa-secret-code">{move || mfa_secret_b32.get()}</code>
                            </div>
                            <div class="settings-row">
                                <label>{move || t("sync.verification_code", lang.get())}</label>
                                <input
                                    type="text"
                                    class="settings-input mfa-code-input"
                                    placeholder="000000"
                                    maxlength="6"
                                    inputmode="numeric"
                                    autocomplete="one-time-code"
                                    prop:value=move || totp_code.get()
                                    on:input=move |ev| set_totp_code.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="import-buttons">
                                <button
                                    class="btn btn-primary btn-sm"
                                    on:click=handle_mfa_confirm
                                    disabled=move || loading.get() || totp_code.get().len() != 6
                                >
                                    {move || if loading.get() { t("sync.verifying", lang.get()) } else { t("sync.confirm_code", lang.get()) }}
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    on:click=handle_cancel_mfa
                                >
                                    {move || t("cancel", lang.get())}
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else if mfa_phase.get() == MfaPhase::Challenge {
                    // MFA Challenge phase (login step 2)
                    view! {
                        <div class="settings-group">
                            <h3>{move || t("sync.mfa_verify_title", lang.get())}</h3>
                            <p class="settings-hint">{move || t("sync.mfa_verify_hint", lang.get())}</p>
                            <div class="settings-row">
                                <label>{move || t("sync.mfa_code", lang.get())}</label>
                                <input
                                    type="text"
                                    class="settings-input mfa-code-input"
                                    placeholder="000000"
                                    maxlength="6"
                                    inputmode="numeric"
                                    autocomplete="one-time-code"
                                    prop:value=move || totp_code.get()
                                    on:input=move |ev| set_totp_code.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="import-buttons">
                                <button
                                    class="btn btn-primary btn-sm"
                                    on:click=handle_mfa_verify
                                    disabled=move || loading.get() || totp_code.get().len() != 6
                                >
                                    {move || if loading.get() { t("sync.verifying", lang.get()) } else { t("sync.verify", lang.get()) }}
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    on:click=handle_cancel_mfa
                                >
                                    {move || t("cancel", lang.get())}
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    // Disconnected state — login/register form
                    view! {
                        <div class="settings-group">
                            <h3>{move || t("sync.login_title", lang.get())}</h3>
                            <div class="settings-row">
                                <label>{move || t("sync.server_url", lang.get())}</label>
                                <input
                                    type="url"
                                    class="settings-input"
                                    placeholder="https://api.saladvault.app"
                                    prop:value=move || api_url.get()
                                    on:input=move |ev| set_api_url.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="settings-row">
                                <label>{move || t("email", lang.get())}</label>
                                <input
                                    type="email"
                                    class="settings-input"
                                    placeholder="votre@email.com"
                                    prop:value=move || email.get()
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="settings-row">
                                <label>{move || t("sync.server_password", lang.get())}</label>
                                <input
                                    type="password"
                                    class="settings-input"
                                    placeholder=move || t("sync.server_password_placeholder", lang.get())
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                />
                            </div>
                            <p class="settings-hint">{move || t("sync.password_hint", lang.get())}</p>
                            <div class="import-buttons">
                                <button
                                    class="btn btn-primary btn-sm"
                                    on:click=handle_login
                                    disabled=move || loading.get()
                                >
                                    {move || if loading.get() { t("sync.connecting", lang.get()) } else { t("sync.login", lang.get()) }}
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    on:click=move |_| set_show_register.set(!show_register.get_untracked())
                                >
                                    {move || t("sync.create_account", lang.get())}
                                </button>
                            </div>
                            {move || {
                                if show_register.get() {
                                    view! {
                                        <div class="settings-note">
                                            <p>{move || t("sync.create_account_hint", lang.get())}</p>
                                            <button
                                                class="btn btn-primary btn-sm"
                                                on:click=handle_register
                                                disabled=move || loading.get()
                                            >
                                                {move || if loading.get() { t("sync.registering", lang.get()) } else { t("sync.register", lang.get()) }}
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }
                            }}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[derive(Deserialize)]
struct DeadmanStatus {
    enabled: bool,
    inactivity_days: u32,
    last_seen_at: String,
}

async fn load_deadman_status(
    set_enabled: WriteSignal<bool>,
    set_days: WriteSignal<u32>,
    set_last_seen: WriteSignal<String>,
) {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    if let Ok(result) = invoke("deadman_status", args).await {
        if let Ok(status) = serde_wasm_bindgen::from_value::<DeadmanStatus>(result) {
            set_enabled.set(status.enabled);
            set_days.set(status.inactivity_days);
            set_last_seen.set(status.last_seen_at);
        }
    }
}

async fn load_sync_status(
    set_version: WriteSignal<i64>,
    set_updated: WriteSignal<String>,
) {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    if let Ok(result) = invoke("sync_status", args).await {
        if let Ok(status) = serde_wasm_bindgen::from_value::<SyncStatus>(result) {
            set_version.set(status.version);
            set_updated.set(status.updated_at);
        }
    }
}
