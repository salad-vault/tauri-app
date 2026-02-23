use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::components::feuille_form::FeuilleForm;
use crate::components::settings::UserSettings;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "clipboardManager"], catch)]
    async fn writeText(text: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "clipboardManager"], catch)]
    async fn readText() -> Result<JsValue, JsValue>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeuilleData {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeuilleInfo {
    pub uuid: String,
    pub saladier_id: String,
    pub data: FeuilleData,
}

#[derive(Serialize)]
struct ListFeuillesArgs {
    #[serde(rename = "saladierId")]
    saladier_id: String,
}

#[derive(Serialize)]
struct DeleteFeuilleArgs {
    uuid: String,
    #[serde(rename = "saladierPassword")]
    saladier_password: String,
}

#[component]
pub fn SaladierView(
    saladier_uuid: String,
    saladier_name: String,
    on_back: WriteSignal<bool>,
) -> impl IntoView {
    let (feuilles, set_feuilles) = signal(Vec::<FeuilleInfo>::new());
    let (show_form, set_show_form) = signal(false);
    let (editing_feuille, set_editing_feuille) = signal(Option::<FeuilleInfo>::None);
    // Delete Feuille confirmation
    let (delete_target, set_delete_target) = signal(Option::<(String, String)>::None); // (feuille uuid, title)
    let (delete_password, set_delete_password) = signal(String::new());
    let (delete_error, set_delete_error) = signal(String::new());
    let (delete_loading, set_delete_loading) = signal(false);

    // Clipboard auto-clear
    let (clipboard_clear_secs, set_clipboard_clear_secs) = signal(30u32);
    let (copied_field, set_copied_field) = signal(String::new());

    // Load clipboard_clear_seconds from settings
    {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            if let Ok(result) = invoke("get_settings", args).await {
                if let Ok(s) = serde_wasm_bindgen::from_value::<UserSettings>(result) {
                    set_clipboard_clear_secs.set(s.clipboard_clear_seconds);
                }
            }
        });
    }

    let saladier_id = saladier_uuid.clone();
    let saladier_id_for_load = saladier_id.clone();

    let load_feuilles = move || {
        let sid = saladier_id_for_load.clone();
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&ListFeuillesArgs {
                saladier_id: sid,
            })
            .unwrap();
            if let Ok(result) = invoke("list_feuilles", args).await {
                if let Ok(list) = serde_wasm_bindgen::from_value::<Vec<FeuilleInfo>>(result) {
                    set_feuilles.set(list);
                }
            }
        });
    };

    // Initial load
    load_feuilles();

    let saladier_id_for_form = saladier_id.clone();
    let load_feuilles_cb = load_feuilles.clone();
    let load_feuilles_del = load_feuilles.clone();

    let do_delete = {
        let load_feuilles_del = load_feuilles_del.clone();
        move || {
            set_delete_loading.set(true);
            set_delete_error.set(String::new());

            let target = delete_target.get_untracked();
            let pwd = delete_password.get_untracked();
            let load = load_feuilles_del.clone();

            if let Some((feuille_uuid, _title)) = target {
                spawn_local(async move {
                    let args = serde_wasm_bindgen::to_value(&DeleteFeuilleArgs {
                        uuid: feuille_uuid,
                        saladier_password: pwd,
                    })
                    .unwrap();

                    let result = invoke("delete_feuille", args).await;

                    set_delete_loading.set(false);

                    match result {
                        Ok(_) => {
                            set_delete_target.set(None);
                            set_delete_password.set(String::new());
                            load();
                        }
                        Err(_) => {
                            set_delete_error.set("Mot de passe du Saladier incorrect.".to_string());
                        }
                    }
                });
            }
        }
    };

    view! {
        <div class="saladier-view">
            <header class="dashboard-header">
                <div class="header-left">
                    <button class="btn btn-ghost" on:click=move |_| on_back.set(true)>
                        "← Retour"
                    </button>
                    <span class="header-icon">"🥗"</span>
                    <h1>{saladier_name}</h1>
                </div>
                <div class="header-actions">
                    <button class="btn btn-primary btn-sm" on:click=move |_| {
                        set_editing_feuille.set(None);
                        set_show_form.set(true);
                    }>
                        "+ Nouvelle Feuille"
                    </button>
                </div>
            </header>

            // Delete Feuille confirmation modal
            {move || {
                if let Some((_uuid, ref title)) = delete_target.get() {
                    let title_display = title.clone();
                    view! {
                        <div class="modal-overlay">
                            <div class="modal-card">
                                <div class="modal-header">
                                    <h2>"Supprimer la Feuille"</h2>
                                    <p class="modal-subtitle">
                                        "Voulez-vous vraiment supprimer « " {title_display} " » ?"
                                    </p>
                                </div>
                                <form class="auth-form" on:submit={
                                    let do_delete = do_delete.clone();
                                    move |ev: SubmitEvent| {
                                        ev.prevent_default();
                                        do_delete();
                                    }
                                }>
                                    <div class="form-group">
                                        <label>"Mot de passe du Saladier"</label>
                                        <input
                                            type="password"
                                            placeholder="Mot de passe de ce Saladier"
                                            required=true
                                            on:input=move |ev| set_delete_password.set(event_target_value(&ev))
                                        />
                                        <span class="form-hint">"Entrez le mot de passe du Saladier pour confirmer."</span>
                                    </div>
                                    {move || {
                                        let err = delete_error.get();
                                        if err.is_empty() {
                                            view! { <div></div> }.into_any()
                                        } else {
                                            view! { <div class="error-msg">{err}</div> }.into_any()
                                        }
                                    }}
                                    <div class="form-actions">
                                        <button type="button" class="btn btn-ghost" on:click=move |_| {
                                            set_delete_target.set(None);
                                            set_delete_password.set(String::new());
                                            set_delete_error.set(String::new());
                                        }>
                                            "Annuler"
                                        </button>
                                        <button type="submit" class="btn btn-primary btn-danger" disabled=move || delete_loading.get()>
                                            {move || if delete_loading.get() { "Suppression..." } else { "Supprimer" }}
                                        </button>
                                    </div>
                                </form>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            {move || {
                if show_form.get() {
                    let sid = saladier_id_for_form.clone();
                    let editing = editing_feuille.get();
                    let (on_saved, set_on_saved) = signal(false);
                    let (on_cancel, set_on_cancel) = signal(false);

                    let load_feuilles_inner = load_feuilles_cb.clone();
                    Effect::new(move |_| {
                        if on_saved.get() {
                            set_show_form.set(false);
                            load_feuilles_inner();
                        }
                    });

                    Effect::new(move |_| {
                        if on_cancel.get() {
                            set_show_form.set(false);
                        }
                    });

                    view! {
                        <FeuilleForm
                            saladier_id=sid
                            editing=editing
                            on_saved=set_on_saved
                            on_cancel=set_on_cancel
                        />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            <div class="feuilles-list">
                <For
                    each=move || feuilles.get()
                    key=|f| f.uuid.clone()
                    children=move |feuille: FeuilleInfo| {
                        let uuid = feuille.uuid.clone();
                        let uuid_for_delete = uuid.clone();
                        let title_for_delete = feuille.data.title.clone();
                        let data = feuille.data.clone();

                        let username_for_copy = data.username.clone();
                        let password_for_copy = data.password.clone();
                        let feuille_uuid = uuid.clone();
                        let feuille_uuid2 = feuille_uuid.clone();

                        view! {
                            <div class="card feuille-card">
                                <div class="feuille-header">
                                    <h3>{data.title.clone()}</h3>
                                    <div class="feuille-actions">
                                        <button
                                            class="btn btn-ghost btn-sm"
                                            on:click=move |_| {
                                                set_editing_feuille.set(Some(feuille.clone()));
                                                set_show_form.set(true);
                                            }
                                        >
                                            "Modifier"
                                        </button>
                                        <button
                                            class="btn btn-ghost btn-danger btn-sm"
                                            on:click=move |_| {
                                                set_delete_target.set(Some((uuid_for_delete.clone(), title_for_delete.clone())));
                                                set_delete_error.set(String::new());
                                                set_delete_password.set(String::new());
                                            }
                                        >
                                            "Supprimer"
                                        </button>
                                    </div>
                                </div>
                                <div class="feuille-details">
                                    <div class="feuille-field">
                                        <span class="field-label">"Identifiant"</span>
                                        <span class="field-value">{data.username.clone()}</span>
                                        <button
                                            class="btn btn-ghost btn-xs copy-btn"
                                            on:click={
                                                let val = username_for_copy.clone();
                                                let id = feuille_uuid.clone();
                                                move |_| {
                                                    let val = val.clone();
                                                    let id = id.clone();
                                                    let secs = clipboard_clear_secs.get_untracked();
                                                    spawn_local(async move {
                                                        let _ = writeText(&val).await;
                                                        set_copied_field.set(format!("{}-user", id));
                                                        gloo_timers::callback::Timeout::new(2_000, move || {
                                                            set_copied_field.set(String::new());
                                                        }).forget();
                                                        // Schedule clipboard clear (only if still our value)
                                                        let copied_val = val.clone();
                                                        gloo_timers::callback::Timeout::new(secs * 1_000, move || {
                                                            spawn_local(async move {
                                                                if let Ok(current) = readText().await {
                                                                    if current.as_string().as_deref() == Some(&copied_val) {
                                                                        let _ = writeText("").await;
                                                                    }
                                                                }
                                                            });
                                                        }).forget();
                                                    });
                                                }
                                            }
                                        >
                                            {
                                                let id = feuille_uuid.clone();
                                                move || {
                                                    if copied_field.get() == format!("{}-user", id) {
                                                        "Copié !"
                                                    } else {
                                                        "Copier"
                                                    }
                                                }
                                            }
                                        </button>
                                    </div>
                                    <div class="feuille-field">
                                        <span class="field-label">"Mot de passe"</span>
                                        <span class="field-value password-field">"••••••••"</span>
                                        <button
                                            class="btn btn-ghost btn-xs copy-btn"
                                            on:click={
                                                let val = password_for_copy.clone();
                                                let id = feuille_uuid2.clone();
                                                move |_| {
                                                    let val = val.clone();
                                                    let id = id.clone();
                                                    let secs = clipboard_clear_secs.get_untracked();
                                                    spawn_local(async move {
                                                        let _ = writeText(&val).await;
                                                        set_copied_field.set(format!("{}-pwd", id));
                                                        gloo_timers::callback::Timeout::new(2_000, move || {
                                                            set_copied_field.set(String::new());
                                                        }).forget();
                                                        // Schedule clipboard clear (only if still our value)
                                                        let copied_val = val.clone();
                                                        gloo_timers::callback::Timeout::new(secs * 1_000, move || {
                                                            spawn_local(async move {
                                                                if let Ok(current) = readText().await {
                                                                    if current.as_string().as_deref() == Some(&copied_val) {
                                                                        let _ = writeText("").await;
                                                                    }
                                                                }
                                                            });
                                                        }).forget();
                                                    });
                                                }
                                            }
                                        >
                                            {
                                                let id = feuille_uuid2.clone();
                                                move || {
                                                    if copied_field.get() == format!("{}-pwd", id) {
                                                        "Copié !"
                                                    } else {
                                                        "Copier"
                                                    }
                                                }
                                            }
                                        </button>
                                    </div>
                                    {move || {
                                        if !data.url.is_empty() {
                                            view! {
                                                <div class="feuille-field">
                                                    <span class="field-label">"URL"</span>
                                                    <span class="field-value">{data.url.clone()}</span>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    }}
                                    {move || {
                                        if !data.notes.is_empty() {
                                            view! {
                                                <div class="feuille-field">
                                                    <span class="field-label">"Notes"</span>
                                                    <span class="field-value notes">{data.notes.clone()}</span>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    }}
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            {move || {
                if feuilles.get().is_empty() {
                    view! {
                        <div class="empty-state">
                            <p class="empty-icon">"🍃"</p>
                            <p>"Ce Saladier est vide."</p>
                            <p class="empty-hint">"Ajoutez votre première Feuille pour stocker un identifiant."</p>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}
