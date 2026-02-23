use leptos::task::spawn_local;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Nag Screen: forces the user to generate and save their recovery phrase
/// before accessing the Dashboard. There is no "Skip" button.
#[component]
pub fn NagScreen(
    on_confirmed: WriteSignal<bool>,
) -> impl IntoView {
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
            on_confirmed.set(true);
        });
    };

    view! {
        <div class="nag-screen">
            <div class="nag-card">
                <div class="nag-icon">"🚨"</div>
                <h1 class="nag-title">"Sauvegardez votre Kit de Secours"</h1>
                <p class="nag-subtitle">
                    "Votre fichier " <code>"device_secret.key"</code> " est essentiel pour accéder à vos données. "
                    "Si vous changez d'ordinateur ou perdez ce fichier, "
                    <strong>"vous ne pourrez plus jamais vous connecter"</strong> " sans cette phrase de récupération."
                </p>

                <div class="nag-warning">
                    <div class="warning-icon">"⚠️"</div>
                    <p>
                        "Vous " <strong>"ne pouvez pas ignorer"</strong> " cette étape. "
                        "Générez votre phrase de 24 mots, notez-la sur papier ou imprimez-la, "
                        "puis confirmez l'avoir sauvegardée."
                    </p>
                </div>

                {move || {
                    let p = phrase.get();
                    if p.is_empty() {
                        // Step 1: Generate the phrase
                        view! {
                            <div class="nag-step">
                                <h2 class="step-title">"Étape 1 : Générer la phrase"</h2>
                                <button class="btn btn-primary btn-lg" on:click=generate disabled=move || loading.get()>
                                    {move || if loading.get() { "Génération..." } else { "Générer ma phrase de récupération" }}
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        // Step 2: Display phrase and confirm
                        let words: Vec<String> = p.split_whitespace().map(|s| s.to_string()).collect();
                        view! {
                            <div class="nag-step">
                                <h2 class="step-title">"Étape 1 : Votre phrase de récupération"</h2>
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
                                        "ATTENTION : Notez ces 24 mots dans l'ordre exact sur papier. "
                                        "Ne les stockez " <strong>"JAMAIS"</strong> " dans un fichier numérique. "
                                        "Quiconque possède cette phrase peut accéder à vos données."
                                    </p>
                                </div>

                                <h2 class="step-title">"Étape 2 : Confirmer"</h2>
                                <button
                                    class="btn btn-primary btn-lg"
                                    on:click=confirm
                                    disabled=move || confirming.get() || !phrase_generated.get()
                                >
                                    {move || if confirming.get() { "Confirmation..." } else { "J'ai sauvegardé ma phrase de récupération" }}
                                </button>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
