//! Comandos Tauri para gerenciamento de agentes remotos ARK Manager.
//!
//! Fluxo de pareamento:
//!   - Modo cliente: `discover_agents` → `pair_agent(address, port, code)` → agente salvo no banco
//!   - Modo servidor: `generate_pairing_code` → exibe código para o outro ARK Manager digitar

use crate::models::agent::{Agent, DiscoveredAgent};
use crate::services::{agent_auth::PairingState, agent_client};
use crate::AppState;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

// ---------------------------------------------------------------------------
// Estado de runtime dos agentes (não persistido)
// ---------------------------------------------------------------------------

/// Estado de runtime gerenciado pelo Tauri: agentes descobertos, sessões WS,
/// pairing state e nome local deste agente.
pub struct AgentRuntimeState {
    pub discovered: Arc<Mutex<Vec<DiscoveredAgent>>>,
    pub sessions: Arc<Mutex<HashMap<String, String>>>,
    pub pairing: Arc<PairingState>,
    pub agent_name: String,
}

impl AgentRuntimeState {
    pub fn new(agent_name: String, pairing: Arc<PairingState>) -> Self {
        Self {
            discovered: Arc::new(Mutex::new(Vec::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            pairing,
            agent_name,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

// ---------------------------------------------------------------------------
// Comandos Tauri
// ---------------------------------------------------------------------------

/// Retorna agentes descobertos via UDP broadcast nesta sessão.
#[tauri::command]
pub async fn discover_agents(
    runtime: State<'_, AgentRuntimeState>,
) -> Result<Vec<DiscoveredAgent>, String> {
    let guard = runtime.discovered.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}

/// Retorna todos os agentes pareados armazenados no banco.
#[tauri::command]
pub async fn list_agents(state: State<'_, AppState>) -> Result<Vec<Agent>, String> {
    sqlx::query_as::<_, Agent>(
        "SELECT id, name, address, port, paired, token_hash, session_token, last_seen_at, created_at
         FROM sync_agents
         ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Realiza o pareamento com um agente remoto usando o código de 6 dígitos.
/// Armazena o agente no banco e retorna o registro criado.
#[tauri::command]
pub async fn pair_agent(
    address: String,
    port: u32,
    code: String,
    state: State<'_, AppState>,
    runtime: State<'_, AgentRuntimeState>,
) -> Result<Agent, String> {
    let resp = agent_client::pair_with_agent(&address, port, &code, &runtime.agent_name).await?;
    let token_hash = sha256_hex(&resp.token);

    // Verifica se já existe um agente com o mesmo endereço
    let existing = sqlx::query_as::<_, Agent>(
        "SELECT id, name, address, port, paired, token_hash, session_token, last_seen_at, created_at
         FROM sync_agents
         WHERE address = ? AND port = ?",
    )
    .bind(&address)
    .bind(port)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(existing_agent) = existing {
        // Atualiza agente existente (salva token original para conexões WS)
        sqlx::query(
            "UPDATE sync_agents
             SET name = ?, token_hash = ?, session_token = ?, paired = 1, last_seen_at = NOW()
             WHERE id = ?",
        )
        .bind(&resp.agent_name)
        .bind(&token_hash)
        .bind(&resp.token)
        .bind(existing_agent.id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

        return sqlx::query_as::<_, Agent>(
            "SELECT id, name, address, port, paired, token_hash, session_token, last_seen_at, created_at
             FROM sync_agents WHERE id = ?",
        )
        .bind(existing_agent.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string());
    }

    // Insere novo agente
    let insert_result = sqlx::query(
        "INSERT INTO sync_agents (name, address, port, paired, token_hash, session_token, last_seen_at, created_at)
         VALUES (?, ?, ?, 1, ?, ?, NOW(), NOW())",
    )
    .bind(&resp.agent_name)
    .bind(&address)
    .bind(port)
    .bind(&token_hash)
    .bind(&resp.token)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let new_id = insert_result.last_insert_id() as u32;

    sqlx::query_as::<_, Agent>(
        "SELECT id, name, address, port, paired, token_hash, session_token, last_seen_at, created_at
         FROM sync_agents WHERE id = ?",
    )
    .bind(new_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Remove um agente pareado pelo ID.
#[tauri::command]
pub async fn remove_agent(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    sqlx::query("DELETE FROM sync_agents WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Verifica se um agente remoto está online via HTTP health check.
#[tauri::command]
pub async fn get_agent_status(address: String, port: u32) -> Result<bool, String> {
    Ok(agent_client::check_agent_health(&address, port).await)
}

/// Gera um código de pareamento de 6 dígitos para este agente.
/// Outro ARK Manager pode usá-lo para se parear com este.
#[tauri::command]
pub async fn generate_pairing_code(
    runtime: State<'_, AgentRuntimeState>,
) -> Result<String, String> {
    Ok(runtime.pairing.generate_new_code())
}
