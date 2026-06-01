//! Modelos de sincronização de pastas entre agentes ARK Manager.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Pasta local configurada para sincronização com um agente peer.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncFolder {
    pub id: u32,
    pub name: String,
    pub local_path: String,
    pub agent_id: Option<u32>,
    pub status: String,
    pub last_sync_at: Option<NaiveDateTime>,
    pub bytes_transferred: i64,
    pub conflict_count: u32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Evento de sincronização registrado para auditoria/histórico.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncEvent {
    pub id: u32,
    pub folder_id: u32,
    /// Tipo: "transfer", "conflict", "error", "connected", "disconnected", "sync_complete"
    pub event_type: String,
    pub path: Option<String>,
    pub bytes: Option<i64>,
    /// Direção: "upload", "download"
    pub direction: Option<String>,
    pub message: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Conflito de sincronização resolvido por last-write-wins.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflict {
    pub id: u32,
    pub folder_id: u32,
    pub path: String,
    pub local_mtime: i64,
    pub remote_mtime: i64,
    /// Resolução: "local" ou "remote"
    pub resolution: String,
    pub created_at: NaiveDateTime,
}
