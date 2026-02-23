use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::components::password_utils::validate_password;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize)]
struct RegisterArgs {
    email: String,
    #[serde(rename = "masterPassword")]
    master_password: String,
}

#[component]
pub fn Register(
    on_registered: WriteSignal<bool>,
    on_switch_login: WriteSignal<bool>,
) -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(String::new());
    let (loading, set_loading) = signal(false);

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        if password.get_untracked() != confirm_password.get_untracked() {
            set_error_msg.set("Les mots de passe ne correspondent pas.".to_string());
            return;
        }

        if let Err(msg) = validate_password(&password.get_untracked()) {
            set_error_msg.set(msg);
            return;
        }

        set_loading.set(true);
        set_error_msg.set(String::new());

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&RegisterArgs {
                email: email.get_untracked(),
                master_password: password.get_untracked(),
            })
            .unwrap();

            let result = invoke("register", args).await;

            set_loading.set(false);

            match result {
                Ok(_) => {
                    on_registered.set(true);
                }
                Err(err) => {
                    set_error_msg.set(err.as_string().unwrap_or_else(|| "Erreur lors de la création du compte.".to_string()));
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <div class="auth-header">
                    <div class="auth-icon">"🌱"</div>
                    <h1 class="auth-title">"Créer votre Potager"</h1>
                    <p class="auth-subtitle">"Configurez votre espace sécurisé"</p>
                </div>

                <form class="auth-form" on:submit=handle_submit>
                    <div class="form-group">
                        <label for="reg-email">"Email"</label>
                        <input
                            id="reg-email"
                            type="email"
                            placeholder="votre@email.com"
                            required=true
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                        <span class="form-hint">"Votre email ne sera jamais stocké en clair."</span>
                    </div>

                    <div class="form-group">
                        <label for="reg-password">"Mot de Passe Maître"</label>
                        <input
                            id="reg-password"
                            type="password"
                            placeholder="Min. 16 caractères, maj, min, chiffre, spécial"
                            required=true
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="form-group">
                        <label for="reg-confirm">"Confirmer le Mot de Passe"</label>
                        <input
                            id="reg-confirm"
                            type="password"
                            placeholder="••••••••••••"
                            required=true
                            on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                        />
                    </div>

                    {move || {
                        let err = error_msg.get();
                        if err.is_empty() {
                            view! { <div></div> }.into_any()
                        } else {
                            view! { <div class="error-msg">{err}</div> }.into_any()
                        }
                    }}

                    <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                        {move || if loading.get() { "Création..." } else { "Créer le compte" }}
                    </button>
                </form>

                <div class="auth-footer">
                    <button
                        class="btn btn-link"
                        on:click=move |_| on_switch_login.set(true)
                    >
                        "Déjà un compte ? Se connecter"
                    </button>
                </div>
            </div>
        </div>
    }
}
