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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaladierInfo {
    pub uuid: String,
    pub name: String,
}

#[derive(Serialize)]
struct CreateSaladierArgs {
    name: String,
    password: String,
    hidden: bool,
}

#[derive(Serialize)]
struct DeleteSaladierArgs {
    uuid: String,
    #[serde(rename = "masterPassword")]
    master_password: String,
}

#[derive(Serialize)]
struct UnlockHiddenArgs {
    password: String,
}

#[component]
pub fn Dashboard(
    on_select_saladier: Callback<(String, String)>,
    on_logout: WriteSignal<bool>,
    on_show_recovery: Callback<()>,
    on_show_settings: Callback<()>,
) -> impl IntoView {
    let (saladiers, set_saladiers) = signal(Vec::<SaladierInfo>::new());
    let (show_create, set_show_create) = signal(false);
    let (new_name, set_new_name) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (new_hidden, set_new_hidden) = signal(false);
    let (error_msg, set_error_msg) = signal(String::new());
    let (loading, set_loading) = signal(false);

    // Logout confirmation
    let (show_logout_confirm, set_show_logout_confirm) = signal(false);

    // Delete Saladier confirmation
    let (delete_target, set_delete_target) = signal(Option::<(String, String)>::None);
    let (delete_password, set_delete_password) = signal(String::new());
    let (delete_error, set_delete_error) = signal(String::new());
    let (delete_loading, set_delete_loading) = signal(false);

    // Search bar
    let (search_query, set_search_query) = signal(String::new());
    let (search_no_result, set_search_no_result) = signal(false);
    let (search_loading, set_search_loading) = signal(false);

    // Load saladiers on mount
    let load_saladiers = move || {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            if let Ok(result) = invoke("list_saladiers", args).await {
                if let Ok(list) = serde_wasm_bindgen::from_value::<Vec<SaladierInfo>>(result) {
                    set_saladiers.set(list);
                }
            }
        });
    };

    // Initial load
    load_saladiers();

    let handle_create = move |ev: SubmitEvent| {
        ev.prevent_default();

        let pwd = new_password.get_untracked();
        if let Err(msg) = validate_password(&pwd) {
            set_error_msg.set(msg);
            return;
        }

        set_loading.set(true);
        set_error_msg.set(String::new());

        let is_hidden = new_hidden.get_untracked();

        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&CreateSaladierArgs {
                name: new_name.get_untracked(),
                password: pwd,
                hidden: is_hidden,
            })
            .unwrap();

            let result = invoke("create_saladier", args).await;

            set_loading.set(false);

            match result {
                Ok(val) => {
                    if let Ok(_info) = serde_wasm_bindgen::from_value::<SaladierInfo>(val) {
                        set_show_create.set(false);
                        set_new_name.set(String::new());
                        set_new_password.set(String::new());
                        set_new_hidden.set(false);
                        // Only reload visible saladiers list if it was visible
                        if !is_hidden {
                            let args2 = serde_wasm_bindgen::to_value(&()).unwrap();
                            if let Ok(result2) = invoke("list_saladiers", args2).await {
                                if let Ok(list) = serde_wasm_bindgen::from_value::<Vec<SaladierInfo>>(result2) {
                                    set_saladiers.set(list);
                                }
                            }
                        }
                    } else {
                        set_error_msg.set("Erreur lors de la création du Saladier.".to_string());
                    }
                }
                Err(err) => {
                    set_error_msg.set(err.as_string().unwrap_or_else(|| "Erreur lors de la création du Saladier.".to_string()));
                }
            }
        });
    };

    let handle_lock = move |_| {
        set_show_logout_confirm.set(true);
    };

    let confirm_logout = move |_| {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let _ = invoke("lock", args).await;
            on_logout.set(true);
        });
    };

    let handle_delete_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_delete_loading.set(true);
        set_delete_error.set(String::new());

        let target = delete_target.get_untracked();
        let pwd = delete_password.get_untracked();

        if let Some((uuid, _name)) = target {
            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&DeleteSaladierArgs {
                    uuid,
                    master_password: pwd,
                })
                .unwrap();

                let result = invoke("delete_saladier", args).await;

                set_delete_loading.set(false);

                match result {
                    Ok(_) => {
                        set_delete_target.set(None);
                        set_delete_password.set(String::new());
                        // Reload
                        let args2 = serde_wasm_bindgen::to_value(&()).unwrap();
                        if let Ok(result2) = invoke("list_saladiers", args2).await {
                            if let Ok(list) = serde_wasm_bindgen::from_value::<Vec<SaladierInfo>>(result2) {
                                set_saladiers.set(list);
                            }
                        }
                    }
                    Err(_) => {
                        set_delete_error.set("Mot de passe incorrect.".to_string());
                    }
                }
            });
        }
    };

    // Search handler: filter visible saladiers by name, and on Enter try hidden unlock
    let handle_search_keydown = move |ev: leptos::web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            let query = search_query.get_untracked();
            if query.is_empty() {
                return;
            }
            set_search_no_result.set(false);
            set_search_loading.set(true);

            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&UnlockHiddenArgs {
                    password: query,
                })
                .unwrap();

                let result = invoke("unlock_hidden_saladier", args).await;

                set_search_loading.set(false);

                match result {
                    Ok(val) => {
                        if let Ok(Some(info)) = serde_wasm_bindgen::from_value::<Option<SaladierInfo>>(val) {
                            // Hidden saladier found: navigate to it directly
                            on_select_saladier.run((info.uuid, info.name));
                            set_search_query.set(String::new());
                        } else {
                            set_search_no_result.set(true);
                        }
                    }
                    Err(_) => {
                        set_search_no_result.set(true);
                    }
                }
            });
        }
    };

    view! {
        <div class="dashboard">
            <header class="dashboard-header">
                <div class="header-left">
                    <span class="header-icon">"🥗"</span>
                    <h1>"Mon Potager"</h1>
                </div>
                <div class="header-actions">
                    <button class="btn btn-ghost" on:click=move |_| on_show_settings.run(())>
                        "⚙️ Paramètres"
                    </button>
                    <button class="btn btn-ghost" on:click=move |_| on_show_recovery.run(())>
                        "Kit de Secours"
                    </button>
                    <button class="btn btn-ghost btn-danger" on:click=handle_lock>
                        "Se déconnecter"
                    </button>
                </div>
            </header>

            // Logout confirmation modal
            {move || {
                if show_logout_confirm.get() {
                    view! {
                        <div class="modal-overlay">
                            <div class="modal-card">
                                <div class="modal-header">
                                    <h2>"Se déconnecter"</h2>
                                    <p class="modal-subtitle">"Êtes-vous sûr de vouloir vous déconnecter ? Vos Saladiers ouverts seront verrouillés."</p>
                                </div>
                                <div class="form-actions">
                                    <button class="btn btn-ghost" on:click=move |_| set_show_logout_confirm.set(false)>
                                        "Annuler"
                                    </button>
                                    <button class="btn btn-primary btn-danger" on:click=confirm_logout>
                                        "Se déconnecter"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            // Delete Saladier confirmation modal
            {move || {
                if let Some((_uuid, ref name)) = delete_target.get() {
                    let name_display = name.clone();
                    view! {
                        <div class="modal-overlay">
                            <div class="modal-card">
                                <div class="modal-header">
                                    <h2>"Supprimer le Saladier"</h2>
                                    <p class="modal-subtitle">
                                        "Voulez-vous vraiment supprimer « " {name_display} " » ? Cette action est irréversible."
                                    </p>
                                </div>
                                <form class="auth-form" on:submit=handle_delete_submit>
                                    <div class="form-group">
                                        <label>"Mot de passe du compte"</label>
                                        <input
                                            type="password"
                                            placeholder="Votre mot de passe maître"
                                            required=true
                                            on:input=move |ev| set_delete_password.set(event_target_value(&ev))
                                        />
                                        <span class="form-hint">"Entrez votre mot de passe maître pour confirmer la suppression."</span>
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

            <main class="dashboard-content">
                // Search bar (also serves as hidden saladier unlock)
                <div class="search-bar-container">
                    <input
                        class="search-bar"
                        type="text"
                        placeholder="Rechercher un Saladier... (Entrée pour déverrouiller)"
                        on:input=move |ev| {
                            set_search_query.set(event_target_value(&ev));
                            set_search_no_result.set(false);
                        }
                        on:keydown=handle_search_keydown
                        prop:value=move || search_query.get()
                    />
                    {move || {
                        if search_loading.get() {
                            view! { <p class="search-no-result">"Recherche..."</p> }.into_any()
                        } else if search_no_result.get() {
                            view! { <p class="search-no-result">"Aucun résultat"</p> }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                <div class="section-header">
                    <h2>"Mes Saladiers"</h2>
                    <button class="btn btn-primary btn-lg" on:click=move |_| set_show_create.set(true)>
                        "+ Nouveau Saladier"
                    </button>
                </div>

                {move || {
                    if show_create.get() {
                        view! {
                            <div class="card create-card">
                                <form on:submit=handle_create>
                                    <div class="form-group">
                                        <label>"Nom du Saladier"</label>
                                        <input
                                            type="text"
                                            placeholder="ex: Personnel, Travail, Crypto..."
                                            required=true
                                            on:input=move |ev| set_new_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label>"Mot de passe du Saladier"</label>
                                        <input
                                            type="password"
                                            placeholder="Min. 16 caractères, maj, min, chiffre, spécial"
                                            required=true
                                            on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                        />
                                        <span class="form-hint">"Ce mot de passe protège ce Saladier même si le Potager est ouvert (Panic Mode). Min. 16 caractères avec majuscule, minuscule, chiffre et caractère spécial."</span>
                                    </div>
                                    <div class="form-group">
                                        <div class="checkbox-group">
                                            <input
                                                type="checkbox"
                                                id="hidden-checkbox"
                                                on:change=move |ev| {
                                                    let checked = event_target_checked(&ev);
                                                    set_new_hidden.set(checked);
                                                }
                                            />
                                            <label for="hidden-checkbox">"Saladier secret (invisible dans la liste)"</label>
                                        </div>
                                        <span class="form-hint">"Un Saladier secret n'apparaît nulle part. Pour y accéder, tapez son mot de passe dans la barre de recherche."</span>
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
                                        <button type="button" class="btn btn-ghost" on:click=move |_| {
                                            set_show_create.set(false);
                                            set_new_hidden.set(false);
                                        }>
                                            "Annuler"
                                        </button>
                                        <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                                            "Créer"
                                        </button>
                                    </div>
                                </form>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}

                <div class="saladier-grid">
                    <For
                        each=move || {
                            let query = search_query.get().to_lowercase();
                            let all = saladiers.get();
                            if query.is_empty() {
                                all
                            } else {
                                all.into_iter()
                                    .filter(|s| s.name.to_lowercase().contains(&query))
                                    .collect()
                            }
                        }
                        key=|s| s.uuid.clone()
                        children=move |saladier: SaladierInfo| {
                            let uuid = saladier.uuid.clone();
                            let name = saladier.name.clone();
                            let uuid_for_click = uuid.clone();
                            let name_for_click = name.clone();
                            let uuid_for_delete = uuid.clone();
                            let name_for_delete = name.clone();
                            view! {
                                <div class="card saladier-card">
                                    <div class="saladier-card-content"
                                         on:click=move |_| {
                                            on_select_saladier.run((uuid_for_click.clone(), name_for_click.clone()));
                                         }
                                    >
                                        <div class="saladier-icon">"🥗"</div>
                                        <h3>{name.clone()}</h3>
                                    </div>
                                    <button
                                        class="btn btn-ghost btn-danger btn-sm"
                                        on:click=move |_| {
                                            set_delete_target.set(Some((uuid_for_delete.clone(), name_for_delete.clone())));
                                            set_delete_error.set(String::new());
                                            set_delete_password.set(String::new());
                                        }
                                    >
                                        "Supprimer"
                                    </button>
                                </div>
                            }
                        }
                    />
                </div>

                {move || {
                    if saladiers.get().is_empty() {
                        view! {
                            <div class="empty-state">
                                <p class="empty-icon">"🌿"</p>
                                <p>"Aucun Saladier pour le moment."</p>
                                <p class="empty-hint">"Créez votre premier Saladier pour commencer à stocker vos mots de passe."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </main>
        </div>
    }
}
