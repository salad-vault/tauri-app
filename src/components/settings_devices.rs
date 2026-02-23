use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn SettingsDevices() -> impl IntoView {
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
            <h2 class="settings-section-title">"📱 Appareils & Synchronisation"</h2>
            <p class="settings-section-desc">"Gérez le lien entre vos appareils."</p>

            <div class="settings-group">
                <h3>"Appairer un nouvel appareil"</h3>
                <p class="settings-hint">"Générez un QR code contenant votre clé de périphérique pour la transférer vers un autre appareil."</p>

                {move || {
                    let data = qr_data.get();
                    if !data.is_empty() {
                        view! {
                            <div class="qr-display">
                                <div class="qr-code-svg" inner_html=data.clone()></div>
                                <p class="settings-hint">"Scannez ce code avec l'application mobile SaladVault pour transférer votre clé."</p>
                                <button class="btn btn-ghost btn-sm" on:click=move |_| set_qr_data.set(String::new())>
                                    "Masquer"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <button class="btn btn-primary" on:click=generate_qr disabled=move || loading.get()>
                                {move || if loading.get() { "Génération..." } else { "📷 Générer le QR Code" }}
                            </button>
                        }.into_any()
                    }
                }}
            </div>

            <div class="settings-group">
                <h3>"Appareils connectés"</h3>
                <div class="settings-note">
                    <span class="note-icon">"ℹ️"</span>
                    <p>"La synchronisation entre appareils et la révocation des sessions seront disponibles dans une prochaine version avec le serveur de synchronisation."</p>
                </div>
            </div>
        </div>
    }
}
