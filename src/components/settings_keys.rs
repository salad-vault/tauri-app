use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "dialog"], catch)]
    async fn save(options: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
struct VerifyPasswordArgs {
    #[serde(rename = "masterPassword")]
    master_password: String,
}

#[component]
pub fn SettingsKeys() -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();

    let (key_path, set_key_path) = signal(String::new());
    let (show_kit, set_show_kit) = signal(false);
    let (kit_password, set_kit_password) = signal(String::new());
    let (kit_error, set_kit_error) = signal(String::new());
    let (kit_phrase, set_kit_phrase) = signal(Vec::<String>::new());
    let (kit_loading, set_kit_loading) = signal(false);

    // Change master password state
    let (show_change_pwd, set_show_change_pwd) = signal(false);
    let (current_pwd, set_current_pwd) = signal(String::new());
    let (new_pwd, set_new_pwd) = signal(String::new());
    let (change_pwd_error, set_change_pwd_error) = signal(String::new());
    let (change_pwd_loading, set_change_pwd_loading) = signal(false);
    let (change_pwd_success, set_change_pwd_success) = signal(false);

    // Regenerate key state
    let (show_regen, set_show_regen) = signal(false);
    let (regen_pwd, set_regen_pwd) = signal(String::new());
    let (regen_error, set_regen_error) = signal(String::new());
    let (regen_loading, set_regen_loading) = signal(false);
    let (regen_success, set_regen_success) = signal(false);

    // Load device key path
    {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            if let Ok(result) = invoke("get_device_key_path", args).await {
                if let Some(path) = result.as_string() {
                    set_key_path.set(path);
                }
            }
        });
    }

    let handle_kit_verify = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_kit_loading.set(true);
        set_kit_error.set(String::new());

        let pwd = kit_password.get_untracked();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&VerifyPasswordArgs {
                master_password: pwd,
            })
            .unwrap();
            match invoke("verify_master_password", args).await {
                Ok(_) => {
                    // Password verified, get recovery phrase
                    let args2 = serde_wasm_bindgen::to_value(&()).unwrap();
                    if let Ok(phrase_result) = invoke("generate_recovery_phrase", args2).await {
                        if let Ok(words) = serde_wasm_bindgen::from_value::<Vec<String>>(phrase_result) {
                            set_kit_phrase.set(words);
                        } else {
                            set_kit_error.set(t("keys.error_generating_phrase", lang.get()).to_string());
                        }
                    } else {
                        set_kit_error.set(t("keys.error_generating_phrase", lang.get()).to_string());
                    }
                }
                Err(_) => {
                    set_kit_error.set(t("keys.wrong_password", lang.get()).to_string());
                }
            }
            set_kit_loading.set(false);
        });
    };

    let handle_change_pwd = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_change_pwd_loading.set(true);
        set_change_pwd_error.set(String::new());
        set_change_pwd_success.set(false);

        let curr = current_pwd.get_untracked();
        let new = new_pwd.get_untracked();

        // Validate new password
        if let Err(msg) = crate::components::password_utils::validate_password(&new, lang.get_untracked()) {
            set_change_pwd_error.set(msg);
            set_change_pwd_loading.set(false);
            return;
        }

        spawn_local(async move {
            #[derive(Serialize)]
            struct Args {
                #[serde(rename = "currentPassword")]
                current_password: String,
                #[serde(rename = "newPassword")]
                new_password: String,
            }
            let args = serde_wasm_bindgen::to_value(&Args {
                current_password: curr,
                new_password: new,
            })
            .unwrap();
            set_change_pwd_loading.set(false);
            match invoke("change_master_password", args).await {
                Ok(_) => {
                    set_change_pwd_success.set(true);
                    set_show_change_pwd.set(false);
                    set_current_pwd.set(String::new());
                    set_new_pwd.set(String::new());
                }
                Err(_) => {
                    set_change_pwd_error.set(t("keys.wrong_current_password", lang.get()).to_string());
                }
            }
        });
    };

    let handle_regen_key = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_regen_loading.set(true);
        set_regen_error.set(String::new());
        set_regen_success.set(false);

        let pwd = regen_pwd.get_untracked();
        spawn_local(async move {
            #[derive(Serialize)]
            struct Args {
                #[serde(rename = "masterPassword")]
                master_password: String,
            }
            let args = serde_wasm_bindgen::to_value(&Args {
                master_password: pwd,
            })
            .unwrap();
            set_regen_loading.set(false);
            match invoke("regenerate_device_key", args).await {
                Ok(_) => {
                    set_regen_success.set(true);
                    set_show_regen.set(false);
                    set_regen_pwd.set(String::new());
                }
                Err(_) => {
                    set_regen_error.set(t("keys.wrong_password", lang.get()).to_string());
                }
            }
        });
    };

    let (move_error, set_move_error) = signal(String::new());

    let handle_move_key = move |_| {
        set_move_error.set(String::new());
        spawn_local(async move {
            // Open a save dialog to choose the new location
            let opts = js_sys::Object::new();
            js_sys::Reflect::set(&opts, &"defaultPath".into(), &"device_secret.key".into()).unwrap();
            js_sys::Reflect::set(&opts, &"title".into(), &t("keys.choose_key_location", lang.get()).into()).unwrap();

            match save(opts.into()).await {
                Ok(path_val) => {
                    if let Some(path) = path_val.as_string() {
                        if path.is_empty() {
                            return; // User cancelled
                        }
                        #[derive(Serialize)]
                        struct MoveArgs {
                            #[serde(rename = "newPath")]
                            new_path: String,
                        }
                        let args = serde_wasm_bindgen::to_value(&MoveArgs { new_path: path }).unwrap();
                        match invoke("move_device_key", args).await {
                            Ok(result) => {
                                if let Some(new_path) = result.as_string() {
                                    set_key_path.set(new_path);
                                }
                            }
                            Err(err) => {
                                let msg = err.as_string().unwrap_or_else(|| t("keys.move_error", lang.get()).to_string());
                                set_move_error.set(msg);
                            }
                        }
                    }
                    // path_val is null/undefined = user cancelled
                }
                Err(_) => {
                    // Dialog cancelled or error
                }
            }
        });
    };

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("keys.section_title", lang.get())}</h2>
            <p class="settings-section-desc">{move || t("keys.section_desc", lang.get())}</p>

            // Device key path
            <div class="settings-group">
                <h3>{move || t("keys.local_key_path", lang.get())}</h3>
                <div class="key-path-display">
                    <code class="key-path-text">{move || key_path.get()}</code>
                </div>
                <div class="settings-actions">
                    <button class="btn btn-ghost btn-sm" on:click=handle_move_key>
                        {move || t("keys.move_key", lang.get())}
                    </button>
                </div>
                {move || {
                    let err = move_error.get();
                    if err.is_empty() {
                        view! { <div></div> }.into_any()
                    } else {
                        view! { <div class="error-msg">{err}</div> }.into_any()
                    }
                }}
                <p class="settings-hint">{move || t("keys.move_hint", lang.get())}</p>
            </div>

            // Emergency Kit
            <div class="settings-group">
                <h3>{move || t("keys.emergency_kit", lang.get())}</h3>
                <p class="settings-hint">{move || t("keys.emergency_kit_hint", lang.get())}</p>
                {move || {
                    if !kit_phrase.get().is_empty() {
                        let words = kit_phrase.get();
                        view! {
                            <div class="recovery-phrase">
                                <div class="word-grid">
                                    {words.into_iter().enumerate().map(|(i, word)| {
                                        view! {
                                            <div class="word-item">
                                                <span class="word-num">{format!("{}", i + 1)}</span>
                                                <span class="word-text">{word}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                            <button class="btn btn-ghost btn-sm" style="margin-top: 0.5rem;" on:click=move |_| {
                                set_kit_phrase.set(Vec::new());
                                set_kit_password.set(String::new());
                                set_show_kit.set(false);
                            }>
                                {move || t("hide", lang.get())}
                            </button>
                        }.into_any()
                    } else if show_kit.get() {
                        view! {
                            <form class="auth-form" on:submit=handle_kit_verify>
                                <div class="form-group">
                                    <label>{move || t("master_password", lang.get())}</label>
                                    <input
                                        type="password"
                                        placeholder=move || t("keys.enter_master_password", lang.get())
                                        required=true
                                        on:input=move |ev| set_kit_password.set(event_target_value(&ev))
                                    />
                                </div>
                                {move || {
                                    let err = kit_error.get();
                                    if err.is_empty() {
                                        view! { <div></div> }.into_any()
                                    } else {
                                        view! { <div class="error-msg">{err}</div> }.into_any()
                                    }
                                }}
                                <div class="form-actions">
                                    <button type="button" class="btn btn-ghost" on:click=move |_| set_show_kit.set(false)>
                                        {move || t("cancel", lang.get())}
                                    </button>
                                    <button type="submit" class="btn btn-primary" disabled=move || kit_loading.get()>
                                        {move || if kit_loading.get() { t("keys.verifying", lang.get()) } else { t("keys.show", lang.get()) }}
                                    </button>
                                </div>
                            </form>
                        }.into_any()
                    } else {
                        view! {
                            <button class="btn btn-primary btn-danger" on:click=move |_| set_show_kit.set(true)>
                                {move || t("keys.show_emergency_kit", lang.get())}
                            </button>
                        }.into_any()
                    }
                }}
            </div>

            // Danger Zone
            <div class="settings-group danger-zone">
                <h3>{move || t("keys.danger_zone", lang.get())}</h3>

                // Change master password
                {move || {
                    if change_pwd_success.get() {
                        view! {
                            <div class="info-msg">{move || t("keys.password_changed_success", lang.get())}</div>
                        }.into_any()
                    } else if show_change_pwd.get() {
                        view! {
                            <form class="auth-form" on:submit=handle_change_pwd>
                                <div class="form-group">
                                    <label>{move || t("keys.current_password", lang.get())}</label>
                                    <input
                                        type="password"
                                        required=true
                                        on:input=move |ev| set_current_pwd.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-group">
                                    <label>{move || t("keys.new_password", lang.get())}</label>
                                    <input
                                        type="password"
                                        placeholder=move || t("keys.new_password_placeholder", lang.get())
                                        required=true
                                        on:input=move |ev| set_new_pwd.set(event_target_value(&ev))
                                    />
                                </div>
                                {move || {
                                    let err = change_pwd_error.get();
                                    if err.is_empty() {
                                        view! { <div></div> }.into_any()
                                    } else {
                                        view! { <div class="error-msg">{err}</div> }.into_any()
                                    }
                                }}
                                <div class="form-actions">
                                    <button type="button" class="btn btn-ghost" on:click=move |_| set_show_change_pwd.set(false)>
                                        {move || t("cancel", lang.get())}
                                    </button>
                                    <button type="submit" class="btn btn-primary btn-danger" disabled=move || change_pwd_loading.get()>
                                        {move || if change_pwd_loading.get() { t("keys.changing", lang.get()) } else { t("keys.change_btn", lang.get()) }}
                                    </button>
                                </div>
                            </form>
                        }.into_any()
                    } else {
                        view! {
                            <button class="btn btn-ghost btn-danger" on:click=move |_| set_show_change_pwd.set(true)>
                                {move || t("keys.change_password", lang.get())}
                            </button>
                        }.into_any()
                    }
                }}

                <hr class="divider" />

                // Regenerate device key
                {move || {
                    if regen_success.get() {
                        view! {
                            <div class="info-msg">{move || t("keys.regen_success", lang.get())}</div>
                        }.into_any()
                    } else if show_regen.get() {
                        view! {
                            <div>
                                <div class="warning-text">{move || t("keys.regen_warning", lang.get())}</div>
                                <form class="auth-form" style="margin-top: 1rem;" on:submit=handle_regen_key>
                                    <div class="form-group">
                                        <label>{move || t("master_password", lang.get())}</label>
                                        <input
                                            type="password"
                                            required=true
                                            on:input=move |ev| set_regen_pwd.set(event_target_value(&ev))
                                        />
                                    </div>
                                    {move || {
                                        let err = regen_error.get();
                                        if err.is_empty() {
                                            view! { <div></div> }.into_any()
                                        } else {
                                            view! { <div class="error-msg">{err}</div> }.into_any()
                                        }
                                    }}
                                    <div class="form-actions">
                                        <button type="button" class="btn btn-ghost" on:click=move |_| set_show_regen.set(false)>
                                            {move || t("cancel", lang.get())}
                                        </button>
                                        <button type="submit" class="btn btn-primary btn-danger" disabled=move || regen_loading.get()>
                                            {move || if regen_loading.get() { t("keys.regenerating", lang.get()) } else { t("keys.regenerate", lang.get()) }}
                                        </button>
                                    </div>
                                </form>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <button class="btn btn-ghost btn-danger" on:click=move |_| set_show_regen.set(true)>
                                {move || t("keys.regenerate_local_key", lang.get())}
                            </button>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
