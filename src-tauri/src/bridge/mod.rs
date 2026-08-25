mod handler;
mod protocol;

use tauri::AppHandle;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};

pub const BRIDGE_PORT: u16 = 17295;

/// Pairing codes expire this many real seconds after generation (SV-H1).
pub const PAIRING_TTL_SECS: u64 = 60;

/// Whether a WebSocket `Origin` header is acceptable for the bridge (SV-H1).
/// Only browser-extension origins are allowed; a web page's `http(s)://` origin
/// is rejected. A missing `Origin` (non-browser local client) is allowed — the
/// pairing code remains the gate, and browsers always send an `Origin` for WS
/// handshakes, so this still blocks malicious web pages.
fn origin_allowed(origin: Option<&str>) -> bool {
    match origin {
        None => true,
        Some(o) => {
            o.starts_with("chrome-extension://")
                || o.starts_with("moz-extension://")
                || o.starts_with("ms-browser-extension://")
        }
    }
}

/// Handshake callback that rejects WebSocket connections from disallowed origins.
// The `Result<Response, ErrorResponse>` signature is imposed by tungstenite's
// `Callback` trait, so the large Err variant cannot be boxed away here.
#[allow(clippy::result_large_err)]
fn check_handshake_origin(req: &Request, response: Response) -> Result<Response, ErrorResponse> {
    let origin = req.headers().get("Origin").and_then(|v| v.to_str().ok());
    if origin_allowed(origin) {
        Ok(response)
    } else {
        log::warn!("Bridge: rejected WS handshake from a disallowed origin");
        let err = tokio_tungstenite::tungstenite::http::Response::builder()
            .status(403)
            .body(Some("Forbidden origin".to_string()))
            .expect("static 403 response builds");
        Err(err)
    }
}

pub async fn start(app_handle: AppHandle) {
    let addr = format!("127.0.0.1:{BRIDGE_PORT}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Bridge: failed to bind {addr}: {e}");
            return;
        }
    };
    log::info!("Bridge: listening on {addr}");

    loop {
        match listener.accept().await {
            Ok((stream, _peer)) => {
                let handle = app_handle.clone();
                tokio::spawn(async move {
                    match tokio_tungstenite::accept_hdr_async(stream, check_handshake_origin).await {
                        Ok(ws) => handler::handle_connection(ws, handle).await,
                        Err(e) => log::warn!("Bridge: WS handshake rejected/failed: {e}"),
                    }
                });
            }
            Err(e) => log::error!("Bridge: accept error: {e}"),
        }
    }
}
