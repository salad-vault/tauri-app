use serde::{Deserialize, Serialize};

/// Incoming request from the browser extension.
#[derive(Deserialize)]
pub struct Request {
    /// Echoed back in the response for request/response matching.
    pub id: Option<String>,
    #[serde(flatten)]
    pub action: Action,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "auth")]
    Auth { token: String },
    #[serde(rename = "pair")]
    Pair { code: String },
    #[serde(rename = "get_status")]
    GetStatus,
    #[serde(rename = "list_saladiers")]
    ListSaladiers,
    #[serde(rename = "search")]
    Search { query: String },
    #[serde(rename = "get_credentials")]
    GetCredentials { feuille_id: String },
}

/// Outgoing response to the browser extension.
#[derive(Serialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Response {
    pub fn ok(id: Option<String>, data: serde_json::Value) -> Self {
        Self { id, ok: true, error: None, data: Some(data) }
    }

    pub fn ok_empty(id: Option<String>) -> Self {
        Self { id, ok: true, error: None, data: None }
    }

    pub fn err(id: Option<String>, msg: impl Into<String>) -> Self {
        Self { id, ok: false, error: Some(msg.into()), data: None }
    }
}
