//! Modelo de agente ARK Manager para sincronização em rede local.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Estado de conexão de um agente remoto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentStatus {
    Online,
    Offline,
    Pairing,
}

// ---------------------------------------------------------------------------
// Structs persistidas (banco de dados)
// ---------------------------------------------------------------------------

/// Agente ARK Manager pareado, armazenado no banco.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: u32,
    pub name: String,
    pub address: String,
    pub port: u32,
    pub paired: bool,
    pub token_hash: Option<String>,
    /// Token de sessão em claro — usado para conexões WS subsequentes.
    pub session_token: Option<String>,
    pub last_seen_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

// ---------------------------------------------------------------------------
// Structs de runtime (não persistidas)
// ---------------------------------------------------------------------------

/// Agente recém-descoberto via UDP broadcast (ainda não pareado).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredAgent {
    pub name: String,
    pub address: String,
    pub port: u32,
}

// ---------------------------------------------------------------------------
// Mensagens de protocolo
// ---------------------------------------------------------------------------

/// Payload do broadcast UDP de anúncio de presença.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAnnouncement {
    pub name: String,
    pub port: u32,
    pub version: String,
}

/// Payload enviado ao agente remoto para iniciar pareamento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairRequest {
    pub code: String,
    pub requester_name: String,
}

/// Resposta do agente remoto após pareamento bem-sucedido.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairResponse {
    pub token: String,
    pub agent_name: String,
}
