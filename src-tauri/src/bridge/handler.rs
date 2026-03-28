use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tauri::{AppHandle, Manager};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use crate::crypto::xchacha;
use crate::db;
use crate::state::AppState;

use super::protocol::{Action, Request, Response};

/// Handle a single authenticated WebSocket connection.
pub async fn handle_connection(ws: WebSocketStream<TcpStream>, app: AppHandle) {
    let (mut sink, mut stream) = ws.split();
    let mut authenticated = false;

    while let Some(Ok(msg)) = stream.next().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let req: Request = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::err(None, format!("Invalid JSON: {e}"));
                let _ = sink.send(Message::Text(serde_json::to_string(&resp).unwrap())).await;
                continue;
            }
        };

        let id = req.id.clone();

        // Before auth, only allow auth/pair/get_status
        if !authenticated {
            match &req.action {
                Action::Auth { token } => {
                    let state = app.state::<AppState>();
                    let stored = state.bridge_token.lock().ok()
                        .and_then(|t| t.clone());
                    if stored.as_deref() == Some(token.as_str()) {
                        authenticated = true;
                        let resp = Response::ok_empty(id);
                        let _ = sink.send(Message::Text(serde_json::to_string(&resp).unwrap())).await;
                    } else {
                        let resp = Response::err(id, "Invalid token");
                        let _ = sink.send(Message::Text(serde_json::to_string(&resp).unwrap())).await;
                    }
                    continue;
                }
                Action::Pair { code } => {
                    let state = app.state::<AppState>();
                    let stored_code = state.bridge_pairing_code.lock().ok()
                        .and_then(|c| c.clone());
                    if stored_code.as_deref() == Some(code.as_str()) {
                        // Generate persistent token
                        let token = generate_token();
                        {
                            let mut t = state.bridge_token.lock().unwrap();
                            *t = Some(token.clone());
                        }
                        // Clear pairing code
                        {
                            let mut c = state.bridge_pairing_code.lock().unwrap();
                            *c = None;
                        }
                        // Persist token to DB
                        if let Ok(conn) = state.db.lock() {
                            let _ = db::bridge::set_bridge_token(&conn, &token);
                        }
                        authenticated = true;
                        let resp = Response::ok(id, json!({ "token": token }));
                        let _ = sink.send(Message::Text(serde_json::to_string(&resp).unwrap())).await;
                    } else {
                        let resp = Response::err(id, "Invalid pairing code");
                        let _ = sink.send(Message::Text(serde_json::to_string(&resp).unwrap())).await;
                    }
                    continue;
                }
                Action::GetStatus => {
                    let state = app.state::<AppState>();
                    let unlocked = state.require_session().is_ok();
                    let resp = Response::ok(id, json!({ "unlocked": unlocked }));
                    let _ = sink.send(Message::Text(serde_json::to_string(&resp).unwrap())).await;
                    continue;
                }
                _ => {
                    let resp = Response::err(id, "Not authenticated");
                    let _ = sink.send(Message::Text(serde_json::to_string(&resp).unwrap())).await;
                    continue;
                }
            }
        }

        // Authenticated — process all actions
        let resp = handle_action(&app, id.clone(), &req.action);
        let _ = sink.send(Message::Text(serde_json::to_string(&resp).unwrap())).await;
    }
}

fn handle_action(app: &AppHandle, id: Option<String>, action: &Action) -> Response {
    let state = app.state::<AppState>();

    // Check session for all data requests
    let (user_id, master_key) = match state.require_session() {
        Ok(s) => s,
        Err(_) => return Response::err(id, "locked"),
    };

    match action {
        Action::GetStatus => {
            Response::ok(id, json!({ "unlocked": true }))
        }

        Action::ListSaladiers => {
            let conn = match state.db.lock() {
                Ok(c) => c,
                Err(e) => return Response::err(id, e.to_string()),
            };
            match db::saladiers::list_all_saladiers(&conn, &user_id) {
                Ok(saladiers) => {
                    let items: Vec<serde_json::Value> = saladiers.iter().map(|s| {
                        let name = xchacha::decrypt(&master_key, &s.nonce, &s.name_enc)
                            .ok()
                            .and_then(|b| String::from_utf8(b).ok())
                            .unwrap_or_else(|| "[encrypted]".to_string());
                        json!({
                            "uuid": s.uuid,
                            "name": name,
                            "hidden": s.hidden,
                        })
                    }).collect();
                    Response::ok(id, json!(items))
                }
                Err(e) => Response::err(id, format!("{e}")),
            }
        }

        Action::Search { query } => {
            let conn = match state.db.lock() {
                Ok(c) => c,
                Err(e) => return Response::err(id, e.to_string()),
            };
            let cache = match state.open_saladiers_cache() {
                Ok(c) => c,
                Err(e) => return Response::err(id, e),
            };

            let query_lower = query.to_lowercase();
            let mut results = Vec::new();

            for (sal_uuid, sal_key) in cache.iter() {
                let feuilles = match db::feuilles::list_feuilles(&conn, sal_uuid) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                for f in feuilles {
                    if let Ok(json_bytes) = xchacha::decrypt(sal_key, &f.nonce, &f.data_blob) {
                        if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&json_bytes) {
                            let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("");
                            let url = data.get("url").and_then(|v| v.as_str()).unwrap_or("");
                            let username = data.get("username").and_then(|v| v.as_str()).unwrap_or("");

                            if title.to_lowercase().contains(&query_lower)
                                || url.to_lowercase().contains(&query_lower)
                                || username.to_lowercase().contains(&query_lower)
                            {
                                results.push(json!({
                                    "feuille_id": f.uuid,
                                    "saladier_id": sal_uuid,
                                    "title": title,
                                    "username": username,
                                    "url": url,
                                }));
                            }
                        }
                    }
                }
            }
            Response::ok(id, json!(results))
        }

        Action::GetCredentials { feuille_id } => {
            let conn = match state.db.lock() {
                Ok(c) => c,
                Err(e) => return Response::err(id, e.to_string()),
            };
            let cache = match state.open_saladiers_cache() {
                Ok(c) => c,
                Err(e) => return Response::err(id, e),
            };

            // Find the feuille and its saladier key
            match db::feuilles::get_feuille(&conn, feuille_id) {
                Ok(f) => {
                    if let Some(sal_key) = cache.get(&f.saladier_id) {
                        match xchacha::decrypt(sal_key, &f.nonce, &f.data_blob) {
                            Ok(json_bytes) => {
                                if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&json_bytes) {
                                    Response::ok(id, data)
                                } else {
                                    Response::err(id, "Decryption parse error")
                                }
                            }
                            Err(_) => Response::err(id, "Decryption failed"),
                        }
                    } else {
                        Response::err(id, "Saladier not unlocked")
                    }
                }
                Err(e) => Response::err(id, format!("{e}")),
            }
        }

        // Auth/Pair already handled in the unauthenticated phase
        Action::Auth { .. } | Action::Pair { .. } => {
            Response::ok_empty(id)
        }
    }
}

fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
