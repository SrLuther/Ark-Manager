//! Servidor HTTP/WebSocket local do agente ARK Manager.
//!
//! Roda na porta 45678 e expõe:
//! - `GET  /health` → informações do agente (nome, versão)
//! - `POST /pair`   → inicia pareamento com código de 6 dígitos
//! - `GET  /ws?token=<uuid>` → WebSocket para mensagens de sincronização (Fase 9)

use axum::{
    extract::{Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum::extract::ws::{Message, WebSocket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::db::DbPool;
use crate::models::agent::{PairRequest, PairResponse};
use crate::services::{
    agent_auth::PairingState,
    sync_conflict::log_sync_event,
    sync_protocol::SyncMessage,
    sync_reconciler::list_files,
    sync_transfer::{prepare_transfer, FileReceiver},
};

pub const AGENT_SERVER_PORT: u16 = 45678;

// ---------------------------------------------------------------------------
// Estado compartilhado do servidor axum
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AgentServerState {
    pub agent_name: String,
    pub pairing: Arc<PairingState>,
    /// token → nome do peer (sessões WS ativas)
    pub sessions: Arc<Mutex<HashMap<String, String>>>,
    /// Banco de dados — preenchido após inicialização do DB.
    pub db: Arc<tokio::sync::RwLock<Option<DbPool>>>,
}

// ---------------------------------------------------------------------------
// Inicialização do servidor
// ---------------------------------------------------------------------------

/// Inicia o servidor axum em background (non-blocking).
pub async fn start_agent_server(
    agent_name: String,
    pairing: Arc<PairingState>,
    sessions: Arc<Mutex<HashMap<String, String>>>,
    db: Arc<tokio::sync::RwLock<Option<DbPool>>>,
) {
    let state = AgentServerState { agent_name, pairing, sessions, db };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/pair", post(pair_handler))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", AGENT_SERVER_PORT);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Falha ao iniciar servidor de agente em {}: {}", addr, e);
            return;
        }
    };

    log::info!("Servidor de agente iniciado em {}", addr);
    if let Err(e) = axum::serve(listener, app).await {
        log::error!("Servidor de agente encerrado com erro: {}", e);
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    name: String,
    version: &'static str,
}

async fn health_handler(State(s): State<AgentServerState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        name: s.agent_name,
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn pair_handler(
    State(s): State<AgentServerState>,
    Json(req): Json<PairRequest>,
) -> Result<Json<PairResponse>, (StatusCode, String)> {
    if !s.pairing.validate(&req.code) {
        return Err((StatusCode::UNAUTHORIZED, "Código inválido ou expirado".to_string()));
    }

    let token = uuid::Uuid::new_v4().to_string();
    {
        let mut sessions = s.sessions.lock().unwrap();
        sessions.insert(token.clone(), req.requester_name);
    }
    s.pairing.invalidate();

    Ok(Json(PairResponse {
        token,
        agent_name: s.agent_name,
    }))
}

#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsQuery>,
    State(s): State<AgentServerState>,
) -> impl IntoResponse {
    let valid = {
        let guard = s.sessions.lock().unwrap();
        guard.contains_key(&params.token)
    };

    if !valid {
        return (StatusCode::UNAUTHORIZED, "Token inválido").into_response();
    }

    let sessions = s.sessions.clone();
    let db_lock = s.db.clone();
    ws.on_upgrade(move |socket| handle_ws(socket, params.token, sessions, db_lock))
}

/// Handler WebSocket — implementa o protocolo de sincronização completo (Fase 9).
async fn handle_ws(
    mut socket: WebSocket,
    token: String,
    sessions: Arc<Mutex<HashMap<String, String>>>,
    db_lock: Arc<tokio::sync::RwLock<Option<DbPool>>>,
) {
    let short = &token[..8.min(token.len())];
    log::info!("WebSocket conectado: {}", short);

    let pool_opt = db_lock.read().await.clone();
    let pool = match pool_opt {
        Some(p) => p,
        None => {
            let err = SyncMessage::Error { message: "DB não disponível".to_string() };
            let _ = socket.send(Message::Text(err.to_json())).await;
            sessions.lock().unwrap().remove(&token);
            return;
        }
    };

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Some(sync_msg) = SyncMessage::from_json(&text) {
                    if !handle_sync_message(&mut socket, sync_msg, &pool).await {
                        break;
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                let _ = socket.send(Message::Pong(data)).await;
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    log::info!("WebSocket desconectado: {}", short);
    sessions.lock().unwrap().remove(&token);
}

/// Processa uma mensagem do protocolo de sync. Retorna `false` para encerrar a sessão.
async fn handle_sync_message(
    socket: &mut WebSocket,
    msg: SyncMessage,
    pool: &DbPool,
) -> bool {
    match msg {
        // --- Reconciliação: peer solicita nossa lista de arquivos ---
        SyncMessage::ReconcileRequest { folder_id } => {
            let local_path = match get_folder_path(pool, folder_id).await {
                Some(p) => p,
                None => {
                    let err = SyncMessage::Error {
                        message: format!("Pasta {} não encontrada", folder_id),
                    };
                    let _ = socket.send(Message::Text(err.to_json())).await;
                    return true;
                }
            };

            let entries = match list_files(Path::new(&local_path)) {
                Ok(e) => e,
                Err(e) => {
                    let err = SyncMessage::Error {
                        message: format!("Erro ao listar arquivos: {}", e),
                    };
                    let _ = socket.send(Message::Text(err.to_json())).await;
                    return true;
                }
            };

            let files = entries
                .into_iter()
                .map(|e| crate::services::sync_protocol::FileEntry {
                    path: e.path,
                    size: e.size,
                    mtime: e.mtime,
                })
                .collect();
            let resp = SyncMessage::FileList { folder_id, files };
            let _ = socket.send(Message::Text(resp.to_json())).await;
        }

        // --- Peer iniciando transferência de arquivo para nós ---
        SyncMessage::TransferStart { transfer_id, folder_id, path, size, checksum } => {
            let local_path = match get_folder_path(pool, folder_id).await {
                Some(p) => p,
                None => {
                    let ack = SyncMessage::TransferAck {
                        transfer_id,
                        ok: false,
                        error: Some(format!("Pasta {} não encontrada", folder_id)),
                    };
                    let _ = socket.send(Message::Text(ack.to_json())).await;
                    return true;
                }
            };

            let mut receiver =
                match FileReceiver::new(Path::new(&local_path), &path, checksum, size) {
                    Ok(r) => r,
                    Err(e) => {
                        let ack = SyncMessage::TransferAck {
                            transfer_id,
                            ok: false,
                            error: Some(e),
                        };
                        let _ = socket.send(Message::Text(ack.to_json())).await;
                        return true;
                    }
                };

            match receive_chunks(socket, &transfer_id, &mut receiver).await {
                Ok(()) => match receiver.finish() {
                    Ok(()) => {
                        let _ = log_sync_event(
                            pool, folder_id, "transfer",
                            Some(&path), Some(size as i64), Some("download"), None,
                        )
                        .await;
                        let ack = SyncMessage::TransferAck {
                            transfer_id,
                            ok: true,
                            error: None,
                        };
                        let _ = socket.send(Message::Text(ack.to_json())).await;
                    }
                    Err(e) => {
                        let ack = SyncMessage::TransferAck {
                            transfer_id,
                            ok: false,
                            error: Some(e),
                        };
                        let _ = socket.send(Message::Text(ack.to_json())).await;
                    }
                },
                Err(e) => {
                    let ack =
                        SyncMessage::TransferAck { transfer_id, ok: false, error: Some(e) };
                    let _ = socket.send(Message::Text(ack.to_json())).await;
                }
            }
        }

        // --- Peer solicitando arquivos específicos ---
        SyncMessage::RequestFiles { folder_id, paths } => {
            let local_path = match get_folder_path(pool, folder_id).await {
                Some(p) => p,
                None => {
                    let err = SyncMessage::Error {
                        message: format!("Pasta {} não encontrada", folder_id),
                    };
                    let _ = socket.send(Message::Text(err.to_json())).await;
                    return true;
                }
            };

            for rel_path in &paths {
                let abs = Path::new(&local_path)
                    .join(rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
                if !abs.exists() {
                    continue;
                }
                let transfer_id = uuid::Uuid::new_v4().to_string();
                match prepare_transfer(&transfer_id, folder_id, rel_path, &abs) {
                    Ok(msgs) => {
                        for m in msgs {
                            if socket.send(Message::Text(m.to_json())).await.is_err() {
                                return false;
                            }
                        }
                        // Aguardar ACK do peer
                        loop {
                            match socket.recv().await {
                                Some(Ok(Message::Text(t))) => {
                                    if let Some(SyncMessage::TransferAck { .. }) =
                                        SyncMessage::from_json(&t)
                                    {
                                        break;
                                    }
                                }
                                Some(Ok(Message::Ping(d))) => {
                                    let _ = socket.send(Message::Pong(d)).await;
                                }
                                _ => return false,
                            }
                        }
                    }
                    Err(e) => log::warn!("Falha ao preparar {}: {}", rel_path, e),
                }
            }

            let done = SyncMessage::SyncComplete { folder_id };
            let _ = socket.send(Message::Text(done.to_json())).await;
        }

        SyncMessage::Ping => {
            let _ = socket.send(Message::Text(SyncMessage::Pong.to_json())).await;
        }
        SyncMessage::Error { message } => {
            log::warn!("Peer reportou erro via WS: {}", message);
        }
        _ => {}
    }
    true
}

// ---------------------------------------------------------------------------
// Helpers internos
// ---------------------------------------------------------------------------

/// Busca o caminho local de uma pasta pelo ID no banco.
async fn get_folder_path(pool: &DbPool, folder_id: u32) -> Option<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT local_path FROM sync_folders WHERE id = ? LIMIT 1",
    )
    .bind(folder_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// Lê chunks do socket até encontrar `TransferDone` com o mesmo `transfer_id`.
async fn receive_chunks(
    socket: &mut WebSocket,
    expected_id: &str,
    receiver: &mut FileReceiver,
) -> Result<(), String> {
    loop {
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => match SyncMessage::from_json(&text) {
                Some(SyncMessage::TransferChunk { transfer_id, offset, data })
                    if transfer_id == expected_id =>
                {
                    receiver.write_chunk(offset, &data)?;
                }
                Some(SyncMessage::TransferDone { transfer_id })
                    if transfer_id == expected_id =>
                {
                    return Ok(());
                }
                _ => {}
            },
            Some(Ok(Message::Ping(d))) => {
                let _ = socket.send(Message::Pong(d)).await;
            }
            Some(Err(e)) => return Err(e.to_string()),
            None => return Err("Conexão encerrada durante transferência".to_string()),
            _ => {}
        }
    }
}
