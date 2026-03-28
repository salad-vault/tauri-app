use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::i18n::{t, Language};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI_PLUGIN_DIALOG__"], js_name = open, catch)]
    async fn dialog_open(options: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI_PLUGIN_DIALOG__"], js_name = save, catch)]
    async fn dialog_save(options: JsValue) -> Result<JsValue, JsValue>;
}

// ── Helper: read a file via Tauri FS plugin ──
async fn read_text_file(path: &str) -> Result<String, String> {
    #[derive(Serialize)]
    struct ReadArgs {
        path: String,
    }
    let args = serde_wasm_bindgen::to_value(&ReadArgs {
        path: path.to_string(),
    })
    .map_err(|e| e.to_string())?;
    let result = invoke("read_text_file", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_default())?;
    result
        .as_string()
        .ok_or_else(|| "Failed to read file".to_string())
}

// ── Helper: write a file via Tauri FS plugin ──
async fn write_text_file(path: &str, content: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct WriteArgs {
        path: String,
        content: String,
    }
    let args = serde_wasm_bindgen::to_value(&WriteArgs {
        path: path.to_string(),
        content: content.to_string(),
    })
    .map_err(|e| e.to_string())?;
    invoke("write_text_file", args)
        .await
        .map_err(|e| e.as_string().unwrap_or_default())?;
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SaladierInfo {
    uuid: String,
    name: String,
}

#[component]
pub fn SettingsData() -> impl IntoView {
    let lang = expect_context::<ReadSignal<Language>>();
    let (saladiers, set_saladiers) = signal(Vec::<SaladierInfo>::new());
    let (import_target, set_import_target) = signal(String::new());
    let (import_msg, set_import_msg) = signal(String::new());
    let (import_error, set_import_error) = signal(String::new());
    let (maintenance_msg, set_maintenance_msg) = signal(String::new());
    let (maintenance_loading, set_maintenance_loading) = signal(false);

    // Export modal state
    let (show_export_json_modal, set_show_export_json_modal) = signal(false);
    let (show_export_csv_modal, set_show_export_csv_modal) = signal(false);
    let (export_saladier, set_export_saladier) = signal(String::new());
    let (export_password, set_export_password) = signal(String::new());

    // Load saladiers for import target selector
    {
        spawn_local(async move {
            let args = serde_wasm_bindgen::to_value(&()).unwrap();
            if let Ok(result) = invoke("list_saladiers", args).await {
                if let Ok(list) = serde_wasm_bindgen::from_value::<Vec<SaladierInfo>>(result) {
                    if let Some(first) = list.first() {
                        set_import_target.set(first.uuid.clone());
                        set_export_saladier.set(first.uuid.clone());
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
                    set_maintenance_msg.set(t("data.vacuum_success", lang.get()).to_string());
                }
                Err(err) => {
                    set_maintenance_msg.set(format!("{}: {}", t("data.error_prefix", lang.get()), err.as_string().unwrap_or_default()));
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
                        set_maintenance_msg.set(format!("{}: {msg}", t("data.result_prefix", lang.get())));
                    } else {
                        set_maintenance_msg.set(t("data.integrity_done", lang.get()).to_string());
                    }
                }
                Err(err) => {
                    set_maintenance_msg.set(format!("{}: {}", t("data.error_prefix", lang.get()), err.as_string().unwrap_or_default()));
                }
            }
        });
    };

    // ── Import: open file picker, read file, send to backend ──
    let handle_import = move |source: &'static str| {
        let target = import_target.get_untracked();
        if target.is_empty() {
            set_import_error.set(t("data.select_target", lang.get()).to_string());
            return;
        }
        set_import_msg.set(String::new());
        set_import_error.set(String::new());

        let filter_name = match source {
            "bitwarden" => "Bitwarden JSON",
            "keepass" => "KeePass XML",
            "chrome" => "Chrome CSV",
            _ => "File",
        };
        let extensions: Vec<&str> = match source {
            "bitwarden" => vec!["json"],
            "keepass" => vec!["xml"],
            "chrome" => vec!["csv"],
            _ => vec!["*"],
        };

        spawn_local(async move {
            // Open file picker via Tauri dialog plugin
            let filter = serde_wasm_bindgen::to_value(&serde_json::json!({
                "filters": [{
                    "name": filter_name,
                    "extensions": extensions
                }],
                "multiple": false
            }))
            .unwrap();

            let path = match dialog_open(filter).await {
                Ok(p) => match p.as_string() {
                    Some(path) => path,
                    None => return, // user cancelled
                },
                Err(_) => return,
            };

            // Read the file content
            let file_data = match read_text_file(&path).await {
                Ok(data) => data,
                Err(e) => {
                    set_import_error.set(format!("{}: {e}", t("data.error_prefix", lang.get())));
                    return;
                }
            };

            // Send to backend
            #[derive(Serialize)]
            struct ImportArgs {
                #[serde(rename = "saladierUuid")]
                saladier_uuid: String,
                source: String,
                #[serde(rename = "fileData")]
                file_data: String,
            }
            let args = serde_wasm_bindgen::to_value(&ImportArgs {
                saladier_uuid: target,
                source: source.to_string(),
                file_data,
            })
            .unwrap();
            match invoke("import_passwords", args).await {
                Ok(count_js) => {
                    let count = count_js.as_f64().unwrap_or(0.0) as u32;
                    set_import_msg.set(format!(
                        "{} ({count})",
                        t("data.import_success", lang.get())
                    ));
                }
                Err(err) => {
                    set_import_error.set(format!(
                        "{}: {}",
                        t("data.error_prefix", lang.get()),
                        err.as_string().unwrap_or_default()
                    ));
                }
            }
        });
    };

    // ── Export JSON: ask for saladier + password, encrypt and save ──
    let handle_export_json_confirm = move |_| {
        let sal_uuid = export_saladier.get_untracked();
        let pwd = export_password.get_untracked();
        if sal_uuid.is_empty() || pwd.is_empty() {
            return;
        }
        set_show_export_json_modal.set(false);

        spawn_local(async move {
            // Get encrypted data from backend
            #[derive(Serialize)]
            struct ExportJsonArgs {
                #[serde(rename = "saladierUuid")]
                saladier_uuid: String,
                #[serde(rename = "exportPassword")]
                export_password: String,
            }
            let args = serde_wasm_bindgen::to_value(&ExportJsonArgs {
                saladier_uuid: sal_uuid,
                export_password: pwd,
            })
            .unwrap();

            match invoke("export_encrypted_json", args).await {
                Ok(blob_js) => {
                    if let Some(blob) = blob_js.as_string() {
                        // Save file dialog
                        let save_opts = serde_wasm_bindgen::to_value(&serde_json::json!({
                            "filters": [{"name": "SaladVault Export", "extensions": ["svault"]}],
                            "defaultPath": "saladier_export.svault"
                        }))
                        .unwrap();

                        if let Ok(path_js) = dialog_save(save_opts).await {
                            if let Some(path) = path_js.as_string() {
                                match write_text_file(&path, &blob).await {
                                    Ok(_) => set_maintenance_msg.set(
                                        t("data.export_json_done", lang.get()).to_string(),
                                    ),
                                    Err(e) => set_maintenance_msg.set(format!(
                                        "{}: {e}",
                                        t("data.error_prefix", lang.get())
                                    )),
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    set_maintenance_msg.set(format!(
                        "{}: {}",
                        t("data.error_prefix", lang.get()),
                        err.as_string().unwrap_or_default()
                    ));
                }
            }
            set_export_password.set(String::new());
        });
    };

    // ── Export CSV: ask for saladier + master password, save ──
    let handle_export_csv_confirm = move |_| {
        let sal_uuid = export_saladier.get_untracked();
        let pwd = export_password.get_untracked();
        if sal_uuid.is_empty() || pwd.is_empty() {
            return;
        }
        set_show_export_csv_modal.set(false);

        spawn_local(async move {
            #[derive(Serialize)]
            struct ExportCsvArgs {
                #[serde(rename = "saladierUuid")]
                saladier_uuid: String,
                #[serde(rename = "masterPassword")]
                master_password: String,
            }
            let args = serde_wasm_bindgen::to_value(&ExportCsvArgs {
                saladier_uuid: sal_uuid,
                master_password: pwd,
            })
            .unwrap();

            match invoke("export_csv_clear", args).await {
                Ok(csv_js) => {
                    if let Some(csv_content) = csv_js.as_string() {
                        let save_opts = serde_wasm_bindgen::to_value(&serde_json::json!({
                            "filters": [{"name": "CSV", "extensions": ["csv"]}],
                            "defaultPath": "saladier_export.csv"
                        }))
                        .unwrap();

                        if let Ok(path_js) = dialog_save(save_opts).await {
                            if let Some(path) = path_js.as_string() {
                                match write_text_file(&path, &csv_content).await {
                                    Ok(_) => set_maintenance_msg.set(
                                        t("data.export_csv_done", lang.get()).to_string(),
                                    ),
                                    Err(e) => set_maintenance_msg.set(format!(
                                        "{}: {e}",
                                        t("data.error_prefix", lang.get())
                                    )),
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    set_maintenance_msg.set(format!(
                        "{}: {}",
                        t("data.error_prefix", lang.get()),
                        err.as_string().unwrap_or_default()
                    ));
                }
            }
            set_export_password.set(String::new());
        });
    };

    view! {
        <div class="settings-section">
            <h2 class="settings-section-title">{move || t("data.section_title", lang.get())}</h2>
            <p class="settings-section-desc">{move || t("data.section_desc", lang.get())}</p>

            // Import section
            <div class="settings-group">
                <h3>{move || t("data.import_title", lang.get())}</h3>
                <div class="settings-row">
                    <label>{move || t("data.target_saladier", lang.get())}</label>
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
                        "Bitwarden (JSON)"
                    </button>
                    <button class="btn btn-ghost btn-sm" on:click={
                        let handle_import = handle_import.clone();
                        move |_| handle_import("keepass")
                    }>
                        "KeePass (XML)"
                    </button>
                    <button class="btn btn-ghost btn-sm" on:click={
                        let handle_import = handle_import.clone();
                        move |_| handle_import("chrome")
                    }>
                        "Chrome (CSV)"
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
                <h3>{move || t("data.export_title", lang.get())}</h3>
                <div class="import-buttons">
                    <button
                        class="btn btn-ghost btn-sm"
                        on:click=move |_| {
                            set_export_password.set(String::new());
                            set_show_export_json_modal.set(true);
                        }
                    >
                        {move || t("data.export_encrypted", lang.get())}
                    </button>
                    <button
                        class="btn btn-ghost btn-danger btn-sm"
                        on:click=move |_| {
                            set_export_password.set(String::new());
                            set_show_export_csv_modal.set(true);
                        }
                    >
                        {move || t("data.export_csv", lang.get())}
                    </button>
                </div>
                <p class="settings-hint settings-hint-danger">{move || t("data.export_csv_warn", lang.get())}</p>
            </div>

            // Export JSON modal
            <Show when=move || show_export_json_modal.get()>
                <div class="modal-overlay">
                    <div class="modal-box">
                        <h3>{move || t("data.export_encrypted", lang.get())}</h3>
                        <div class="settings-row">
                            <label>{move || t("data.target_saladier", lang.get())}</label>
                            <select
                                class="settings-select"
                                on:change=move |ev| set_export_saladier.set(event_target_value(&ev))
                            >
                                {move || {
                                    saladiers.get().into_iter().map(|s| {
                                        let uuid = s.uuid.clone();
                                        view! { <option value={uuid}>{s.name}</option> }
                                    }).collect::<Vec<_>>()
                                }}
                            </select>
                        </div>
                        <div class="settings-row">
                            <label>{move || t("data.export_password", lang.get())}</label>
                            <input
                                type="password"
                                class="settings-input"
                                placeholder=move || t("data.export_password_hint", lang.get())
                                prop:value=move || export_password.get()
                                on:input=move |ev| set_export_password.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="import-buttons">
                            <button
                                class="btn btn-primary btn-sm"
                                disabled=move || export_password.get().is_empty()
                                on:click=handle_export_json_confirm
                            >
                                {move || t("data.export_confirm", lang.get())}
                            </button>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| set_show_export_json_modal.set(false)>
                                {move || t("cancel", lang.get())}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Export CSV modal
            <Show when=move || show_export_csv_modal.get()>
                <div class="modal-overlay">
                    <div class="modal-box">
                        <h3>{move || t("data.export_csv", lang.get())}</h3>
                        <p class="settings-hint settings-hint-danger">{move || t("data.export_csv_warn", lang.get())}</p>
                        <div class="settings-row">
                            <label>{move || t("data.target_saladier", lang.get())}</label>
                            <select
                                class="settings-select"
                                on:change=move |ev| set_export_saladier.set(event_target_value(&ev))
                            >
                                {move || {
                                    saladiers.get().into_iter().map(|s| {
                                        let uuid = s.uuid.clone();
                                        view! { <option value={uuid}>{s.name}</option> }
                                    }).collect::<Vec<_>>()
                                }}
                            </select>
                        </div>
                        <div class="settings-row">
                            <label>{move || t("data.master_password", lang.get())}</label>
                            <input
                                type="password"
                                class="settings-input"
                                placeholder=move || t("data.master_password_hint", lang.get())
                                prop:value=move || export_password.get()
                                on:input=move |ev| set_export_password.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="import-buttons">
                            <button
                                class="btn btn-primary btn-danger btn-sm"
                                disabled=move || export_password.get().is_empty()
                                on:click=handle_export_csv_confirm
                            >
                                {move || t("data.export_confirm", lang.get())}
                            </button>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| set_show_export_csv_modal.set(false)>
                                {move || t("cancel", lang.get())}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Maintenance section
            <div class="settings-group">
                <h3>{move || t("data.maintenance", lang.get())}</h3>
                <div class="import-buttons">
                    <button class="btn btn-ghost btn-sm" on:click=handle_vacuum disabled=move || maintenance_loading.get()>
                        {move || t("data.vacuum", lang.get())}
                    </button>
                    <button class="btn btn-ghost btn-sm" on:click=handle_integrity disabled=move || maintenance_loading.get()>
                        {move || t("data.integrity", lang.get())}
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
