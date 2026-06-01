use std::collections::HashSet;
use thiserror::Error;
use tokio::net::TcpListener;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Erro ao verificar porta: {0}")]
    CheckFailed(String),
}

/// Verifica se uma porta TCP está em uso no host local.
pub async fn is_port_in_use(port: u16) -> bool {
    // Tenta fazer bind na porta — se falhar, está em uso
    TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .is_err()
}

/// Dado um conjunto de portas, retorna quais estão em uso.
pub async fn check_ports(ports: &[u16]) -> HashSet<u16> {
    let mut in_use = HashSet::new();
    for &port in ports {
        if is_port_in_use(port).await {
            in_use.insert(port);
        }
    }
    in_use
}

/// Verifica se há conflito entre as portas de um servidor e as portas já em uso.
/// Retorna as portas conflitantes.
pub async fn detect_port_conflicts(game_port: u16, query_port: u16, rcon_port: u16) -> Vec<u16> {
    let ports = [game_port, query_port, rcon_port];
    let in_use = check_ports(&ports).await;
    in_use.into_iter().collect()
}

/// Sugere uma porta disponível a partir de um valor base.
/// Incrementa até encontrar uma porta livre.
pub async fn suggest_available_port(base: u16) -> u16 {
    let mut port = base;
    while is_port_in_use(port).await {
        port = port.saturating_add(1);
        if port == 0 {
            break;
        }
    }
    port
}

