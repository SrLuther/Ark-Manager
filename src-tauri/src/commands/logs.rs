use crate::services::log_watcher::{self, LogLine};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{broadcast::Sender, Mutex};

/// Mapa de watchers ativos: server_id → shutdown sender.
pub type WatcherMap = Arc<Mutex<HashMap<u32, Sender<()>>>>;

/// Cria um novo mapa de watchers de log.
pub fn new_watcher_map() -> WatcherMap {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Inicia o watcher de log para o servidor.
/// Emite `log:line` com cada nova linha detectada.
#[tauri::command]
pub async fn start_log_watcher(
    server_id: u32,
    install_dir: String,
    watcher_map: State<'_, WatcherMap>,
    app: AppHandle,
) -> Result<(), String> {
    // Para o watcher anterior se já existir
    {
        let mut map = watcher_map.lock().await;
        if let Some(tx) = map.remove(&server_id) {
            let _ = tx.send(());
        }
    }

    let log_path =
        log_watcher::default_log_path(std::path::Path::new(&install_dir));

    let handle = app.clone();
    let shutdown_tx = log_watcher::start_watcher(
        server_id.to_string(),
        log_path,
        move |line: LogLine| {
            let _ = handle.emit("log:line", &line);
        },
    );

    watcher_map.lock().await.insert(server_id, shutdown_tx);
    Ok(())
}

/// Para o watcher de log do servidor.
#[tauri::command]
pub async fn stop_log_watcher(
    server_id: u32,
    watcher_map: State<'_, WatcherMap>,
) -> Result<(), String> {
    let mut map = watcher_map.lock().await;
    if let Some(tx) = map.remove(&server_id) {
        let _ = tx.send(());
    }
    Ok(())
}

/// Verifica se o watcher de log está ativo para o servidor.
#[tauri::command]
pub async fn is_log_watcher_active(
    server_id: u32,
    watcher_map: State<'_, WatcherMap>,
) -> Result<bool, String> {
    Ok(watcher_map.lock().await.contains_key(&server_id))
}

