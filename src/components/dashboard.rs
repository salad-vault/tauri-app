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
    on_show_docs: Callback<()>,
) -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();

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
        if let Err(msg) = validate_password(&pwd, lang.get_untracked()) {
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
                        set_error_msg.set(t("dash.error_creating_saladier", lang.get_untracked()).to_string());
                    }
                }
                Err(err) => {
                    set_error_msg.set(err.as_string().unwrap_or_else(|| t("dash.error_creating_saladier", lang.get_untracked()).to_string()));
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
                        set_delete_error.set(t("dash.wrong_password", lang.get_untracked()).to_string());
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
                    <h1>{move || t("dash.my_garden", lang.get())}</h1>
                </div>
                <div class="header-actions">
                    <button class="btn btn-ghost" on:click=move |_| on_show_docs.run(())>
                        {move || t("doc.title", lang.get())}
                    </button>
                    <button class="btn btn-ghost" on:click=move |_| on_show_settings.run(())>
                        {move || t("dash.settings", lang.get())}
                    </button>
                    <button class="btn btn-ghost" on:click=move |_| on_show_recovery.run(())>
                        {move || t("dash.recovery_kit", lang.get())}
                    </button>
                    <button class="btn btn-ghost btn-danger" on:click=handle_lock>
                        {move || t("dash.sign_out", lang.get())}
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
                                    <h2>{move || t("dash.sign_out", lang.get())}</h2>
                                    <p class="modal-subtitle">{move || t("dash.logout_confirm", lang.get())}</p>
                                </div>
                                <div class="form-actions">
                                    <button class="btn btn-ghost" on:click=move |_| set_show_logout_confirm.set(false)>
                                        {move || t("cancel", lang.get())}
                                    </button>
                                    <button class="btn btn-primary btn-danger" on:click=confirm_logout>
                                        {move || t("dash.sign_out", lang.get())}
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
                                    <h2>{move || t("dash.delete_saladier", lang.get())}</h2>
                                    <p class="modal-subtitle">
                                        {move || t("dash.delete_confirm_pre", lang.get())} {name_display.clone()} {move || t("dash.delete_confirm_post", lang.get())}
                                    </p>
                                </div>
                                <form class="auth-form" on:submit=handle_delete_submit>
                                    <div class="form-group">
                                        <label>{move || t("dash.account_password", lang.get())}</label>
                                        <input
                                            type="password"
                                            placeholder=move || t("dash.master_pwd_placeholder", lang.get())
                                            required=true
                                            on:input=move |ev| set_delete_password.set(event_target_value(&ev))
                                        />
                                        <span class="form-hint">{move || t("dash.delete_confirm_hint", lang.get())}</span>
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
                                            {move || t("cancel", lang.get())}
                                        </button>
                                        <button type="submit" class="btn btn-primary btn-danger" disabled=move || delete_loading.get()>
                                            {move || if delete_loading.get() { t("deleting", lang.get()) } else { t("delete", lang.get()) }}
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
                        type="password"
                        placeholder=move || t("dash.search_placeholder", lang.get())
                        on:input=move |ev| {
                            set_search_query.set(event_target_value(&ev));
                            set_search_no_result.set(false);
                        }
                        on:keydown=handle_search_keydown
                        prop:value=move || search_query.get()
                    />
                    {move || {
                        if search_loading.get() {
                            view! { <p class="search-no-result">{move || t("dash.searching", lang.get())}</p> }.into_any()
                        } else if search_no_result.get() {
                            view! { <p class="search-no-result">{move || t("dash.no_results", lang.get())}</p> }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </div>

                <div class="section-header">
                    <h2>{move || t("dash.my_saladiers", lang.get())}</h2>
                    <button class="btn btn-primary btn-lg" on:click=move |_| set_show_create.set(true)>
                        {move || t("dash.new_saladier", lang.get())}
                    </button>
                </div>

                {move || {
                    if show_create.get() {
                        view! {
                            <div class="card create-card">
                                <form on:submit=handle_create>
                                    <div class="form-group">
                                        <label>{move || t("dash.saladier_name", lang.get())}</label>
                                        <input
                                            type="text"
                                            placeholder=move || t("dash.saladier_name_placeholder", lang.get())
                                            required=true
                                            on:input=move |ev| set_new_name.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label>{move || t("dash.saladier_password", lang.get())}</label>
                                        <input
                                            type="password"
                                            placeholder=move || t("register.password_placeholder", lang.get())
                                            required=true
                                            on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                        />
                                        <span class="form-hint">{move || t("dash.saladier_pwd_hint", lang.get())}</span>
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
                                            <label for="hidden-checkbox">{move || t("dash.hidden_checkbox", lang.get())}</label>
                                        </div>
                                        <span class="form-hint">{move || t("dash.hidden_hint", lang.get())}</span>
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
                                            {move || t("cancel", lang.get())}
                                        </button>
                                        <button type="submit" class="btn btn-primary" disabled=move || loading.get()>
                                            {move || t("create", lang.get())}
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
                                        {move || t("delete", lang.get())}
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
                                <p>{move || t("dash.empty_title", lang.get())}</p>
                                <p class="empty-hint">{move || t("dash.empty_hint", lang.get())}</p>
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
