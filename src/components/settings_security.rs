use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};
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
    let lang = expect_context::<ReadSignal<Language>>();
    let save = on_save.clone();

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("sec.section_title", lang.get())}</h2>
            <p class="settings-section-desc">{move || t("sec.section_desc", lang.get())}</p>

            // Auto-Lock Timeout
            <div class="settings-group">
                <h3>{move || t("sec.auto_lock", lang.get())}</h3>
                <div class="settings-row">
                    <label>{move || t("sec.auto_lock_timeout", lang.get())}</label>
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
                        <option value="immediate" selected=move || settings.get().auto_lock_timeout == AutoLockTimeout::Immediate>{move || t("sec.immediate", lang.get())}</option>
                        <option value="1min" selected=move || settings.get().auto_lock_timeout == AutoLockTimeout::After1Min>{move || t("sec.after_1min", lang.get())}</option>
                        <option value="5min" selected=move || settings.get().auto_lock_timeout == AutoLockTimeout::After5Min>{move || t("sec.after_5min", lang.get())}</option>
                        <option value="never" selected=move || settings.get().auto_lock_timeout == AutoLockTimeout::Never>{move || t("sec.never", lang.get())}</option>
                    </select>
                </div>

                <div class="settings-row">
                    <label>{move || t("sec.lock_on_sleep", lang.get())}</label>
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
                    <label>{move || t("sec.lock_on_close", lang.get())}</label>
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
                    <label>{move || t("sec.lock_on_inactivity", lang.get())}</label>
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
                <h3>{move || t("sec.clipboard_title", lang.get())}</h3>
                <div class="settings-row">
                    <label>{move || t("sec.clipboard_clear", lang.get())}</label>
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
                <h3>{move || t("sec.screenshot", lang.get())}</h3>
                <div class="settings-row">
                    <label>{move || t("sec.screenshot_block", lang.get())}</label>
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
                <p class="settings-hint">{move || t("sec.screenshot_hint", lang.get())}</p>
                {move || {
                    if !settings.get().screenshot_protection {
                        view! {
                            <p class="settings-hint settings-hint-warn">{move || t("sec.screenshot_warn", lang.get())}</p>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
