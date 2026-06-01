use crate::services::rcon::{self, RconConnection};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Mapa de conexões RCON ativas: server_id → RconConnection.
pub type RconMap = Arc<Mutex<HashMap<u32, RconConnection>>>;

/// Cria um novo mapa de conexões RCON.
pub fn new_rcon_map() -> RconMap {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Abre uma conexão RCON com o servidor.
#[tauri::command]
pub async fn rcon_connect(
    server_id: u32,
    host: String,
    port: u16,
    password: String,
    rcon_map: State<'_, RconMap>,
) -> Result<(), String> {
    let conn = RconConnection::connect(&host, port, &password)
        .await
        .map_err(|e| e.to_string())?;

    rcon_map.lock().await.insert(server_id, conn);
    Ok(())
}

/// Envia um comando RCON e retorna a resposta.
#[tauri::command]
pub async fn rcon_send_command(
    server_id: u32,
    command: String,
    rcon_map: State<'_, RconMap>,
) -> Result<String, String> {
    let mut map = rcon_map.lock().await;
    let conn = map
        .get_mut(&server_id)
        .ok_or_else(|| "Sem conexão RCON ativa para este servidor".to_string())?;

    conn.send_command(&command)
        .await
        .map_err(|e| e.to_string())
}

/// Encerra a conexão RCON com o servidor.
#[tauri::command]
pub async fn rcon_disconnect(
    server_id: u32,
    rcon_map: State<'_, RconMap>,
) -> Result<(), String> {
    rcon_map.lock().await.remove(&server_id);
    Ok(())
}

/// Executa um único comando RCON sem manter conexão persistente.
#[tauri::command]
pub async fn rcon_execute(
    host: String,
    port: u16,
    password: String,
    command: String,
) -> Result<String, String> {
    rcon::execute_command(&host, port, &password, &command)
        .await
        .map_err(|e| e.to_string())
}

/// Verifica se há uma conexão RCON ativa para o servidor.
#[tauri::command]
pub async fn rcon_is_connected(
    server_id: u32,
    rcon_map: State<'_, RconMap>,
) -> Result<bool, String> {
    Ok(rcon_map.lock().await.contains_key(&server_id))
}

