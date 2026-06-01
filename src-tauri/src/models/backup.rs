//! Modelo de backup de servidor.
//!
//! Representa registros de backup armazenados no banco, junto com
//! as structs de política de backup e payloads para o frontend.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Tipo de backup conforme a razão de sua criação.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "kebab-case")]
#[sqlx(type_name = "VARCHAR", rename_all = "kebab-case")]
pub enum BackupType {
    Manual,
    Auto,
    #[serde(rename = "pre-update")]
    PreUpdate,
    #[serde(rename = "pre-restart")]
    PreRestart,
}

impl Default for BackupType {
    fn default() -> Self {
        Self::Manual
    }
}

impl std::fmt::Display for BackupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Manual => "manual",
            Self::Auto => "auto",
            Self::PreUpdate => "pre-update",
            Self::PreRestart => "pre-restart",
        };
        write!(f, "{}", s)
    }
}

/// Status de execução do backup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
pub enum BackupStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Default for BackupStatus {
    fn default() -> Self {
        Self::Completed
    }
}

// ---------------------------------------------------------------------------
// Struct principal — mapeada do banco
// ---------------------------------------------------------------------------

/// Registro de backup conforme armazenado no banco.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub id: u32,
    pub server_id: u32,
    pub backup_type: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub includes_configs: bool,
    pub includes_mods: bool,
    pub includes_saves: bool,
    pub includes_cluster: bool,
    pub label: Option<String>,
    pub notes: Option<String>,
    pub is_protected: bool,
    pub status: String,
    pub hash: Option<String>,
    pub verified: bool,
    pub compression_level: Option<u8>,
    pub duration_secs: Option<u16>,
    pub created_at: NaiveDateTime,
}

impl Backup {
    /// Retorna o tamanho formatado de forma legível (ex: "1.2 GB").
    pub fn size_human(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size_bytes >= GB {
            format!("{:.2} GB", self.size_bytes as f64 / GB as f64)
        } else if self.size_bytes >= MB {
            format!("{:.1} MB", self.size_bytes as f64 / MB as f64)
        } else if self.size_bytes >= KB {
            format!("{:.0} KB", self.size_bytes as f64 / KB as f64)
        } else {
            format!("{} B", self.size_bytes)
        }
    }

    /// Retorna o tipo de backup tipado.
    pub fn backup_type_enum(&self) -> BackupType {
        match self.backup_type.as_str() {
            "auto" => BackupType::Auto,
            "pre-update" => BackupType::PreUpdate,
            "pre-restart" => BackupType::PreRestart,
            _ => BackupType::Manual,
        }
    }

    /// Retorna o status tipado.
    pub fn backup_status_enum(&self) -> BackupStatus {
        match self.status.as_str() {
            "pending" => BackupStatus::Pending,
            "running" => BackupStatus::Running,
            "failed" => BackupStatus::Failed,
            _ => BackupStatus::Completed,
        }
    }
}

// ---------------------------------------------------------------------------
// Política de backup
// ---------------------------------------------------------------------------

/// Política de backup automático por servidor.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BackupPolicy {
    pub server_id: u32,
    pub enabled: bool,
    pub interval_hours: u16,
    pub retention_days: u16,
    pub retention_count: u16,
    pub storage_quota_gb: f32,
    pub backup_before_update: bool,
    pub backup_before_restart: bool,
    pub compression_enabled: bool,
}

impl Default for BackupPolicy {
    fn default() -> Self {
        Self {
            server_id: 0,
            enabled: false,
            interval_hours: 24,
            retention_days: 7,
            retention_count: 10,
            storage_quota_gb: 50.0,
            backup_before_update: true,
            backup_before_restart: true,
            compression_enabled: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Structs de requisição (frontend → backend)
// ---------------------------------------------------------------------------

/// Payload para criação manual de backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupRequest {
    pub server_id: u32,
    pub label: Option<String>,
    pub notes: Option<String>,
    pub includes_configs: Option<bool>,
    pub includes_mods: Option<bool>,
    pub includes_saves: Option<bool>,
    pub includes_cluster: Option<bool>,
}

/// Payload para restauração de backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreBackupRequest {
    pub backup_id: u32,
    pub stop_server_first: Option<bool>,
}

// ---------------------------------------------------------------------------
// Struct de resposta para o frontend
// ---------------------------------------------------------------------------

/// Resposta de backup enriquecida enviada ao frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResponse {
    pub id: u32,
    pub server_id: u32,
    pub backup_type: BackupType,
    pub file_path: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub includes_configs: bool,
    pub includes_mods: bool,
    pub includes_saves: bool,
    pub includes_cluster: bool,
    pub label: Option<String>,
    pub notes: Option<String>,
    pub is_protected: bool,
    pub status: BackupStatus,
    pub hash: Option<String>,
    pub verified: bool,
    pub duration_secs: Option<u16>,
    pub created_at: String,
}

impl From<Backup> for BackupResponse {
    fn from(b: Backup) -> Self {
        let size_human = b.size_human();
        let backup_type = b.backup_type_enum();
        let status = b.backup_status_enum();
        Self {
            id: b.id,
            server_id: b.server_id,
            backup_type,
            file_path: b.file_path,
            size_bytes: b.size_bytes,
            size_human,
            includes_configs: b.includes_configs,
            includes_mods: b.includes_mods,
            includes_saves: b.includes_saves,
            includes_cluster: b.includes_cluster,
            label: b.label,
            notes: b.notes,
            is_protected: b.is_protected,
            status,
            hash: b.hash,
            verified: b.verified,
            duration_secs: b.duration_secs,
            created_at: b.created_at.to_string(),
        }
    }
}

