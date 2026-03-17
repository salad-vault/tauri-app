use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn SettingsDevices() -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let (qr_data, set_qr_data) = signal(String::new());
    let (loading, set_loading) = signal(false);

    let generate_qr = move |_| {
        set_loading.set(true);
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            if let Ok(result) = invoke("generate_device_key_qr_svg", args).await {
                if let Some(data) = result.as_string() {
                    set_qr_data.set(data);
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("devices.section_title", lang.get())}</h2>
            <p class="settings-section-desc">{move || t("devices.section_desc", lang.get())}</p>

            <div class="settings-group">
                <h3>{move || t("devices.pair_new", lang.get())}</h3>
                <p class="settings-hint">{move || t("devices.pair_hint", lang.get())}</p>

                {move || {
                    let data = qr_data.get();
                    if !data.is_empty() {
                        view! {
                            <div class="qr-display">
                                <div class="qr-code-svg" inner_html=data.clone()></div>
                                <p class="settings-hint">{move || t("devices.scan_hint", lang.get())}</p>
                                <button class="btn btn-ghost btn-sm" on:click=move |_| set_qr_data.set(String::new())>
                                    {move || t("hide", lang.get())}
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <button class="btn btn-primary" on:click=generate_qr disabled=move || loading.get()>
                                {move || if loading.get() { t("devices.generating", lang.get()) } else { t("devices.generate_qr", lang.get()) }}
                            </button>
                        }.into_any()
                    }
                }}
            </div>

            <div class="settings-group">
                <h3>{move || t("devices.connected_devices", lang.get())}</h3>
                <div class="settings-note">
                    <span class="note-icon">"ℹ️"</span>
                    <p>{move || t("devices.sync_coming_soon", lang.get())}</p>
                </div>
            </div>
        </div>
    }
}
