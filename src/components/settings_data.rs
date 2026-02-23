use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SaladierInfo {
    uuid: String,
    name: String,
}

#[component]
pub fn SettingsData() -> impl IntoView {
    let (saladiers, set_saladiers) = signal(Vec::<SaladierInfo>::new());
    let (import_target, set_import_target) = signal(String::new());
    let (import_msg, set_import_msg) = signal(String::new());
    let (import_error, set_import_error) = signal(String::new());
    let (maintenance_msg, set_maintenance_msg) = signal(String::new());
    let (maintenance_loading, set_maintenance_loading) = signal(false);

    // Load saladiers for import target selector
    {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            if let Ok(result) = invoke("list_saladiers", args).await {
                if let Ok(list) = serde_wasm_bindgen::from_value::<Vec<SaladierInfo>>(result) {
                    if let Some(first) = list.first() {
                        set_import_target.set(first.uuid.clone());
                    }
                    set_saladiers.set(list);
                }
            }
        });
    }

    let handle_vacuum = move |_| {
        set_maintenance_loading.set(true);
        set_maintenance_msg.set(String::new());
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let result = invoke("vacuum_database", args).await;
            set_maintenance_loading.set(false);
            match result {
                Ok(_) => {
                    set_maintenance_msg.set("Base de données optimisée avec succès.".to_string());
                }
                Err(err) => {
                    set_maintenance_msg.set(format!("Erreur : {}", err.as_string().unwrap_or_default()));
                }
            }
        });
    };

    let handle_integrity = move |_| {
        set_maintenance_loading.set(true);
        set_maintenance_msg.set(String::new());
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            let result = invoke("check_integrity", args).await;
            set_maintenance_loading.set(false);
            match result {
                Ok(val) => {
                    if let Some(msg) = val.as_string() {
                        set_maintenance_msg.set(format!("Résultat : {msg}"));
                    } else {
                        set_maintenance_msg.set("Vérification terminée.".to_string());
                    }
                }
                Err(err) => {
                    set_maintenance_msg.set(format!("Erreur : {}", err.as_string().unwrap_or_default()));
                }
            }
        });
    };

    let handle_import = move |source: &'static str| {
        let target = import_target.get_untracked();
        if target.is_empty() {
            set_import_error.set("Veuillez sélectionner un Saladier cible.".to_string());
            return;
        }
        set_import_msg.set(String::new());
        set_import_error.set(String::new());

        spawn_local(async move {
            // Open file picker
            #[derive(Serialize)]
            struct ImportArgs {
                #[serde(rename = "saladierUuid")]
                saladier_uuid: String,
                source: String,
            }
            let args = serde_wasm_bindgen::to_value(&ImportArgs {
                saladier_uuid: target,
                source: source.to_string(),
            }).unwrap();
            match invoke("import_passwords", args).await {
                Ok(_) => {
                    set_import_msg.set("Import réussi !".to_string());
                }
                Err(err) => {
                    set_import_error.set(format!("Erreur : {}", err.as_string().unwrap_or_default()));
                }
            }
        });
    };

    let handle_export_json = move |_| {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            match invoke("export_encrypted_json", args).await {
                Ok(_) => {
                    set_maintenance_msg.set("Export JSON chiffré terminé.".to_string());
                }
                Err(err) => {
                    set_maintenance_msg.set(format!("Erreur : {}", err.as_string().unwrap_or_default()));
                }
            }
        });
    };

    let handle_export_csv = move |_| {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            match invoke("export_csv_clear", args).await {
                Ok(_) => {
                    set_maintenance_msg.set("Export CSV terminé.".to_string());
                }
                Err(err) => {
                    set_maintenance_msg.set(format!("Erreur : {}", err.as_string().unwrap_or_default()));
                }
            }
        });
    };

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">"💾 Données & Sauvegardes"</h2>
            <p class="settings-section-desc">"Importez, exportez et maintenez vos données."</p>

            // Import section
            <div class="settings-group">
                <h3>"Importation"</h3>
                <div class="settings-row">
                    <label>"Saladier cible"</label>
                    <select
                        class="settings-select"
                        on:change=move |ev| set_import_target.set(event_target_value(&ev))
                    >
                        {move || {
                            saladiers.get().into_iter().map(|s| {
                                let uuid = s.uuid.clone();
                                view! { <option value={uuid}>{s.name}</option> }
                            }).collect::<Vec<_>>()
                        }}
                    </select>
                </div>
                <div class="import-buttons">
                    <button class="btn btn-ghost btn-sm" on:click={
                        let handle_import = handle_import.clone();
                        move |_| handle_import("bitwarden")
                    }>
                        "📥 Bitwarden (JSON)"
                    </button>
                    <button class="btn btn-ghost btn-sm" on:click={
                        let handle_import = handle_import.clone();
                        move |_| handle_import("keepass")
                    }>
                        "📥 KeePass (XML)"
                    </button>
                    <button class="btn btn-ghost btn-sm" on:click={
                        let handle_import = handle_import.clone();
                        move |_| handle_import("chrome")
                    }>
                        "📥 Chrome (CSV)"
                    </button>
                </div>
                {move || {
                    let msg = import_msg.get();
                    let err = import_error.get();
                    if !msg.is_empty() {
                        view! { <div class="info-msg">{msg}</div> }.into_any()
                    } else if !err.is_empty() {
                        view! { <div class="error-msg">{err}</div> }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>

            // Export section
            <div class="settings-group">
                <h3>"Exportation"</h3>
                <div class="import-buttons">
                    <button class="btn btn-ghost btn-sm" on:click=handle_export_json>
                        "📤 JSON Chiffré"
                    </button>
                    <button class="btn btn-ghost btn-danger btn-sm" on:click=handle_export_csv>
                        "⚠️ CSV Clair"
                    </button>
                </div>
                <p class="settings-hint settings-hint-danger">"L'export CSV clair contient tous vos mots de passe en texte lisible. Utilisez avec précaution."</p>
            </div>

            // Maintenance section
            <div class="settings-group">
                <h3>"Maintenance de la base de données"</h3>
                <div class="import-buttons">
                    <button class="btn btn-ghost btn-sm" on:click=handle_vacuum disabled=move || maintenance_loading.get()>
                        "🔧 Optimiser le stockage"
                    </button>
                    <button class="btn btn-ghost btn-sm" on:click=handle_integrity disabled=move || maintenance_loading.get()>
                        "🔍 Vérifier l'intégrité"
                    </button>
                </div>
                {move || {
                    let msg = maintenance_msg.get();
                    if msg.is_empty() {
                        view! { <div></div> }.into_any()
                    } else {
                        view! { <div class="info-msg">{msg}</div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
