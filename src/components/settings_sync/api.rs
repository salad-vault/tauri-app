use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use super::types::{DeadmanStatus, SyncStatus};

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
