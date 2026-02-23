use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::components::settings::{AutoLockTimeout, UserSettings};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn SettingsSecurity(
    settings: ReadSignal<UserSettings>,
    set_settings: WriteSignal<UserSettings>,
    on_save: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let save = on_save.clone();

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">"🛡️ Sécurité & Verrouillage"</h2>
            <p class="settings-section-desc">"Configurez le comportement défensif de l'application."</p>

            // Auto-Lock Timeout
            <div class="settings-group">
                <h3>"Verrouillage automatique"</h3>
                <div class="settings-row">
                    <label>"Délai de verrouillage"</label>
                    <select
                        class="settings-select"
                        on:change={
                            let save = save.clone();
                            move |ev| {
                                let val = event_target_value(&ev);
                                let mut s = settings.get_untracked();
                                s.auto_lock_timeout = match val.as_str() {
                                    "immediate" => AutoLockTimeout::Immediate,
                                    "1min" => AutoLockTimeout::After1Min,
                                    "5min" => AutoLockTimeout::After5Min,
                                    _ => AutoLockTimeout::Never,
                                };
                                set_settings.set(s);
                                save();
                            }
                        }
                    >
                        <option value="immediate" selected=move || settings.get().auto_lock_timeout == AutoLockTimeout::Immediate>"Immédiatement"</option>
                        <option value="1min" selected=move || settings.get().auto_lock_timeout == AutoLockTimeout::After1Min>"Après 1 minute"</option>
                        <option value="5min" selected=move || settings.get().auto_lock_timeout == AutoLockTimeout::After5Min>"Après 5 minutes"</option>
                        <option value="never" selected=move || settings.get().auto_lock_timeout == AutoLockTimeout::Never>"Jamais"</option>
                    </select>
                </div>

                <div class="settings-row">
                    <label>"À la mise en veille du système"</label>
                    <input
                        type="checkbox"
                        class="settings-toggle"
                        prop:checked=move || settings.get().auto_lock_on_sleep
                        on:change={
                            let save = save.clone();
                            move |ev| {
                                let checked = event_target_checked(&ev);
                                let mut s = settings.get_untracked();
                                s.auto_lock_on_sleep = checked;
                                set_settings.set(s);
                                save();
                            }
                        }
                    />
                </div>

                <div class="settings-row">
                    <label>"À la fermeture de la fenêtre"</label>
                    <input
                        type="checkbox"
                        class="settings-toggle"
                        prop:checked=move || settings.get().auto_lock_on_close
                        on:change={
                            let save = save.clone();
                            move |ev| {
                                let checked = event_target_checked(&ev);
                                let mut s = settings.get_untracked();
                                s.auto_lock_on_close = checked;
                                set_settings.set(s);
                                save();
                            }
                        }
                    />
                </div>

                <div class="settings-row">
                    <label>"En cas d'inactivité"</label>
                    <input
                        type="checkbox"
                        class="settings-toggle"
                        prop:checked=move || settings.get().auto_lock_on_inactivity
                        on:change={
                            let save = save.clone();
                            move |ev| {
                                let checked = event_target_checked(&ev);
                                let mut s = settings.get_untracked();
                                s.auto_lock_on_inactivity = checked;
                                set_settings.set(s);
                                save();
                            }
                        }
                    />
                </div>
            </div>

            // Clipboard
            <div class="settings-group">
                <h3>"Presse-papiers"</h3>
                <div class="settings-row">
                    <label>"Vider le presse-papiers après (secondes)"</label>
                    <input
                        type="number"
                        class="settings-input-sm"
                        min="5"
                        max="300"
                        prop:value=move || settings.get().clipboard_clear_seconds.to_string()
                        on:change={
                            let save = save.clone();
                            move |ev| {
                                let val: u32 = event_target_value(&ev).parse().unwrap_or(30);
                                let mut s = settings.get_untracked();
                                s.clipboard_clear_seconds = val.max(5).min(300);
                                set_settings.set(s);
                                save();
                            }
                        }
                    />
                </div>
            </div>

            // Screenshot protection
            <div class="settings-group">
                <h3>"Protection contre les captures d'écran"</h3>
                <div class="settings-row">
                    <label>"Bloquer les captures d'écran de l'application"</label>
                    <input
                        type="checkbox"
                        class="settings-toggle"
                        prop:checked=move || settings.get().screenshot_protection
                        on:change={
                            let save = save.clone();
                            move |ev| {
                                let checked = event_target_checked(&ev);
                                let mut s = settings.get_untracked();
                                s.screenshot_protection = checked;
                                set_settings.set(s);
                                save();
                                // Apply immediately
                                spawn_local(async move {
                                    #[derive(Serialize)]
                                    struct Args { enabled: bool }
                                    let args = serde_wasm_bindgen::to_value(&Args { enabled: checked }).unwrap();
                                    let _ = invoke("apply_screenshot_protection", args).await;
                                });
                            }
                        }
                    />
                </div>
                <p class="settings-hint">"Empêche les applications de capture d'écran de voir le contenu de SaladVault."</p>
                {move || {
                    if !settings.get().screenshot_protection {
                        view! {
                            <p class="settings-hint settings-hint-warn">"La protection est désactivée — vos données sont visibles par les outils de capture d'écran."</p>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
