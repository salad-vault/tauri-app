use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize)]
struct UnlockArgs {
    email: String,
    #[serde(rename = "masterPassword")]
    master_password: String,
}

#[component]
pub fn Login(
    on_login: WriteSignal<bool>,
    on_switch_register: WriteSignal<bool>,
) -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(String::new());
    let (loading, set_loading) = signal(false);

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error_msg.set(String::new());

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&UnlockArgs {
                email: email.get_untracked(),
                master_password: password.get_untracked(),
            })
            .unwrap();

            match invoke("unlock", args).await {
                Ok(_) => {
                    set_loading.set(false);
                    on_login.set(true);
                }
                Err(err) => {
                    set_loading.set(false);
                    let msg = err.as_string().unwrap_or_default();
                    if msg.is_empty() {
                        set_error_msg.set("Identifiants invalides".to_string());
                    } else {
                        set_error_msg.set(msg);
                    }
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <div class="auth-header">
                    <div class="auth-icon">"🥗"</div>
                    <h1 class="auth-title">"SaladVault"</h1>
                    <p class="auth-subtitle">"Déverrouillez votre Potager"</p>
                </div>

                <form class="auth-form" on:submit=handle_submit>
                    <div class="form-group">
                        <label for="email">"Email"</label>
                        <input
                            id="email"
                            type="email"
                            placeholder="votre@email.com"
                            required=true
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="form-group">
                        <label for="password">"Mot de Passe Maître"</label>
                        <input
                            id="password"
                            type="password"
                            placeholder="••••••••••••"
                            required=true
                            on:input=move |ev| set_password.set(event_target_value(&ev))
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
                        {move || if loading.get() { "Déverrouillage..." } else { "Déverrouiller" }}
                    </button>
                </form>

                <div class="auth-footer">
                    <button
                        class="btn btn-link"
                        on:click=move |_| on_switch_register.set(true)
                    >
                        "Créer un compte"
                    </button>
                </div>
            </div>
        </div>
    }
}
