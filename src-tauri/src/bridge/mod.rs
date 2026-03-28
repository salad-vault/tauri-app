mod handler;
mod protocol;

use tauri::AppHandle;
use tokio::net::TcpListener;

pub const BRIDGE_PORT: u16 = 17295;

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
                    match tokio_tungstenite::accept_async(stream).await {
                        Ok(ws) => handler::handle_connection(ws, handle).await,
                        Err(e) => log::warn!("Bridge: WS handshake failed: {e}"),
                    }
                });
            }
            Err(e) => log::error!("Bridge: accept error: {e}"),
        }
    }
}
