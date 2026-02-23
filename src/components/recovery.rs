use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
struct RecoverArgs {
    phrase: String,
}

#[component]
pub fn Recovery(
    on_close: WriteSignal<bool>,
) -> impl IntoView {
    let (phrase, set_phrase) = signal(String::new());
    let (recovery_input, set_recovery_input) = signal(String::new());
    let (show_recover, set_show_recover) = signal(false);
    let (loading, set_loading) = signal(false);
    let (message, set_message) = signal(String::new());

    let generate = move |_| {
        set_loading.set(true);
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let result = invoke("generate_recovery_phrase", args).await;
            set_loading.set(false);

            match result {
                Ok(val) => {
                    if let Some(p) = val.as_string() {
                        if p.contains(' ') {
                            set_phrase.set(p);
                        } else {
                            set_message.set("Erreur lors de la génération de la phrase.".to_string());
                        }
                    }
                }
                Err(err) => {
                    set_message.set(format!("Erreur : {}", err.as_string().unwrap_or_default()));
                }
            }
        });
    };

    let handle_recover = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_message.set(String::new());

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&RecoverArgs {
                phrase: recovery_input.get_untracked(),
            })
            .unwrap();

            let result = invoke("recover_from_phrase", args).await;

            set_loading.set(false);

            match result {
                Ok(_) => {
                    set_message.set("Clé de périphérique restaurée avec succès !".to_string());
                }
                Err(_) => {
                    set_message.set("Phrase de récupération invalide.".to_string());
                }
            }
        });
    };

    view! {
        <div class="modal-overlay">
            <div class="modal-card modal-wide">
                <div class="modal-header">
                    <h2>"Kit de Secours"</h2>
                    <p class="modal-subtitle">"Gérez votre phrase de récupération"</p>
                </div>

                <div class="recovery-content">
                    <div class="recovery-section">
                        <h3>"Générer la Phrase de Récupération"</h3>
                        <p class="form-hint">
                            "Cette phrase de 24 mots permet de régénérer votre clé de périphérique. "
                            <strong>"Imprimez-la et gardez-la en lieu sûr !"</strong>
                        </p>
                        <button class="btn btn-primary" on:click=generate disabled=move || loading.get()>
                            "Générer la Phrase"
                        </button>

                        {move || {
                            let p = phrase.get();
                            if p.is_empty() {
                                view! { <div></div> }.into_any()
                            } else {
                                let words: Vec<String> = p.split_whitespace().map(|s| s.to_string()).collect();
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
                                            }).collect_view()}
                                        </div>
                                        <p class="warning-text">
                                            "ATTENTION : Ne partagez JAMAIS cette phrase. Quiconque la possède peut accéder à vos données."
                                        </p>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>

                    <hr class="divider" />

                    <div class="recovery-section">
                        <h3>"Restaurer depuis une Phrase"</h3>
                        <button
                            class="btn btn-ghost"
                            on:click=move |_| set_show_recover.set(!show_recover.get_untracked())
                        >
                            {move || if show_recover.get() { "Masquer" } else { "Afficher le formulaire de restauration" }}
                        </button>

                        {move || {
                            if show_recover.get() {
                                view! {
                                    <form class="auth-form" on:submit=handle_recover>
                                        <div class="form-group">
                                            <label>"Phrase de 24 mots"</label>
                                            <textarea
                                                rows=3
                                                placeholder="mot1 mot2 mot3 ..."
                                                required=true
                                                on:input=move |ev| set_recovery_input.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                                            "Restaurer la clé"
                                        </button>
                                    </form>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}
                    </div>

                    {move || {
                        let msg = message.get();
                        if msg.is_empty() {
                            view! { <div></div> }.into_any()
                        } else {
                            view! { <div class="info-msg">{msg}</div> }.into_any()
                        }
                    }}
                </div>

                <div class="form-actions">
                    <button class="btn btn-ghost" on:click=move |_| on_close.set(true)>
                        "Fermer"
                    </button>
                </div>
            </div>
        </div>
    }
}
