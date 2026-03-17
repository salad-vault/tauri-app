use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};
use crate::components::settings::{PasswordType, Theme, UserSettings};

/// Apply theme by setting data-theme attribute on <html>.
fn apply_theme(theme: &Theme) {
    let theme_str = match theme {
        Theme::Dark => "dark",
        Theme::Light => "light",
    };
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-theme", theme_str);
            }
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn SettingsGeneral(
    settings: ReadSignal<UserSettings>,
    set_settings: WriteSignal<UserSettings>,
    on_save: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let save = on_save.clone();
    let (preview_password, set_preview_password) = signal(String::new());
    let (gen_loading, set_gen_loading) = signal(false);
    // Recovery kit state
    let (recovery_password, set_recovery_password) = signal(String::new());
    let (recovery_loading, set_recovery_loading) = signal(false);
    let (recovery_msg, set_recovery_msg) = signal(String::new());
    let (recovery_error, set_recovery_error) = signal(String::new());

    let generate_preview = move |_| {
        set_gen_loading.set(true);
        let s = settings.get_untracked();
        spawn_local(async move {
            #[derive(Serialize)]
            struct GenArgs {
                length: u32,
                #[serde(rename = "passwordType")]
                password_type: String,
            }
            let ptype = match s.password_type {
                PasswordType::Alphanumeric => "alphanumeric",
                PasswordType::Passphrase => "passphrase",
            };
            let args = serde_wasm_bindgen::to_value(&GenArgs {
                length: s.password_default_length,
                password_type: ptype.to_string(),
            })
            .unwrap();
            if let Ok(result) = invoke("generate_password", args).await {
                if let Some(pwd) = result.as_string() {
                    set_preview_password.set(pwd);
                }
            }
            set_gen_loading.set(false);
        });
    };

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("general.section_title", lang.get())}</h2>
            <p class="settings-section-desc">{move || t("general.section_desc", lang.get())}</p>

            // Password generator defaults
            <div class="settings-group">
                <h3>{move || t("general.password_gen", lang.get())}</h3>
                <div class="settings-row">
                    <label>{move || t("general.default_length", lang.get())}</label>
                    <div class="slider-container">
                        <input
                            type="range"
                            min="12"
                            max="64"
                            class="settings-slider"
                            prop:value=move || settings.get().password_default_length.to_string()
                            on:input={
                                let save = save.clone();
                                move |ev| {
                                    let val: u32 = event_target_value(&ev).parse().unwrap_or(20);
                                    let mut s = settings.get_untracked();
                                    s.password_default_length = val;
                                    set_settings.set(s);
                                    save();
                                }
                            }
                        />
                        <span class="slider-value">{move || settings.get().password_default_length.to_string()}</span>
                    </div>
                </div>

                <div class="settings-row">
                    <label>{move || t("general.password_type", lang.get())}</label>
                    <div class="settings-radio-group-inline">
                        <label class="settings-radio">
                            <input
                                type="radio"
                                name="password-type"
                                checked=move || settings.get().password_type == PasswordType::Alphanumeric
                                on:change={
                                    let save = save.clone();
                                    move |_| {
                                        let mut s = settings.get_untracked();
                                        s.password_type = PasswordType::Alphanumeric;
                                        set_settings.set(s);
                                        save();
                                    }
                                }
                            />
                            <span>{move || t("general.alphanumeric", lang.get())}</span>
                        </label>
                        <label class="settings-radio">
                            <input
                                type="radio"
                                name="password-type"
                                checked=move || settings.get().password_type == PasswordType::Passphrase
                                on:change={
                                    let save = save.clone();
                                    move |_| {
                                        let mut s = settings.get_untracked();
                                        s.password_type = PasswordType::Passphrase;
                                        set_settings.set(s);
                                        save();
                                    }
                                }
                            />
                            <span>{move || t("general.passphrase", lang.get())}</span>
                        </label>
                    </div>
                </div>

                <div class="settings-row">
                    <button class="btn btn-ghost btn-sm" on:click=generate_preview disabled=move || gen_loading.get()>
                        {move || if gen_loading.get() { t("general.generating", lang.get()) } else { t("general.generate_preview", lang.get()) }}
                    </button>
                    {move || {
                        let pwd = preview_password.get();
                        if pwd.is_empty() {
                            view! { <div></div> }.into_any()
                        } else {
                            view! {
                                <code class="password-preview">{pwd}</code>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            // Theme
            <div class="settings-group">
                <h3>{move || t("general.theme", lang.get())}</h3>
                <div class="settings-radio-group-inline">
                    <label class="settings-radio">
                        <input
                            type="radio"
                            name="theme"
                            checked=move || settings.get().theme == Theme::Dark
                            on:change={
                                let save = save.clone();
                                move |_| {
                                    let mut s = settings.get_untracked();
                                    s.theme = Theme::Dark;
                                    apply_theme(&s.theme);
                                    set_settings.set(s);
                                    save();
                                }
                            }
                        />
                        <span>{move || t("general.theme_dark", lang.get())}</span>
                    </label>
                    <label class="settings-radio">
                        <input
                            type="radio"
                            name="theme"
                            checked=move || settings.get().theme == Theme::Light
                            on:change={
                                let save = save.clone();
                                move |_| {
                                    let mut s = settings.get_untracked();
                                    s.theme = Theme::Light;
                                    apply_theme(&s.theme);
                                    set_settings.set(s);
                                    save();
                                }
                            }
                        />
                        <span>{move || t("general.theme_light", lang.get())}</span>
                    </label>
                </div>
            </div>

            // Dead Man's Switch
            <div class="settings-group dead-man-switch">
                <h3>{move || t("general.deadman_title", lang.get())}</h3>
                <p class="settings-hint">{move || t("general.deadman_desc", lang.get())}</p>

                <div class="settings-row">
                    <label>{move || t("general.deadman_enable", lang.get())}</label>
                    <input
                        type="checkbox"
                        class="settings-toggle"
                        prop:checked=move || settings.get().dead_man_switch_enabled
                        on:change={
                            let save = save.clone();
                            move |ev| {
                                let checked = event_target_checked(&ev);
                                let mut s = settings.get_untracked();
                                s.dead_man_switch_enabled = checked;
                                set_settings.set(s.clone());
                                save();
                                sync_deadman_to_server(&s);
                            }
                        }
                    />
                </div>
                {move || {
                    if !settings.get().dead_man_switch_enabled {
                        view! {
                            <p class="settings-hint settings-hint-warn">{move || t("general.deadman_disabled_warn", lang.get())}</p>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}

                {move || {
                    if settings.get().dead_man_switch_enabled {
                        let save_days = save.clone();
                        let save_email = save.clone();
                        view! {
                            <div class="dead-man-details">
                                <div class="settings-row">
                                    <label>{move || t("general.deadman_days", lang.get())}</label>
                                    <input
                                        type="number"
                                        class="settings-input-sm"
                                        min="7"
                                        max="365"
                                        prop:value=move || settings.get().dead_man_switch_days.to_string()
                                        on:change=move |ev| {
                                            let val: u32 = event_target_value(&ev).parse().unwrap_or(90);
                                            let mut s = settings.get_untracked();
                                            s.dead_man_switch_days = val.max(7).min(365);
                                            set_settings.set(s.clone());
                                            save_days();
                                            sync_deadman_to_server(&s);
                                        }
                                    />
                                </div>
                                <div class="settings-row">
                                    <label>{move || t("general.deadman_email", lang.get())}</label>
                                    <input
                                        type="email"
                                        class="settings-input"
                                        placeholder="contact@example.com"
                                        prop:value=move || settings.get().dead_man_switch_email.clone()
                                        on:change=move |ev| {
                                            let val = event_target_value(&ev);
                                            let mut s = settings.get_untracked();
                                            s.dead_man_switch_email = val;
                                            set_settings.set(s.clone());
                                            save_email();
                                            sync_deadman_to_server(&s);
                                        }
                                    />
                                </div>
                                // Recovery Kit section
                                <div class="settings-subgroup">
                                    <h4>{move || t("general.recovery_kit_title", lang.get())}</h4>
                                    <p class="settings-hint">{move || t("general.recovery_kit_desc", lang.get())}</p>
                                    <div class="settings-row">
                                        <label>{move || t("general.kit_password", lang.get())}</label>
                                        <input
                                            type="password"
                                            class="settings-input"
                                            placeholder=move || t("general.kit_password_placeholder", lang.get())
                                            prop:value=move || recovery_password.get()
                                            on:input=move |ev| set_recovery_password.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <p class="settings-hint">{move || t("general.kit_password_hint", lang.get())}</p>
                                    <button
                                        class="btn btn-primary btn-sm"
                                        disabled=move || recovery_loading.get() || recovery_password.get().len() < 8
                                        on:click={
                                            move |_| {
                                                set_recovery_loading.set(true);
                                                set_recovery_msg.set(String::new());
                                                set_recovery_error.set(String::new());
                                                let pwd = recovery_password.get_untracked();
                                                let s = settings.get_untracked();
                                                spawn_local(async move {
                                                    // 1. Generate the recovery blob
                                                    #[derive(Serialize)]
                                                    struct RecoveryArgs {
                                                        #[serde(rename = "recoveryPassword")]
                                                        recovery_password: String,
                                                    }
                                                    let gen_args = serde_wasm_bindgen::to_value(&RecoveryArgs {
                                                        recovery_password: pwd,
                                                    }).unwrap();
                                                    match invoke("generate_recovery_kit", gen_args).await {
                                                        Ok(blob_js) => {
                                                            let blob = blob_js.as_string().unwrap_or_default();
                                                            // 2. Upload to server via deadman_update_config
                                                            #[derive(Serialize)]
                                                            struct DeadmanArgs {
                                                                enabled: bool,
                                                                days: u32,
                                                                #[serde(rename = "recipientEmail")]
                                                                recipient_email: String,
                                                                #[serde(rename = "recoveryBlob")]
                                                                recovery_blob: Option<String>,
                                                            }
                                                            let upload_args = serde_wasm_bindgen::to_value(&DeadmanArgs {
                                                                enabled: s.dead_man_switch_enabled,
                                                                days: s.dead_man_switch_days,
                                                                recipient_email: s.dead_man_switch_email.clone(),
                                                                recovery_blob: Some(blob),
                                                            }).unwrap();
                                                            match invoke("deadman_update_config", upload_args).await {
                                                                Ok(_) => {
                                                                    set_recovery_msg.set(t("general.kit_sent_success", lang.get()).to_string());
                                                                }
                                                                Err(e) => {
                                                                    set_recovery_error.set(
                                                                        e.as_string().unwrap_or_else(|| t("general.kit_send_error", lang.get()).to_string())
                                                                    );
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            set_recovery_error.set(
                                                                e.as_string().unwrap_or_else(|| t("general.kit_gen_error", lang.get()).to_string())
                                                            );
                                                        }
                                                    }
                                                    set_recovery_loading.set(false);
                                                });
                                            }
                                        }
                                    >
                                        {move || if recovery_loading.get() { t("general.generating", lang.get()) } else { t("general.generate_send_kit", lang.get()) }}
                                    </button>
                                    {move || {
                                        let msg = recovery_msg.get();
                                        let err = recovery_error.get();
                                        if !msg.is_empty() {
                                            view! { <p class="info-msg">{msg}</p> }.into_any()
                                        } else if !err.is_empty() {
                                            view! { <p class="error-msg">{err}</p> }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    }}
                                </div>

                                <div class="settings-note">
                                    <span class="note-icon">"ℹ️"</span>
                                    <p>{move || t("general.settings_sync_note", lang.get())}</p>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Sync Dead Man's Switch settings to the server (fire-and-forget).
/// Silently ignored if not connected to the server.
fn sync_deadman_to_server(settings: &UserSettings) {
    #[derive(Serialize)]
    struct DeadmanArgs {
        enabled: bool,
        days: u32,
        #[serde(rename = "recipientEmail")]
        recipient_email: String,
        #[serde(rename = "recoveryBlob")]
        recovery_blob: Option<String>,
    }
    let args = serde_wasm_bindgen::to_value(&DeadmanArgs {
        enabled: settings.dead_man_switch_enabled,
        days: settings.dead_man_switch_days,
        recipient_email: settings.dead_man_switch_email.clone(),
        recovery_blob: None,
    })
    .unwrap();
    spawn_local(async move {
        let _ = invoke("deadman_update_config", args).await;
    });
}
