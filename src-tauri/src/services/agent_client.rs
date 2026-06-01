//! Cliente HTTP para conectar e se comunicar com agentes remotos ARK Manager.

use crate::models::agent::{PairRequest, PairResponse};
use reqwest::Client;
use std::time::Duration;

/// Timeout padrão para requisições ao agente remoto.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

fn build_client() -> Result<Client, String> {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())
}

/// Verifica se um agente remoto está acessível via `/health`.
pub async fn check_agent_health(address: &str, port: u32) -> bool {
    let Ok(client) = build_client() else { return false };

    client
        .get(format!("http://{}:{}/health", address, port))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Envia requisição de pareamento ao agente remoto.
///
/// Retorna `PairResponse` com o token de sessão em caso de sucesso.
pub async fn pair_with_agent(
    address: &str,
    port: u32,
    code: &str,
    our_name: &str,
) -> Result<PairResponse, String> {
    let client = build_client()?;

    let req_body = PairRequest {
        code: code.to_string(),
        requester_name: our_name.to_string(),
    };

    let resp = client
        .post(format!("http://{}:{}/pair", address, port))
        .json(&req_body)
        .send()
        .await
        .map_err(|e| format!("Erro de conexão com agente: {}", e))?;

    if resp.status().is_success() {
        resp.json::<PairResponse>()
            .await
            .map_err(|e| format!("Erro ao parsear resposta de pareamento: {}", e))
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(format!("Agente recusou o pareamento ({}): {}", status, body))
    }
}
