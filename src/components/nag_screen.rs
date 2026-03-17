use leptos::task::spawn_local;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Nag Screen: forces the user to generate and save their recovery phrase
/// before accessing the Dashboard. There is no "Skip" button.
#[component]
pub fn NagScreen(
    on_confirmed: Callback<()>,
) -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();

    let (phrase, set_phrase) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (phrase_generated, set_phrase_generated) = signal(false);
    let (confirming, set_confirming) = signal(false);

    let generate = move |_| {
        set_loading.set(true);
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let result = invoke("generate_recovery_phrase", args).await;
            set_loading.set(false);

            if let Ok(result) = result {
                if let Some(p) = result.as_string() {
                    if p.contains(' ') {
                        set_phrase.set(p);
                        set_phrase_generated.set(true);
                    }
                }
            }
        });
    };

    let confirm = move |_| {
        set_confirming.set(true);
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let _ = invoke("confirm_recovery_saved", args).await;
            set_confirming.set(false);
            on_confirmed.run(());
        });
    };

    view! {
        <div class="nag-screen">
            <div class="nag-card">
                <div class="nag-icon">"🚨"</div>
                <h1 class="nag-title">{move || t("nag.title", lang.get())}</h1>
                <p class="nag-subtitle">
                    {move || t("nag.subtitle_1", lang.get())} <code>"device_secret.key"</code> {move || t("nag.subtitle_2", lang.get())}
                    <strong>{move || t("nag.subtitle_3", lang.get())}</strong> {move || t("nag.subtitle_4", lang.get())}
                </p>

                <div class="nag-warning">
                    <div class="warning-icon">"⚠️"</div>
                    <p>
                        {move || t("nag.warning_1", lang.get())} <strong>{move || t("nag.warning_2", lang.get())}</strong> {move || t("nag.warning_3", lang.get())}
                    </p>
                </div>

                {move || {
                    let p = phrase.get();
                    if p.is_empty() {
                        // Step 1: Generate the phrase
                        view! {
                            <div class="nag-step">
                                <h2 class="step-title">{move || t("nag.step1_generate", lang.get())}</h2>
                                <button class="btn btn-primary btn-lg" on:click=generate disabled=move || loading.get()>
                                    {move || if loading.get() { t("nag.generating", lang.get()) } else { t("nag.generate_btn", lang.get()) }}
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        // Step 2: Display phrase and confirm
                        let words: Vec<String> = p.split_whitespace().map(|s| s.to_string()).collect();
                        view! {
                            <div class="nag-step">
                                <h2 class="step-title">{move || t("nag.step1_display", lang.get())}</h2>
                                <div class="recovery-phrase">
                                    <div class="word-grid">
                                        {words.into_iter().enumerate().map(|(i, word)| {
                                            view! {
                                                <div class="word-item">
                                                    <span class="word-num">{format!("{}", i + 1)}</span>
                                                    <span class="word-text">{word}</span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                <div class="nag-warning nag-warning-red">
                                    <p>
                                        {move || t("nag.phrase_warning", lang.get())}
                                    </p>
                                </div>

                                <h2 class="step-title">{move || t("nag.step2", lang.get())}</h2>
                                <button
                                    class="btn btn-primary btn-lg"
                                    on:click=confirm
                                    disabled=move || confirming.get() || !phrase_generated.get()
                                >
                                    {move || if confirming.get() { t("nag.confirming", lang.get()) } else { t("nag.confirm_btn", lang.get()) }}
                                </button>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
