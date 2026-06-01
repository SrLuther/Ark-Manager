//! Modelo de tarefa agendada (Scheduler).
//!
//! Representa tarefas cron configuradas pelo usuário, como restart automático,
//! saveworld, broadcast, destroy wild dinos e atualização de mods.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Tipo de tarefa agendada suportada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "kebab-case")]
#[sqlx(type_name = "VARCHAR", rename_all = "kebab-case")]
pub enum TaskType {
    /// Reinicia o servidor.
    Restart,
    /// Cria um backup automático.
    Backup,
    /// Executa um comando RCON arbitrário.
    #[serde(rename = "rcon-command")]
    RconCommand,
    /// Envia um broadcast para todos os jogadores.
    Announcement,
    /// Executa SaveWorld via RCON.
    #[serde(rename = "save-world")]
    SaveWorld,
    /// Executa DestroyWildDinos via RCON.
    #[serde(rename = "destroy-wild-dinos")]
    DestroyWildDinos,
    /// Verifica e instala atualizações do servidor/mods.
    Update,
}

impl Default for TaskType {
    fn default() -> Self {
        Self::Restart
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Restart => "restart",
            Self::Backup => "backup",
            Self::RconCommand => "rcon-command",
            Self::Announcement => "announcement",
            Self::SaveWorld => "save-world",
            Self::DestroyWildDinos => "destroy-wild-dinos",
            Self::Update => "update",
        };
        write!(f, "{}", s)
    }
}

/// Resultado da última execução da tarefa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
pub enum TaskResult {
    Success,
    Failure,
    Skipped,
}

// ---------------------------------------------------------------------------
// Struct principal — mapeada do banco
// ---------------------------------------------------------------------------

/// Tarefa agendada conforme armazenada no banco.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    pub id: u32,
    pub server_id: u32,
    pub task_name: Option<String>,
    pub task_type: String,
    /// Expressão cron no formato `<sec> <min> <hour> <dom> <month> <dow>`.
    pub cron_expression: String,
    /// Comando RCON (apenas para `TaskType::RconCommand`).
    pub command: Option<String>,
    /// Mensagem de broadcast (apenas para `TaskType::Announcement`).
    pub message: Option<String>,
    /// Minutos de aviso prévio antes da execução (restart, update, etc.).
    pub pre_warning_minutes: u16,
    pub enabled: bool,
    pub run_count: u32,
    pub last_run: Option<NaiveDateTime>,
    pub next_run: Option<NaiveDateTime>,
    pub last_result: Option<String>,
    pub last_error: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl ScheduledTask {
    /// Retorna o tipo de tarefa tipado.
    pub fn task_type_enum(&self) -> TaskType {
        match self.task_type.as_str() {
            "backup" => TaskType::Backup,
            "rcon-command" => TaskType::RconCommand,
            "announcement" => TaskType::Announcement,
            "save-world" => TaskType::SaveWorld,
            "destroy-wild-dinos" => TaskType::DestroyWildDinos,
            "update" => TaskType::Update,
            _ => TaskType::Restart,
        }
    }

    /// Retorna o resultado da última execução tipado.
    pub fn last_result_enum(&self) -> Option<TaskResult> {
        match self.last_result.as_deref() {
            Some("success") => Some(TaskResult::Success),
            Some("failure") => Some(TaskResult::Failure),
            Some("skipped") => Some(TaskResult::Skipped),
            _ => None,
        }
    }

    /// Retorna um nome descritivo para exibição.
    pub fn display_name(&self) -> String {
        if let Some(name) = &self.task_name {
            if !name.is_empty() {
                return name.clone();
            }
        }
        format!("{} (servidor {})", self.task_type, self.server_id)
    }
}

// ---------------------------------------------------------------------------
// Structs de requisição (frontend → backend)
// ---------------------------------------------------------------------------

/// Payload para criação de nova tarefa agendada.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub server_id: u32,
    pub task_name: Option<String>,
    pub task_type: TaskType,
    pub cron_expression: String,
    pub command: Option<String>,
    pub message: Option<String>,
    pub pre_warning_minutes: Option<u16>,
    pub enabled: Option<bool>,
}

/// Payload para atualização parcial de tarefa agendada.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskRequest {
    pub task_name: Option<String>,
    pub task_type: Option<TaskType>,
    pub cron_expression: Option<String>,
    pub command: Option<String>,
    pub message: Option<String>,
    pub pre_warning_minutes: Option<u16>,
    pub enabled: Option<bool>,
}

// ---------------------------------------------------------------------------
// Struct de resposta para o frontend
// ---------------------------------------------------------------------------

/// Resposta de tarefa agendada enviada ao frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTaskResponse {
    pub id: u32,
    pub server_id: u32,
    pub task_name: Option<String>,
    pub display_name: String,
    pub task_type: TaskType,
    pub cron_expression: String,
    pub command: Option<String>,
    pub message: Option<String>,
    pub pre_warning_minutes: u16,
    pub enabled: bool,
    pub run_count: u32,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub last_result: Option<TaskResult>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ScheduledTask> for ScheduledTaskResponse {
    fn from(t: ScheduledTask) -> Self {
        let display_name = t.display_name();
        let task_type = t.task_type_enum();
        let last_result = t.last_result_enum();
        Self {
            id: t.id,
            server_id: t.server_id,
            task_name: t.task_name,
            display_name,
            task_type,
            cron_expression: t.cron_expression,
            command: t.command,
            message: t.message,
            pre_warning_minutes: t.pre_warning_minutes,
            enabled: t.enabled,
            run_count: t.run_count,
            last_run: t.last_run.map(|d| d.to_string()),
            next_run: t.next_run.map(|d| d.to_string()),
            last_result,
            last_error: t.last_error,
            created_at: t.created_at.to_string(),
            updated_at: t.updated_at.to_string(),
        }
    }
}

