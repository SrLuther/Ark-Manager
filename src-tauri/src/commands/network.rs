use crate::services::network;

/// Verifica quais das portas game/query/rcon já estão em uso.
/// Retorna a lista de portas conflitantes.
#[tauri::command]
pub async fn detect_port_conflicts(
    game_port: u16,
    query_port: u16,
    rcon_port: u16,
) -> Result<Vec<u16>, String> {
    Ok(network::detect_port_conflicts(game_port, query_port, rcon_port).await)
}

/// Sugere uma porta disponível a partir de um valor base.
#[tauri::command]
pub async fn suggest_available_port(base: u16) -> Result<u16, String> {
    Ok(network::suggest_available_port(base).await)
}
