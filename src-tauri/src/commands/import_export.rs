use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::crypto::xchacha;
use crate::db;
use crate::error::AppError;
use crate::models::feuille::{Feuille, FeuilleData};
use crate::state::AppState;

/// Generic import entry parsed from various sources.
#[derive(Debug, Clone)]
struct ImportEntry {
    title: String,
    username: String,
    password: String,
    url: String,
    notes: String,
}

/// Helper to get the Saladier key from the cache.
fn get_saladier_key(state: &AppState, saladier_id: &str) -> Result<[u8; 32], AppError> {
    let cache = state
        .open_saladiers_cache()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    cache
        .get(saladier_id)
        .copied()
        .ok_or(AppError::SaladierLocked)
}

/// Import passwords from various sources into a specific Saladier.
/// The `source` parameter indicates the format: "bitwarden", "keepass", or "chrome".
/// The `file_data` is the raw file content as a string.
#[tauri::command]
pub async fn import_passwords(
    saladier_uuid: String,
    source: String,
    file_data: String,
    state: State<'_, AppState>,
) -> Result<u32, AppError> {
    let _ = state.require_session()?;
    let k_s = get_saladier_key(&state, &saladier_uuid)?;

    let entries = match source.as_str() {
        "bitwarden" => parse_bitwarden(&file_data)?,
        "keepass" => parse_keepass(&file_data)?,
        "chrome" => parse_chrome(&file_data)?,
        _ => return Err(AppError::Internal(format!("Unknown import source: {source}"))),
    };

    let count = entries.len() as u32;

    let db_lock = state
        .db
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    for entry in entries {
        let feuille_uuid = Uuid::new_v4().to_string();
        let data = FeuilleData {
            title: entry.title,
            username: entry.username,
            password: entry.password,
            url: entry.url,
            notes: entry.notes,
        };

        let json_data = serde_json::to_vec(&data)
            .map_err(|e| AppError::Internal(format!("Serialization error: {e}")))?;
        let (nonce, ciphertext) = xchacha::encrypt(&k_s, &json_data)?;

        let feuille = Feuille {
            uuid: feuille_uuid,
            saladier_id: saladier_uuid.clone(),
            data_blob: ciphertext,
            nonce,
        };

        db::feuilles::create_feuille(&db_lock, &feuille)?;
    }

    Ok(count)
}

/// Parse Bitwarden JSON export format.
fn parse_bitwarden(json_data: &str) -> Result<Vec<ImportEntry>, AppError> {
    #[derive(Deserialize)]
    struct BitwardenExport {
        items: Option<Vec<BitwardenItem>>,
    }

    #[derive(Deserialize)]
    struct BitwardenItem {
        name: Option<String>,
        login: Option<BitwardenLogin>,
        notes: Option<String>,
    }

    #[derive(Deserialize)]
    struct BitwardenLogin {
        username: Option<String>,
        password: Option<String>,
        uris: Option<Vec<BitwardenUri>>,
    }

    #[derive(Deserialize)]
    struct BitwardenUri {
        uri: Option<String>,
    }

    let export: BitwardenExport = serde_json::from_str(json_data)
        .map_err(|e| AppError::Internal(format!("Invalid Bitwarden JSON: {e}")))?;

    let mut entries = Vec::new();
    if let Some(items) = export.items {
        for item in items {
            if let Some(login) = item.login {
                let url = login
                    .uris
                    .and_then(|u| u.first().and_then(|u| u.uri.clone()))
                    .unwrap_or_default();

                entries.push(ImportEntry {
                    title: item.name.unwrap_or_default(),
                    username: login.username.unwrap_or_default(),
                    password: login.password.unwrap_or_default(),
                    url,
                    notes: item.notes.unwrap_or_default(),
                });
            }
        }
    }

    Ok(entries)
}

/// Parse KeePass XML export format.
fn parse_keepass(xml_data: &str) -> Result<Vec<ImportEntry>, AppError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml_data);
    let mut entries = Vec::new();
    let mut in_entry = false;
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut in_key = false;
    let mut in_value = false;

    let mut title = String::new();
    let mut username = String::new();
    let mut password = String::new();
    let mut url = String::new();
    let mut notes = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "Entry" => {
                        in_entry = true;
                        title.clear();
                        username.clear();
                        password.clear();
                        url.clear();
                        notes.clear();
                    }
                    "Key" if in_entry => {
                        in_key = true;
                        current_key.clear();
                    }
                    "Value" if in_entry => {
                        in_value = true;
                        current_value.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "Entry" if in_entry => {
                        in_entry = false;
                        if !title.is_empty() || !username.is_empty() {
                            entries.push(ImportEntry {
                                title: title.clone(),
                                username: username.clone(),
                                password: password.clone(),
                                url: url.clone(),
                                notes: notes.clone(),
                            });
                        }
                    }
                    "Key" => in_key = false,
                    "Value" => {
                        in_value = false;
                        match current_key.as_str() {
                            "Title" => title = current_value.clone(),
                            "UserName" => username = current_value.clone(),
                            "Password" => password = current_value.clone(),
                            "URL" => url = current_value.clone(),
                            "Notes" => notes = current_value.clone(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_key {
                    current_key.push_str(&text);
                } else if in_value {
                    current_value.push_str(&text);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(AppError::Internal(format!("XML parse error: {e}"))),
            _ => {}
        }
    }

    Ok(entries)
}

/// Parse Chrome CSV export format.
fn parse_chrome(csv_data: &str) -> Result<Vec<ImportEntry>, AppError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_data.as_bytes());

    let mut entries = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| AppError::Internal(format!("CSV parse error: {e}")))?;

        // Chrome CSV: name, url, username, password
        let title = record.get(0).unwrap_or("").to_string();
        let url = record.get(1).unwrap_or("").to_string();
        let username = record.get(2).unwrap_or("").to_string();
        let password = record.get(3).unwrap_or("").to_string();

        entries.push(ImportEntry {
            title,
            username,
            password,
            url,
            notes: String::new(),
        });
    }

    Ok(entries)
}

/// Export all Feuilles from a Saladier as encrypted JSON.
#[tauri::command]
pub async fn export_encrypted_json(
    saladier_uuid: String,
    export_password: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let _ = state.require_session()?;
    let k_s = get_saladier_key(&state, &saladier_uuid)?;

    let feuilles = {
        let db_lock = state
            .db
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::feuilles::list_feuilles(&db_lock, &saladier_uuid)?
    };

    let mut items: Vec<FeuilleData> = Vec::new();
    for f in feuilles {
        let json_data = xchacha::decrypt(&k_s, &f.nonce, &f.data_blob)?;
        let data: FeuilleData = serde_json::from_slice(&json_data)
            .map_err(|e| AppError::Internal(format!("Deserialization error: {e}")))?;
        items.push(data);
    }

    // Serialize and encrypt with the export password
    let json = serde_json::to_string_pretty(&items)
        .map_err(|e| AppError::Internal(format!("Serialization error: {e}")))?;

    // Derive a key from the export password using Argon2id
    let salt = crate::crypto::argon2_kdf::generate_salt();
    let key = crate::crypto::argon2_kdf::derive_key(export_password.as_bytes(), &salt)?;

    let (nonce, ciphertext) = xchacha::encrypt(&key, json.as_bytes())?;

    // Pack: salt (32) + nonce (24) + ciphertext
    let mut packed = salt.to_vec();
    packed.extend_from_slice(&nonce);
    packed.extend_from_slice(&ciphertext);

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &packed,
    ))
}

/// Export all Feuilles from a Saladier as clear CSV.
/// Requires master password verification.
#[tauri::command]
pub async fn export_csv_clear(
    saladier_uuid: String,
    master_password: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    // Verify master password first for this dangerous operation
    crate::commands::auth::verify_master_password_inner(&master_password, &state).await?;

    let k_s = get_saladier_key(&state, &saladier_uuid)?;

    let feuilles = {
        let db_lock = state
            .db
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        db::feuilles::list_feuilles(&db_lock, &saladier_uuid)?
    };

    let mut csv_output = String::from("title,url,username,password,notes\n");
    for f in feuilles {
        let json_data = xchacha::decrypt(&k_s, &f.nonce, &f.data_blob)?;
        let data: FeuilleData = serde_json::from_slice(&json_data)
            .map_err(|e| AppError::Internal(format!("Deserialization error: {e}")))?;

        // Escape CSV fields
        csv_output.push_str(&format!(
            "{},{},{},{},{}\n",
            escape_csv(&data.title),
            escape_csv(&data.url),
            escape_csv(&data.username),
            escape_csv(&data.password),
            escape_csv(&data.notes),
        ));
    }

    Ok(csv_output)
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
