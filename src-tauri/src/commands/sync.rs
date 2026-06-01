//! Comandos Tauri para gerenciamento de sincronização de pastas.

use std::path::Path;
use std::sync::Arc;

use tauri::State;

use crate::AppState;
use crate::models::sync::{SyncConflict, SyncEvent, SyncFolder};
use crate::services::sync_engine::{start_periodic_sync, sync_folder_with_peer, SyncEngineState};

// ---------------------------------------------------------------------------
// CRUD de pastas sincronizadas
// ---------------------------------------------------------------------------

/// Lista todas as pastas configuradas para sincronização.
#[tauri::command]
pub async fn list_sync_folders(state: State<'_, AppState>) -> Result<Vec<SyncFolder>, String> {
    sqlx::query_as::<_, SyncFolder>(
        "SELECT id, name, local_path, agent_id, status, last_sync_at,
                bytes_transferred, conflict_count, created_at, updated_at
         FROM sync_folders
         ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Adiciona uma nova pasta para sincronização.
/// Máximo de 5 pastas por instância (9.9).
#[tauri::command]
pub async fn add_sync_folder(
    name: String,
    local_path: String,
    agent_id: Option<u32>,
    state: State<'_, AppState>,
    engine: State<'_, Arc<SyncEngineState>>,
) -> Result<SyncFolder, String> {
    // Validar limite de 5 pastas
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sync_folders")
            .fetch_one(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    if count >= 5 {
        return Err("Limite de 5 pastas por instância atingido.".to_string());
    }

    // Validar que a pasta existe no sistema de arquivos
    if !Path::new(&local_path).is_dir() {
        return Err(format!("Caminho não encontrado: {}", local_path));
    }

    // Inserir no banco
    let id = sqlx::query_scalar::<_, u64>(
        "INSERT INTO sync_folders (name, local_path, agent_id, status, bytes_transferred, conflict_count, created_at, updated_at)
         VALUES (?, ?, ?, 'idle', 0, 0, NOW(), NOW())",
    )
    .bind(&name)
    .bind(&local_path)
    .bind(agent_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())? as u32;

    // Iniciar watcher
    if let Err(e) = engine.start_watching(id, Path::new(&local_path)) {
        log::warn!("Watcher para pasta {}: {}", id, e);
    }

    // Se houver agente configurado, iniciar sync periódico
    if let Some(aid) = agent_id {
        if let Ok(Some((addr, port, token))) = get_agent_connection(&state.db, aid).await {
            start_periodic_sync(
                engine.inner().clone(),
                state.db.clone(),
                id,
                local_path.clone(),
                addr,
                port,
                token,
            );
        }
    }

    sqlx::query_as::<_, SyncFolder>(
        "SELECT id, name, local_path, agent_id, status, last_sync_at,
                bytes_transferred, conflict_count, created_at, updated_at
         FROM sync_folders WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Remove uma pasta da sincronização (não apaga arquivos locais).
#[tauri::command]
pub async fn remove_sync_folder(
    id: u32,
    state: State<'_, AppState>,
    engine: State<'_, Arc<SyncEngineState>>,
) -> Result<(), String> {
    engine.stop_watching(id);
    sqlx::query("DELETE FROM sync_folders WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Sincronização manual
// ---------------------------------------------------------------------------

/// Força sincronização imediata de uma pasta com seu peer.
#[tauri::command]
pub async fn force_sync(
    folder_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let folder = sqlx::query_as::<_, SyncFolder>(
        "SELECT id, name, local_path, agent_id, status, last_sync_at,
                bytes_transferred, conflict_count, created_at, updated_at
         FROM sync_folders WHERE id = ?",
    )
    .bind(folder_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Pasta {} não encontrada", folder_id))?;

    let agent_id = folder.agent_id.ok_or("Pasta sem agente peer configurado")?;
    let (addr, port, token) = get_agent_connection(&state.db, agent_id)
        .await?
        .ok_or("Agente peer não encontrado ou não pareado")?;

    sync_folder_with_peer(&state.db, folder_id, &folder.local_path, &addr, port, &token).await
}

// ---------------------------------------------------------------------------
// Histórico e conflitos
// ---------------------------------------------------------------------------

/// Retorna o histórico de eventos de sincronização de uma pasta.
#[tauri::command]
pub async fn get_sync_events(
    folder_id: u32,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<SyncEvent>, String> {
    let lim = limit.unwrap_or(100).min(500);
    sqlx::query_as::<_, SyncEvent>(
        "SELECT id, folder_id, event_type, path, bytes, direction, message, created_at
         FROM sync_events
         WHERE folder_id = ?
         ORDER BY created_at DESC
         LIMIT ?",
    )
    .bind(folder_id)
    .bind(lim)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Retorna os conflitos registrados para uma pasta.
#[tauri::command]
pub async fn get_sync_conflicts(
    folder_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<SyncConflict>, String> {
    sqlx::query_as::<_, SyncConflict>(
        "SELECT id, folder_id, path, local_mtime, remote_mtime, resolution, created_at
         FROM sync_conflicts
         WHERE folder_id = ?
         ORDER BY created_at DESC",
    )
    .bind(folder_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Helpers internos
// ---------------------------------------------------------------------------

/// Busca endereço, porta e session_token do agente para conexão WS.
async fn get_agent_connection(
    pool: &crate::db::DbPool,
    agent_id: u32,
) -> Result<Option<(String, u32, String)>, String> {
    let row = sqlx::query_as::<_, (String, u32, Option<String>)>(
        "SELECT address, port, session_token FROM sync_agents WHERE id = ? LIMIT 1",
    )
    .bind(agent_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.and_then(|(addr, port, token)| token.map(|t| (addr, port, t))))
}
