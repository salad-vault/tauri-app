use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};
use crate::components::settings::UserSettings;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SaladierInfo {
    uuid: String,
    name: String,
}

#[component]
pub fn SettingsSaladiers(
    settings: ReadSignal<UserSettings>,
    set_settings: WriteSignal<UserSettings>,
    on_save: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let (saladiers, set_saladiers) = signal(Vec::<SaladierInfo>::new());
    let save = on_save.clone();

    // Load visible saladiers
    {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            if let Ok(result) = invoke("list_saladiers", args).await {
                if let Ok(list) = serde_wasm_bindgen::from_value::<Vec<SaladierInfo>>(result) {
                    set_saladiers.set(list);
                }
            }
        });
    }

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("saladiers.section_title", lang.get())}</h2>
            <p class="settings-section-desc">{move || t("saladiers.section_desc", lang.get())}</p>

            <div class="settings-group">
                <h3>{move || t("saladiers.active", lang.get())}</h3>
                <div class="saladier-list">
                    {move || {
                        let list = saladiers.get();
                        if list.is_empty() {
                            view! { <p class="settings-hint">{move || t("saladiers.none_visible", lang.get())}</p> }.into_any()
                        } else {
                            view! {
                                <div class="settings-saladier-grid">
                                    {list.into_iter().map(|s| {
                                        view! {
                                            <div class="settings-saladier-item">
                                                <span class="saladier-icon">"🥗"</span>
                                                <span>{s.name}</span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            <div class="settings-group">
                <h3>{move || t("saladiers.security_policy", lang.get())}</h3>
                <div class="settings-row">
                    <label>{move || t("saladiers.max_attempts", lang.get())}</label>
                    <input
                        type="number"
                        class="settings-input-sm"
                        min="0"
                        max="100"
                        prop:value=move || settings.get().max_failed_attempts.to_string()
                        on:change={
                            let save = save.clone();
                            move |ev| {
                                let val: u32 = event_target_value(&ev).parse().unwrap_or(0);
                                let mut s = settings.get_untracked();
                                s.max_failed_attempts = val;
                                set_settings.set(s);
                                save();
                            }
                        }
                    />
                </div>
                <p class="settings-hint">{move || t("saladiers.max_attempts_hint", lang.get())}</p>
                {move || {
                    if settings.get().max_failed_attempts == 0 {
                        view! {
                            <p class="settings-hint settings-hint-warn">{move || t("saladiers.autodestruct_disabled_warn", lang.get())}</p>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
