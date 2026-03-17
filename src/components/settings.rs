use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};
use crate::components::settings_security::SettingsSecurity;
use crate::components::settings_keys::SettingsKeys;
use crate::components::settings_devices::SettingsDevices;
use crate::components::settings_saladiers::SettingsSaladiers;
use crate::components::settings_data::SettingsData;
use crate::components::settings_privacy::SettingsPrivacy;
use crate::components::settings_general::SettingsGeneral;
use crate::components::settings_subscription::SettingsSubscription;
use crate::components::settings_sync::SettingsSync;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// All setting enums must match the backend exactly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutoLockTimeout {
    Immediate,
    After1Min,
    After5Min,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PasswordType {
    Alphanumeric,
    Passphrase,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FaviconPolicy {
    None,
    ProxyAnonymous,
    Direct,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    #[serde(alias = "System")]
    Dark,
    Light,
}

/// Mirror of the backend UserSettings struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub auto_lock_timeout: AutoLockTimeout,
    pub auto_lock_on_sleep: bool,
    pub auto_lock_on_close: bool,
    pub auto_lock_on_inactivity: bool,
    pub clipboard_clear_seconds: u32,
    pub screenshot_protection: bool,
    pub password_default_length: u32,
    pub password_type: PasswordType,
    pub favicon_policy: FaviconPolicy,
    pub crash_reports: bool,
    pub max_failed_attempts: u32,
    pub theme: Theme,
    pub dead_man_switch_enabled: bool,
    pub dead_man_switch_days: u32,
    pub dead_man_switch_email: String,
    pub clear_icon_cache_on_close: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            auto_lock_timeout: AutoLockTimeout::After5Min,
            auto_lock_on_sleep: true,
            auto_lock_on_close: true,
            auto_lock_on_inactivity: true,
            clipboard_clear_seconds: 30,
            screenshot_protection: true,
            password_default_length: 20,
            password_type: PasswordType::Alphanumeric,
            favicon_policy: FaviconPolicy::None,
            crash_reports: false,
            max_failed_attempts: 0,
            theme: Theme::Dark,
            dead_man_switch_enabled: false,
            dead_man_switch_days: 90,
            dead_man_switch_email: String::new(),
            clear_icon_cache_on_close: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum SettingsTab {
    Security,
    Keys,
    Devices,
    Saladiers,
    Data,
    Privacy,
    General,
    Sync,
    Subscription,
}

#[component]
pub fn Settings(
    on_back: Callback<()>,
) -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let (active_tab, set_active_tab) = signal(SettingsTab::Security);
    let (settings, set_settings) = signal(UserSettings::default());
    let (loading, set_loading) = signal(true);
    let (save_msg, set_save_msg) = signal(String::new());

    // Load settings on mount
    {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let result = invoke("get_settings", args).await;
            if let Ok(s) = serde_wasm_bindgen::from_value::<UserSettings>(result) {
                set_settings.set(s);
            }
            set_loading.set(false);
        });
    }

    // Save settings helper
    let save_settings = move || {
        let current = settings.get_untracked();
        spawn_local(async move {
            #[derive(Serialize)]
            struct SaveArgs {
                settings: UserSettings,
            }
            let args = serde_wasm_bindgen::to_value(&SaveArgs { settings: current }).unwrap();
            let result = invoke("save_settings", args).await;
            if result.is_null() || result.is_undefined() || result.as_string().map(|s| s.is_empty()).unwrap_or(true) {
                set_save_msg.set(t("settings.saved", lang.get()).to_string());
            } else {
                set_save_msg.set(t("settings.save_error", lang.get()).to_string());
            }
            // Clear message after 2 seconds
            gloo_timers::callback::Timeout::new(2_000, move || {
                set_save_msg.set(String::new());
            })
            .forget();
        });
    };

    view! {
        <div class="settings-page">
            <header class="settings-header">
                <div class="header-left">
                    <button class="btn btn-ghost" on:click=move |_| on_back.run(())>
                        {move || t("back", lang.get())}
                    </button>
                    <h1>{move || t("settings.title", lang.get())}</h1>
                </div>
                {move || {
                    let msg = save_msg.get();
                    if msg.is_empty() {
                        view! { <div></div> }.into_any()
                    } else {
                        view! {
                            <div class="settings-save-msg">{msg}</div>
                        }.into_any()
                    }
                }}
            </header>

            <div class="settings-layout">
                <nav class="settings-nav">
                    <button
                        class=move || if active_tab.get() == SettingsTab::Security { "settings-nav-item active" } else { "settings-nav-item" }
                        on:click=move |_| set_active_tab.set(SettingsTab::Security)
                    >
                        <span class="nav-icon">"🛡️"</span>
                        <span>{move || t("settings.tab_security", lang.get())}</span>
                    </button>
                    <button
                        class=move || if active_tab.get() == SettingsTab::Keys { "settings-nav-item active" } else { "settings-nav-item" }
                        on:click=move |_| set_active_tab.set(SettingsTab::Keys)
                    >
                        <span class="nav-icon">"🔑"</span>
                        <span>{move || t("settings.tab_keys", lang.get())}</span>
                    </button>
                    <button
                        class=move || if active_tab.get() == SettingsTab::Devices { "settings-nav-item active" } else { "settings-nav-item" }
                        on:click=move |_| set_active_tab.set(SettingsTab::Devices)
                    >
                        <span class="nav-icon">"📱"</span>
                        <span>{move || t("settings.tab_devices", lang.get())}</span>
                    </button>
                    <button
                        class=move || if active_tab.get() == SettingsTab::Saladiers { "settings-nav-item active" } else { "settings-nav-item" }
                        on:click=move |_| set_active_tab.set(SettingsTab::Saladiers)
                    >
                        <span class="nav-icon">"🥗"</span>
                        <span>{move || t("settings.tab_saladiers", lang.get())}</span>
                    </button>
                    <button
                        class=move || if active_tab.get() == SettingsTab::Data { "settings-nav-item active" } else { "settings-nav-item" }
                        on:click=move |_| set_active_tab.set(SettingsTab::Data)
                    >
                        <span class="nav-icon">"💾"</span>
                        <span>{move || t("settings.tab_data", lang.get())}</span>
                    </button>
                    <button
                        class=move || if active_tab.get() == SettingsTab::Privacy { "settings-nav-item active" } else { "settings-nav-item" }
                        on:click=move |_| set_active_tab.set(SettingsTab::Privacy)
                    >
                        <span class="nav-icon">"🌐"</span>
                        <span>{move || t("settings.tab_privacy", lang.get())}</span>
                    </button>
                    <button
                        class=move || if active_tab.get() == SettingsTab::General { "settings-nav-item active" } else { "settings-nav-item" }
                        on:click=move |_| set_active_tab.set(SettingsTab::General)
                    >
                        <span class="nav-icon">"⚙️"</span>
                        <span>{move || t("settings.tab_general", lang.get())}</span>
                    </button>
                    <button
                        class=move || if active_tab.get() == SettingsTab::Sync { "settings-nav-item active" } else { "settings-nav-item" }
                        on:click=move |_| set_active_tab.set(SettingsTab::Sync)
                    >
                        <span class="nav-icon">"☁️"</span>
                        <span>{move || t("sync.title", lang.get())}</span>
                    </button>
                    <button
                        class=move || if active_tab.get() == SettingsTab::Subscription { "settings-nav-item active" } else { "settings-nav-item" }
                        on:click=move |_| set_active_tab.set(SettingsTab::Subscription)
                    >
                        <span class="nav-icon">"💎"</span>
                        <span>{move || t("settings.tab_subscription", lang.get())}</span>
                    </button>
                </nav>

                <div class="settings-content">
                    {move || {
                        if loading.get() {
                            return view! { <div class="settings-loading">{move || t("loading", lang.get())}</div> }.into_any();
                        }

                        let save = save_settings.clone();

                        match active_tab.get() {
                            SettingsTab::Security => {
                                view! {
                                    <SettingsSecurity settings=settings set_settings=set_settings on_save=save />
                                }.into_any()
                            }
                            SettingsTab::Keys => {
                                view! {
                                    <SettingsKeys />
                                }.into_any()
                            }
                            SettingsTab::Devices => {
                                view! {
                                    <SettingsDevices />
                                }.into_any()
                            }
                            SettingsTab::Saladiers => {
                                view! {
                                    <SettingsSaladiers settings=settings set_settings=set_settings on_save=save />
                                }.into_any()
                            }
                            SettingsTab::Data => {
                                view! {
                                    <SettingsData />
                                }.into_any()
                            }
                            SettingsTab::Privacy => {
                                view! {
                                    <SettingsPrivacy settings=settings set_settings=set_settings on_save=save />
                                }.into_any()
                            }
                            SettingsTab::General => {
                                view! {
                                    <SettingsGeneral settings=settings set_settings=set_settings on_save=save />
                                }.into_any()
                            }
                            SettingsTab::Sync => {
                                view! {
                                    <SettingsSync />
                                }.into_any()
                            }
                            SettingsTab::Subscription => {
                                view! {
                                    <SettingsSubscription />
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
