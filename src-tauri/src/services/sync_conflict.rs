//! Resolução de conflitos por last-write-wins e registro no banco.

use crate::db::DbPool;

#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    Local,
    Remote,
}

impl Resolution {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
        }
    }
}

/// Decide qual versão prevalece: a com mtime mais alto ganha.
pub fn resolve_last_write_wins(local_mtime: i64, remote_mtime: i64) -> Resolution {
    if local_mtime >= remote_mtime {
        Resolution::Local
    } else {
        Resolution::Remote
    }
}

/// Registra um conflito no banco e incrementa o contador da pasta.
pub async fn record_conflict(
    pool: &DbPool,
    folder_id: u32,
    path: &str,
    local_mtime: i64,
    remote_mtime: i64,
    resolution: &Resolution,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO sync_conflicts (folder_id, path, local_mtime, remote_mtime, resolution, created_at)
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
    )
    .bind(folder_id)
    .bind(path)
    .bind(local_mtime)
    .bind(remote_mtime)
    .bind(resolution.as_str())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE sync_folders SET conflict_count = conflict_count + 1 WHERE id = ?")
        .bind(folder_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Registra um evento genérico no histórico de sincronização.
pub async fn log_sync_event(
    pool: &DbPool,
    folder_id: u32,
    event_type: &str,
    path: Option<&str>,
    bytes: Option<i64>,
    direction: Option<&str>,
    message: Option<&str>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO sync_events (folder_id, event_type, path, bytes, direction, message, created_at)
         VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
    )
    .bind(folder_id)
    .bind(event_type)
    .bind(path)
    .bind(bytes)
    .bind(direction)
    .bind(message)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}
