//! Modelo de servidor ARK: Survival Evolved.
//!
//! Contém todas as structs relacionadas ao ciclo de vida de um servidor,
//! incluindo criação, atualização, status e resposta para o frontend.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Status operacional do servidor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Online,
    Crashed,
    Updating,
    Restarting,
    Stopping,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

impl std::fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Online => "online",
            Self::Crashed => "crashed",
            Self::Updating => "updating",
            Self::Restarting => "restarting",
            Self::Stopping => "stopping",
        };
        write!(f, "{}", s)
    }
}

/// Mapas disponíveis no ARK: Survival Evolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ArkMap {
    TheIsland,
    ScorchedEarth,
    Ragnarok,
    Aberration,
    Extinction,
    Valguero,
    Genesis,
    Genesis2,
    CrystalIsles,
    LostIsland,
    Fjordur,
    Custom(String),
}

impl ArkMap {
    /// Retorna o nome do mapa como usado no parâmetro de lançamento do servidor.
    pub fn as_launch_param(&self) -> &str {
        match self {
            Self::TheIsland => "TheIsland",
            Self::ScorchedEarth => "ScorchedEarth_P",
            Self::Ragnarok => "Ragnarok",
            Self::Aberration => "Aberration_P",
            Self::Extinction => "Extinction",
            Self::Valguero => "Valguero_P",
            Self::Genesis => "Genesis",
            Self::Genesis2 => "Gen2",
            Self::CrystalIsles => "CrystalIsles",
            Self::LostIsland => "LostIsland",
            Self::Fjordur => "Fjordur",
            Self::Custom(name) => name.as_str(),
        }
    }

    /// Tenta criar a partir de uma string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "theisland" => Self::TheIsland,
            "scorchedearth" | "scorchedearth_p" => Self::ScorchedEarth,
            "ragnarok" => Self::Ragnarok,
            "aberration" | "aberration_p" => Self::Aberration,
            "extinction" => Self::Extinction,
            "valguero" | "valguero_p" => Self::Valguero,
            "genesis" => Self::Genesis,
            "gen2" | "genesis2" => Self::Genesis2,
            "crystalisles" => Self::CrystalIsles,
            "lostisland" => Self::LostIsland,
            "fjordur" => Self::Fjordur,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl Default for ArkMap {
    fn default() -> Self {
        Self::TheIsland
    }
}

// ---------------------------------------------------------------------------
// Struct principal — mapeada do banco
// ---------------------------------------------------------------------------

/// Registro completo de um servidor conforme armazenado no banco.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: u32,
    pub name: String,
    pub install_path: String,
    pub map_name: String,
    pub session_name: String,
    pub game_port: u16,
    pub query_port: u16,
    pub rcon_port: u16,
    pub rcon_enabled: bool,
    pub max_players: u16,
    pub server_password: Option<String>,
    pub admin_password: String,
    pub spectator_password: Option<String>,
    pub ip_address: Option<String>,
    pub mods: Option<String>,
    pub cluster_id: Option<u32>,
    pub enable_pvp: bool,
    pub enable_battleye: bool,
    pub enable_crosshair: bool,
    pub allow_third_person: bool,
    pub allow_tribe_alliances: bool,
    pub custom_args: Option<String>,
    pub auto_start: bool,
    pub auto_restart: bool,
    pub startup_delay: u16,
    pub startup_priority: u16,
    pub intelligent_mode: bool,
    pub status: String,
    pub pid: Option<u32>,
    pub last_started: Option<NaiveDateTime>,
    pub last_stopped: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Server {
    /// Retorna a lista de IDs de mods como `Vec<String>`.
    pub fn mod_ids(&self) -> Vec<String> {
        match &self.mods {
            Some(s) if !s.is_empty() => s.split(',').map(|id| id.trim().to_string()).collect(),
            _ => vec![],
        }
    }

    /// Retorna o status tipado.
    pub fn server_status(&self) -> ServerStatus {
        match self.status.as_str() {
            "starting" => ServerStatus::Starting,
            "running" => ServerStatus::Running,
            "online" => ServerStatus::Online,
            "crashed" => ServerStatus::Crashed,
            "updating" => ServerStatus::Updating,
            "restarting" => ServerStatus::Restarting,
            "stopping" => ServerStatus::Stopping,
            _ => ServerStatus::Stopped,
        }
    }

    /// Retorna `true` se o servidor está em um estado "ativo" (processo em execução).
    pub fn is_running(&self) -> bool {
        matches!(
            self.server_status(),
            ServerStatus::Starting
                | ServerStatus::Running
                | ServerStatus::Online
                | ServerStatus::Restarting
                | ServerStatus::Stopping
        )
    }
}

// ---------------------------------------------------------------------------
// Structs de requisição (frontend → backend)
// ---------------------------------------------------------------------------

/// Payload para criação de novo servidor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateServerRequest {
    pub name: String,
    #[serde(rename = "installDir")]
    pub install_path: String,
    #[serde(rename = "map")]
    pub map_name: String,
    #[serde(default)]
    pub session_name: String,
    pub game_port: u16,
    pub query_port: u16,
    pub rcon_port: u16,
    pub rcon_enabled: Option<bool>,
    pub max_players: Option<u16>,
    pub server_password: Option<String>,
    pub admin_password: String,
    pub spectator_password: Option<String>,
    pub ip_address: Option<String>,
    pub mods: Option<String>,
    pub cluster_id: Option<u32>,
    pub enable_pvp: Option<bool>,
    pub enable_battleye: Option<bool>,
    pub enable_crosshair: Option<bool>,
    pub allow_third_person: Option<bool>,
    pub allow_tribe_alliances: Option<bool>,
    pub custom_args: Option<String>,
    pub auto_start: Option<bool>,
    pub auto_restart: Option<bool>,
    pub startup_delay: Option<u16>,
}

/// Payload para atualização parcial de servidor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    #[serde(rename = "installDir")]
    pub install_path: Option<String>,
    #[serde(rename = "map")]
    pub map_name: Option<String>,
    pub session_name: Option<String>,
    pub game_port: Option<u16>,
    pub query_port: Option<u16>,
    pub rcon_port: Option<u16>,
    pub rcon_enabled: Option<bool>,
    pub max_players: Option<u16>,
    pub server_password: Option<String>,
    pub admin_password: Option<String>,
    pub spectator_password: Option<String>,
    pub ip_address: Option<String>,
    pub mods: Option<String>,
    pub cluster_id: Option<u32>,
    pub enable_pvp: Option<bool>,
    pub enable_battleye: Option<bool>,
    pub enable_crosshair: Option<bool>,
    pub allow_third_person: Option<bool>,
    pub allow_tribe_alliances: Option<bool>,
    pub custom_args: Option<String>,
    pub auto_start: Option<bool>,
    pub auto_restart: Option<bool>,
    pub startup_delay: Option<u16>,
    pub startup_priority: Option<u16>,
    pub intelligent_mode: Option<bool>,
}

// ---------------------------------------------------------------------------
// Struct de resposta para o frontend
// ---------------------------------------------------------------------------

/// Resposta enriquecida enviada ao frontend via comando Tauri.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerResponse {
    pub id: u32,
    pub name: String,
    pub install_path: String,
    pub map_name: String,
    pub session_name: String,
    pub game_port: u16,
    pub query_port: u16,
    pub rcon_port: u16,
    pub rcon_enabled: bool,
    pub max_players: u16,
    pub server_password: Option<String>,
    pub admin_password: String,
    pub spectator_password: Option<String>,
    pub ip_address: Option<String>,
    pub mod_ids: Vec<String>,
    pub cluster_id: Option<u32>,
    pub enable_pvp: bool,
    pub enable_battleye: bool,
    pub enable_crosshair: bool,
    pub allow_third_person: bool,
    pub allow_tribe_alliances: bool,
    pub custom_args: Option<String>,
    pub auto_start: bool,
    pub auto_restart: bool,
    pub startup_delay: u16,
    pub status: ServerStatus,
    #[serde(rename = "pidCached")]
    pub pid: Option<u32>,
    pub last_started: Option<String>,
    pub last_stopped: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Server> for ServerResponse {
    fn from(s: Server) -> Self {
        let mod_ids = s.mod_ids();
        let status = s.server_status();
        Self {
            id: s.id,
            name: s.name,
            install_path: s.install_path,
            map_name: s.map_name,
            session_name: s.session_name,
            game_port: s.game_port,
            query_port: s.query_port,
            rcon_port: s.rcon_port,
            rcon_enabled: s.rcon_enabled,
            max_players: s.max_players,
            server_password: s.server_password,
            admin_password: s.admin_password,
            spectator_password: s.spectator_password,
            ip_address: s.ip_address,
            mod_ids,
            cluster_id: s.cluster_id,
            enable_pvp: s.enable_pvp,
            enable_battleye: s.enable_battleye,
            enable_crosshair: s.enable_crosshair,
            allow_third_person: s.allow_third_person,
            allow_tribe_alliances: s.allow_tribe_alliances,
            custom_args: s.custom_args,
            auto_start: s.auto_start,
            auto_restart: s.auto_restart,
            startup_delay: s.startup_delay,
            status,
            pid: s.pid,
            last_started: s.last_started.map(|d| d.to_string()),
            last_stopped: s.last_stopped.map(|d| d.to_string()),
            created_at: s.created_at.to_string(),
            updated_at: s.updated_at.to_string(),
        }
    }
}

