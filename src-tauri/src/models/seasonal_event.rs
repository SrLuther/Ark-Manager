use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ─────────────────────────────────────────────
// Enums
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Scheduled,
    Active,
    Finished,
    Cancelled,
}

// ─────────────────────────────────────────────
// Structs principais (mapeados do banco)
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SeasonalEvent {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub start_at: String,
    pub end_at: String,
    pub status: String,
    /// Intervalo de broadcast durante o evento (segundos)
    pub broadcast_interval_seconds: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EventRate {
    pub id: u32,
    pub event_id: u32,
    pub xp_multiplier: f64,
    pub harvest_multiplier: f64,
    pub taming_multiplier: f64,
    pub breeding_multiplier: f64,
    pub quality_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EventServer {
    pub id: u32,
    pub event_id: u32,
    pub server_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EventBackup {
    pub id: u32,
    pub event_id: u32,
    pub server_id: u32,
    /// Caminho absoluto do backup do GameUserSettings.ini
    pub gus_backup_path: String,
    /// Caminho absoluto do backup do Game.ini
    pub game_ini_backup_path: String,
    pub created_at: String,
}

// ─────────────────────────────────────────────
// Requests de criação / edição
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventRequest {
    pub name: String,
    pub description: Option<String>,
    /// ISO-8601 ou "YYYY-MM-DD HH:MM:SS"
    pub start_at: String,
    pub end_at: String,
    pub broadcast_interval_seconds: Option<u32>,
    pub rates: CreateEventRatesRequest,
    pub server_ids: Vec<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventRatesRequest {
    pub xp_multiplier: f64,
    pub harvest_multiplier: f64,
    pub taming_multiplier: f64,
    pub breeding_multiplier: f64,
    pub quality_multiplier: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEventRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub broadcast_interval_seconds: Option<u32>,
    pub rates: Option<CreateEventRatesRequest>,
    pub server_ids: Option<Vec<u32>>,
}

// ─────────────────────────────────────────────
// Resposta agregada (evento + rates + servers)
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonalEventFull {
    #[serde(flatten)]
    pub event: SeasonalEvent,
    pub rates: Option<EventRate>,
    pub server_ids: Vec<u32>,
}
