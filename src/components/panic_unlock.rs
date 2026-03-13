use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
struct OpenSaladierArgs {
    uuid: String,
    password: String,
}

#[derive(Serialize)]
struct UuidArgs {
    uuid: String,
}

#[derive(Deserialize)]
struct AttemptsInfo {
    remaining: Option<u32>,
}

#[component]
pub fn PanicUnlock(
    saladier_uuid: String,
    saladier_name: String,
    on_unlocked: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (password, set_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (destroyed, set_destroyed) = signal(false);
    let uuid = saladier_uuid.clone();

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error_msg.set(String::new());

        let uuid_inner = uuid.clone();
        let uuid_for_info = uuid_inner.clone();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&OpenSaladierArgs {
                uuid: uuid_inner,
                password: password.get_untracked(),
            })
            .unwrap();

            match invoke("open_saladier", args).await {
                Ok(_) => {
                    set_loading.set(false);
                    on_unlocked.run(());
                }
                Err(_) => {
                    set_loading.set(false);

                    // Fetch remaining attempts
                    let info_args = serde_wasm_bindgen::to_value(&UuidArgs {
                        uuid: uuid_for_info,
                    })
                    .unwrap();

                    match invoke("get_saladier_attempts_info", info_args).await {
                        Ok(info_val) => {
                            if let Ok(info) = serde_wasm_bindgen::from_value::<AttemptsInfo>(info_val) {
                                match info.remaining {
                                    Some(0) => {
                                        set_error_msg.set("Saladier détruit : nombre maximal de tentatives atteint.".to_string());
                                        set_destroyed.set(true);
                                        // Auto-return to dashboard after 3s
                                        gloo_timers::callback::Timeout::new(3_000, move || {
                                            on_cancel.run(());
                                        }).forget();
                                    }
                                    Some(n) => {
                                        set_error_msg.set(format!(
                                            "Mot de passe incorrect. {} tentative(s) restante(s).",
                                            n
                                        ));
                                    }
                                    None => {
                                        // Feature disabled (max=0)
                                        set_error_msg.set("Mot de passe du Saladier incorrect.".to_string());
                                    }
                                }
                            } else {
                                set_error_msg.set("Mot de passe du Saladier incorrect.".to_string());
                            }
                        }
                        Err(_) => {
                            // Saladier may have been destroyed
                            set_error_msg.set("Saladier détruit : nombre maximal de tentatives atteint.".to_string());
                            set_destroyed.set(true);
                            gloo_timers::callback::Timeout::new(3_000, move || {
                                on_cancel.run(());
                            }).forget();
                        }
                    }
                }
            }
        });
    };

    view! {
        <div class="modal-overlay">
            <div class="modal-card">
                <div class="modal-header">
                    <h2>"Déverrouiller le Saladier"</h2>
                    <p class="modal-subtitle">{saladier_name}</p>
                </div>

                <form class="auth-form" on:submit=handle_submit>
                    <div class="form-group">
                        <label for="saladier-pwd">"Mot de passe du Saladier"</label>
                        <input
                            id="saladier-pwd"
                            type="password"
                            placeholder="••••••••••••"
                            required=true
                            autofocus=true
                            disabled=move || destroyed.get()
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                        <span class="form-hint">"Ce mot de passe protège spécifiquement ce Saladier (Panic Mode)."</span>
                    </div>

                    {move || {
                        let err = error_msg.get();
                        if err.is_empty() {
                            view! { <div></div> }.into_any()
                        } else {
                            view! { <div class="error-msg">{err}</div> }.into_any()
                        }
                    }}

                    <div class="form-actions">
                        <button type="button" class="btn btn-ghost" on:click=move |_| on_cancel.run(())>
                            "Annuler"
                        </button>
                        <button type="submit" class="btn btn-primary" disabled=move || loading.get() || destroyed.get()>
                            {move || if loading.get() { "Déverrouillage..." } else { "Déverrouiller" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
