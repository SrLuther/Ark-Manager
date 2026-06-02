//! Motor de sincronização — orquestra reconciliação e watch de pastas.
//!
//! Responsabilidades:
//! - Manter watchers ativos por folder_id.
//! - Conectar ao WebSocket do peer e executar reconciliação bidirecional.
//! - Iniciar tarefas de reconciliação periódica por pasta.
//! - Reconectar automaticamente quando peer voltar online.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

use crate::db::DbPool;
use crate::services::{
    agent_client::check_agent_health,
    sync_conflict::log_sync_event,
    sync_protocol::{FileEntry, SyncMessage},
    sync_reconciler::{compute_diff, list_files},
    sync_transfer::{prepare_transfer, FileReceiver},
    sync_watcher::FolderWatcher,
};

/// Intervalo padrão de reconciliação periódica: 5 minutos.
const RECONCILE_INTERVAL: Duration = Duration::from_secs(300);

// ---------------------------------------------------------------------------
// Estado do motor
// ---------------------------------------------------------------------------

/// Estado gerenciado pelo Tauri — mantém watchers ativos por pasta.
pub struct SyncEngineState {
    pub watchers: Arc<Mutex<HashMap<u32, FolderWatcher>>>,
}

impl SyncEngineState {
    pub fn new() -> Self {
        Self { watchers: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Inicia (ou reinicia) o watcher para a pasta `folder_id`.
    pub fn start_watching(&self, folder_id: u32, path: &Path) -> Result<(), String> {
        let watcher = FolderWatcher::watch(path)?;
        self.watchers.lock().unwrap().insert(folder_id, watcher);
        Ok(())
    }

    /// Para o watcher da pasta `folder_id`.
    pub fn stop_watching(&self, folder_id: u32) {
        self.watchers.lock().unwrap().remove(&folder_id);
    }

    /// Retorna `true` se houver eventos pendentes no watcher da pasta.
    pub fn has_pending_changes(&self, folder_id: u32) -> bool {
        let guard = self.watchers.lock().unwrap();
        if let Some(watcher) = guard.get(&folder_id) {
            let events_arc = watcher.events.clone();
            drop(guard);
            let guard2 = events_arc.lock().unwrap();
            return !guard2.is_empty();
        }
        false
    }

    /// Drena os eventos pendentes do watcher da pasta.
    pub fn drain_changes(&self, folder_id: u32) {
        let guard = self.watchers.lock().unwrap();
        if let Some(watcher) = guard.get(&folder_id) {
            let events_arc = watcher.events.clone();
            drop(guard);
            let mut guard2 = events_arc.lock().unwrap();
            guard2.clear();
        }
    }
}

// ---------------------------------------------------------------------------
// Sincronização bidirecional com peer (outbound)
// ---------------------------------------------------------------------------

/// Conecta ao WebSocket do peer e executa reconciliação bidirecional completa.
pub async fn sync_folder_with_peer(
    pool: &DbPool,
    folder_id: u32,
    local_path: &str,
    agent_address: &str,
    agent_port: u32,
    session_token: &str,
) -> Result<(), String> {
    let url = format!("ws://{}:{}/ws?token={}", agent_address, agent_port, session_token);
    let (ws_stream, _) = connect_async(&url)
        .await
        .map_err(|e| format!("Falha ao conectar ao peer {}:{}: {}", agent_address, agent_port, e))?;
    let (mut writer, mut reader) = ws_stream.split();

    // 1. Solicitar lista de arquivos do peer
    let req = SyncMessage::ReconcileRequest { folder_id };
    writer
        .send(WsMessage::Text(req.to_json()))
        .await
        .map_err(|e| format!("Erro ao enviar ReconcileRequest: {}", e))?;

    // 2. Aguardar FileList
    let remote_files: Vec<FileEntry> = loop {
        match reader.next().await {
            Some(Ok(WsMessage::Text(text))) => {
                if let Some(SyncMessage::FileList { files, .. }) = SyncMessage::from_json(&text) {
                    break files;
                }
            }
            Some(Err(e)) => return Err(format!("Erro no WS aguardando FileList: {}", e)),
            None => return Err("Conexão fechada antes de receber FileList".to_string()),
            _ => {}
        }
    };

    // 3. Listar arquivos locais e calcular diff
    let local_files =
        list_files(Path::new(local_path)).map_err(|e| format!("Erro ao listar arquivos: {}", e))?;
    let diff = compute_diff(&local_files, &remote_files);

    // 4. Enviar arquivos que o peer precisa
    let mut total_bytes_sent: i64 = 0;
    for file_entry in &diff.to_send {
        let abs = Path::new(local_path)
            .join(file_entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !abs.exists() {
            continue;
        }
        let transfer_id = Uuid::new_v4().to_string();
        match prepare_transfer(&transfer_id, folder_id, &file_entry.path, &abs) {
            Ok(msgs) => {
                for msg in &msgs {
                    if let SyncMessage::TransferChunk { data, .. } = msg {
                        // approx: base64 overhead é 4/3
                        total_bytes_sent += (data.len() * 3 / 4) as i64;
                    }
                    writer
                        .send(WsMessage::Text(msg.to_json()))
                        .await
                        .map_err(|e| format!("Erro ao enviar mensagem: {}", e))?;
                }
                // Aguardar TransferAck
                loop {
                    match reader.next().await {
                        Some(Ok(WsMessage::Text(text))) => {
                            if let Some(SyncMessage::TransferAck { ok, error, .. }) =
                                SyncMessage::from_json(&text)
                            {
                                if !ok {
                                    log::warn!(
                                        "Peer rejeitou {}: {:?}",
                                        file_entry.path,
                                        error
                                    );
                                } else {
                                    let _ = log_sync_event(
                                        pool, folder_id, "transfer",
                                        Some(&file_entry.path), Some(file_entry.size as i64),
                                        Some("upload"), None,
                                    )
                                    .await;
                                }
                                break;
                            }
                        }
                        Some(Err(e)) => return Err(format!("Erro WS aguardando ACK: {}", e)),
                        None => return Err("Conexão fechada aguardando ACK".to_string()),
                        _ => {}
                    }
                }
            }
            Err(e) => log::warn!("Falha ao preparar {} para envio: {}", file_entry.path, e),
        }
    }

    // 5. Solicitar arquivos do peer
    if !diff.to_request.is_empty() {
        let req_msg = SyncMessage::RequestFiles {
            folder_id,
            paths: diff.to_request.clone(),
        };
        writer
            .send(WsMessage::Text(req_msg.to_json()))
            .await
            .map_err(|e| format!("Erro ao enviar RequestFiles: {}", e))?;

        // 6. Receber arquivos do peer
        let mut pending: HashMap<String, FileReceiver> = HashMap::new();
        let mut expected = diff.to_request.len();
        let mut received = 0usize;

        while received < expected {
            match reader.next().await {
                Some(Ok(WsMessage::Text(text))) => match SyncMessage::from_json(&text) {
                    Some(SyncMessage::TransferStart { transfer_id, path, size, checksum, .. }) => {
                        match FileReceiver::new(Path::new(local_path), &path, checksum, size) {
                            Ok(r) => { pending.insert(transfer_id, r); }
                            Err(e) => {
                                log::warn!("Falha ao criar receptor para {}: {}", path, e);
                                expected = expected.saturating_sub(1);
                            }
                        }
                    }
                    Some(SyncMessage::TransferChunk { transfer_id, offset, data }) => {
                        if let Some(r) = pending.get_mut(&transfer_id) {
                            if let Err(e) = r.write_chunk(offset, &data) {
                                log::warn!("Erro ao escrever chunk {}: {}", transfer_id, e);
                            }
                        }
                    }
                    Some(SyncMessage::TransferDone { transfer_id }) => {
                        if let Some(r) = pending.remove(&transfer_id) {
                            let path_str = r.dest_path.to_string_lossy().to_string();
                            let ack = match r.finish() {
                                Ok(()) => {
                                    let _ = log_sync_event(
                                        pool, folder_id, "transfer",
                                        Some(&path_str), None, Some("download"), None,
                                    )
                                    .await;
                                    SyncMessage::TransferAck {
                                        transfer_id: transfer_id.clone(),
                                        ok: true,
                                        error: None,
                                    }
                                }
                                Err(e) => {
                                    log::warn!("Checksum falhou para {}: {}", path_str, e);
                                    SyncMessage::TransferAck {
                                        transfer_id: transfer_id.clone(),
                                        ok: false,
                                        error: Some(e),
                                    }
                                }
                            };
                            writer.send(WsMessage::Text(ack.to_json())).await.ok();
                            received += 1;
                        }
                    }
                    Some(SyncMessage::SyncComplete { .. }) => break,
                    Some(SyncMessage::Error { message }) => {
                        return Err(format!("Erro do peer durante sync: {}", message));
                    }
                    _ => {}
                },
                Some(Err(e)) => return Err(format!("Erro WS recebendo arquivos: {}", e)),
                None => break,
                _ => {}
            }
        }
    }

    // 7. Atualizar pasta no banco
    sqlx::query(
        "UPDATE sync_folders
         SET status = 'synced', last_sync_at = CURRENT_TIMESTAMP,
             bytes_transferred = bytes_transferred + ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(total_bytes_sent)
    .bind(folder_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let _ = log_sync_event(pool, folder_id, "sync_complete", None, None, None, None).await;

    writer.send(WsMessage::Close(None)).await.ok();
    Ok(())
}

// ---------------------------------------------------------------------------
// Sincronização periódica em background
// ---------------------------------------------------------------------------

/// Inicia uma tarefa tokio de reconciliação periódica para uma pasta.
/// Reconecta automaticamente quando o peer voltar online (9.10).
pub fn start_periodic_sync(
    engine: Arc<SyncEngineState>,
    pool: DbPool,
    folder_id: u32,
    local_path: String,
    agent_address: String,
    agent_port: u32,
    session_token: String,
) {
    tokio::spawn(async move {
        let mut was_offline = false;
        loop {
            let online = check_agent_health(&agent_address, agent_port).await;

            let should_sync = if online && was_offline {
                // Peer voltou online: reconciliar imediatamente (9.10)
                log::info!("Peer {}:{} voltou online — sincronizando pasta {}", agent_address, agent_port, folder_id);
                was_offline = false;
                true
            } else if online {
                // Verificar se há mudanças pendentes no watcher
                if engine.has_pending_changes(folder_id) {
                    engine.drain_changes(folder_id);
                    true
                } else {
                    false
                }
            } else {
                if !was_offline {
                    log::info!("Peer {}:{} offline — pasta {} em modo offline", agent_address, agent_port, folder_id);
                    let _ = sqlx::query(
                        "UPDATE sync_folders SET status = 'offline', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                    )
                    .bind(folder_id)
                    .execute(&pool)
                    .await;
                }
                was_offline = true;
                false
            };

            if should_sync {
                log::info!("Sincronizando pasta {} com {}:{}...", folder_id, agent_address, agent_port);
                let _ = sqlx::query(
                    "UPDATE sync_folders SET status = 'syncing', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                )
                .bind(folder_id)
                .execute(&pool)
                .await;

                match sync_folder_with_peer(
                    &pool, folder_id, &local_path,
                    &agent_address, agent_port, &session_token,
                )
                .await
                {
                    Ok(()) => log::info!("Pasta {} sincronizada com sucesso.", folder_id),
                    Err(e) => {
                        log::warn!("Sync falhou para pasta {}: {}", folder_id, e);
                        let _ = sqlx::query(
                            "UPDATE sync_folders SET status = 'error', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                        )
                        .bind(folder_id)
                        .execute(&pool)
                        .await;
                    }
                }
            }

            tokio::time::sleep(RECONCILE_INTERVAL).await;
        }
    });
}
