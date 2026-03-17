use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::components::password_utils::validate_password;
use crate::i18n::{t, Language};

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
    on_registered: Callback<()>,
    on_switch_login: Callback<()>,
) -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();

    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(String::new());
    let (loading, set_loading) = signal(false);

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        if password.get_untracked() != confirm_password.get_untracked() {
            set_error_msg.set(t("register.passwords_mismatch", lang.get()).to_string());
            return;
        }

        if let Err(msg) = validate_password(&password.get_untracked(), lang.get_untracked()) {
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
                    on_registered.run(());
                }
                Err(err) => {
                    set_error_msg.set(err.as_string().unwrap_or_else(|| t("register.error_creating", lang.get()).to_string()));
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <div class="auth-header">
                    <div class="auth-icon">"🌱"</div>
                    <h1 class="auth-title">{move || t("register.title", lang.get())}</h1>
                    <p class="auth-subtitle">{move || t("register.subtitle", lang.get())}</p>
                </div>

                <form class="auth-form" on:submit=handle_submit>
                    <div class="form-group">
                        <label for="reg-email">{move || t("email", lang.get())}</label>
                        <input
                            id="reg-email"
                            type="email"
                            placeholder=move || t("login.email_placeholder", lang.get())
                            required=true
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                        <span class="form-hint">{move || t("register.email_hint", lang.get())}</span>
                    </div>

                    <div class="form-group">
                        <label for="reg-password">{move || t("master_password", lang.get())}</label>
                        <input
                            id="reg-password"
                            type="password"
                            placeholder=move || t("register.password_placeholder", lang.get())
                            required=true
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="form-group">
                        <label for="reg-confirm">{move || t("register.confirm_password", lang.get())}</label>
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
                        {move || if loading.get() { t("register.creating", lang.get()) } else { t("register.create_account", lang.get()) }}
                    </button>
                </form>

                <div class="auth-footer">
                    <button
                        class="btn btn-link"
                        on:click=move |_| on_switch_login.run(())
                    >
                        {move || t("register.already_account", lang.get())}
                    </button>
                </div>
            </div>
        </div>
    }
}
