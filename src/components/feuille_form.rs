use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::components::saladier_view::{FeuilleData, FeuilleInfo};
use crate::i18n::{t, Language};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
struct CreateFeuilleArgs {
    #[serde(rename = "saladierId")]
    saladier_id: String,
    data: FeuilleData,
}

#[derive(Serialize)]
struct UpdateFeuilleArgs {
    uuid: String,
    data: FeuilleData,
}

#[component]
pub fn FeuilleForm(
    saladier_id: String,
    editing: Option<FeuilleInfo>,
    on_saved: WriteSignal<bool>,
    on_cancel: WriteSignal<bool>,
) -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let is_edit = editing.is_some();
    let edit_uuid = editing.as_ref().map(|f| f.uuid.clone()).unwrap_or_default();

    let (title, set_title) = signal(
        editing.as_ref().map(|f| f.data.title.clone()).unwrap_or_default(),
    );
    let (username, set_username) = signal(
        editing.as_ref().map(|f| f.data.username.clone()).unwrap_or_default(),
    );
    let (password, set_password) = signal(
        editing.as_ref().map(|f| f.data.password.clone()).unwrap_or_default(),
    );
    let (url, set_url) = signal(
        editing.as_ref().map(|f| f.data.url.clone()).unwrap_or_default(),
    );
    let (notes, set_notes) = signal(
        editing.as_ref().map(|f| f.data.notes.clone()).unwrap_or_default(),
    );
    let (error_msg, set_error_msg) = signal(String::new());
    let (loading, set_loading) = signal(false);

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error_msg.set(String::new());

        let data = FeuilleData {
            title: title.get_untracked(),
            username: username.get_untracked(),
            password: password.get_untracked(),
            url: url.get_untracked(),
            notes: notes.get_untracked(),
        };

        let sid = saladier_id.clone();
        let uid = edit_uuid.clone();

        spawn_local(async move {
            let result = if is_edit {
                let args = serde_wasm_bindgen::to_value(&UpdateFeuilleArgs {
                    uuid: uid,
                    data,
                })
                .unwrap();
                invoke("update_feuille", args).await
            } else {
                let args = serde_wasm_bindgen::to_value(&CreateFeuilleArgs {
                    saladier_id: sid,
                    data,
                })
                .unwrap();
                invoke("create_feuille", args).await
            };

            set_loading.set(false);

            match result {
                Ok(_) => {
                    on_saved.set(true);
                }
                Err(err) => {
                    set_error_msg.set(err.as_string().unwrap_or_else(|| t("ff.error_saving", lang.get()).to_string()));
                }
            }
        });
    };

    view! {
        <div class="modal-overlay">
            <div class="modal-card modal-wide">
                <div class="modal-header">
                    <h2>{move || if is_edit { t("ff.edit_title", lang.get()) } else { t("ff.new_title", lang.get()) }}</h2>
                </div>

                <form class="auth-form" on:submit=handle_submit>
                    <div class="form-group">
                        <label>{move || t("ff.title", lang.get())}</label>
                        <input
                            type="text"
                            placeholder=move || t("ff.title_placeholder", lang.get())
                            required=true
                            prop:value=move || title.get()
                            on:input=move |ev| set_title.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="form-row">
                        <div class="form-group">
                            <label>{move || t("ff.username", lang.get())}</label>
                            <input
                                type="text"
                                placeholder=move || t("ff.username_placeholder", lang.get())
                                prop:value=move || username.get()
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="form-group">
                            <label>{move || t("password", lang.get())}</label>
                            <input
                                type="password"
                                placeholder=move || t("ff.password_placeholder", lang.get())
                                prop:value=move || password.get()
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    <div class="form-group">
                        <label>"URL"</label>
                        <input
                            type="url"
                            placeholder="https://..."
                            prop:value=move || url.get()
                            on:input=move |ev| set_url.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="form-group">
                        <label>{move || t("notes", lang.get())}</label>
                        <textarea
                            placeholder=move || t("ff.notes_placeholder", lang.get())
                            rows=3
                            prop:value=move || notes.get()
                            on:input=move |ev| set_notes.set(event_target_value(&ev))
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

                    <div class="form-actions">
                        <button type="button" class="btn btn-ghost" on:click=move |_| on_cancel.set(true)>
                            {move || t("cancel", lang.get())}
                        </button>
                        <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                            {move || if loading.get() { t("ff.saving", lang.get()) } else if is_edit { t("ff.update", lang.get()) } else { t("create", lang.get()) }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
