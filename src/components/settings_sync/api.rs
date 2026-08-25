use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use super::types::{DeadmanStatus, ServerInfo, SyncStatus};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Invoke a Tauri command with no arguments. Re-exported for the module.
pub(super) async fn invoke_empty(cmd: &str) -> Result<JsValue, JsValue> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    invoke(cmd, args).await
}

/// Invoke a Tauri command with a serializable payload.
pub(super) async fn invoke_with<T: serde::Serialize>(
    cmd: &str,
    payload: &T,
) -> Result<JsValue, JsValue> {
    let args = serde_wasm_bindgen::to_value(payload).unwrap();
    invoke(cmd, args).await
}

/// Fetch the Dead Man's Switch status and populate the given signals.
pub(super) async fn load_deadman_status(
    set_enabled: WriteSignal<bool>,
    set_days: WriteSignal<u32>,
    set_last_seen: WriteSignal<String>,
) {
    if let Ok(result) = invoke_empty("deadman_status").await {
        if let Ok(status) = serde_wasm_bindgen::from_value::<DeadmanStatus>(result) {
            set_enabled.set(status.enabled);
            set_days.set(status.inactivity_days);
            set_last_seen.set(status.last_seen_at);
        }
    }
}

/// Fetch the connected server's URL and capabilities (self-hosted servers may
/// lack SMTP, in which case the Dead Man's Switch cannot deliver its email).
pub(super) async fn load_server_info(
    set_url: WriteSignal<String>,
    set_dm_available: WriteSignal<bool>,
) {
    if let Ok(result) = invoke_empty("server_current_url").await {
        if let Some(url) = result.as_string() {
            set_url.set(url);
        }
    }
    if let Ok(result) = invoke_empty("server_info").await {
        if let Ok(info) = serde_wasm_bindgen::from_value::<ServerInfo>(result) {
            set_dm_available.set(info.deadman_switch_available);
        }
    }
}

/// Fetch the vault sync status and populate the given signals.
pub(super) async fn load_sync_status(
    set_version: WriteSignal<i64>,
    set_updated: WriteSignal<String>,
) {
    if let Ok(result) = invoke_empty("sync_status").await {
        if let Ok(status) = serde_wasm_bindgen::from_value::<SyncStatus>(result) {
            set_version.set(status.version);
            set_updated.set(status.updated_at);
        }
    }
}
