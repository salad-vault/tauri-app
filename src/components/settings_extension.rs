use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn SettingsExtension() -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let (pairing_code, set_pairing_code) = signal(Option::<String>::None);
    let (is_paired, set_is_paired) = signal(false);
    let (loading, set_loading) = signal(false);
    let (msg, set_msg) = signal(String::new());

    // Check bridge status on mount
    {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            if let Ok(result) = invoke("get_bridge_status", args).await {
                if let Some(paired) = js_sys::Reflect::get(&result, &"paired".into())
                    .ok()
                    .and_then(|v| v.as_bool())
                {
                    set_is_paired.set(paired);
                }
            }
        });
    }

    let handle_generate_code = move |_| {
        set_loading.set(true);
        set_msg.set(String::new());
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            match invoke("generate_pairing_code", args).await {
                Ok(code_js) => {
                    if let Some(code) = code_js.as_string() {
                        set_pairing_code.set(Some(code));
                    }
                }
                Err(err) => {
                    set_msg.set(err.as_string().unwrap_or_default());
                }
            }
            set_loading.set(false);
        });
    };

    let handle_revoke = move |_| {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let _ = invoke("revoke_bridge_token", args).await;
            set_is_paired.set(false);
            set_pairing_code.set(None);
            set_msg.set(t("ext.revoked", lang.get()).to_string());
        });
    };

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("ext.title", lang.get())}</h2>
            <p class="settings-section-desc">{move || t("ext.desc", lang.get())}</p>

            <div class="settings-group">
                <h3>{move || t("ext.status_title", lang.get())}</h3>
                <div class="settings-row">
                    <label>{move || t("ext.status", lang.get())}</label>
                    {move || if is_paired.get() {
                        view! { <span class="badge badge-success">{move || t("ext.paired", lang.get())}</span> }.into_any()
                    } else {
                        view! { <span class="badge badge-muted">{move || t("ext.not_paired", lang.get())}</span> }.into_any()
                    }}
                </div>
            </div>

            // Pairing section
            <div class="settings-group">
                <h3>{move || t("ext.pair_title", lang.get())}</h3>
                <p class="settings-hint">{move || t("ext.pair_hint", lang.get())}</p>

                {move || {
                    if let Some(code) = pairing_code.get() {
                        view! {
                            <div class="pairing-code-display">
                                <span class="pairing-code">{code}</span>
                                <p class="settings-hint">{move || t("ext.code_hint", lang.get())}</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <button
                                class="btn btn-primary btn-sm"
                                on:click=handle_generate_code
                                disabled=move || loading.get()
                            >
                                {move || if loading.get() {
                                    t("ext.generating", lang.get())
                                } else {
                                    t("ext.generate_code", lang.get())
                                }}
                            </button>
                        }.into_any()
                    }
                }}
            </div>

            // Revoke section
            <Show when=move || is_paired.get()>
                <div class="settings-group settings-danger-zone">
                    <h3>{move || t("ext.revoke_title", lang.get())}</h3>
                    <p class="settings-hint">{move || t("ext.revoke_hint", lang.get())}</p>
                    <button class="btn btn-ghost btn-danger btn-sm" on:click=handle_revoke>
                        {move || t("ext.revoke", lang.get())}
                    </button>
                </div>
            </Show>

            // Messages
            {move || {
                let m = msg.get();
                if m.is_empty() {
                    view! { <div></div> }.into_any()
                } else {
                    view! { <div class="info-msg">{m}</div> }.into_any()
                }
            }}
        </div>
    }
}
