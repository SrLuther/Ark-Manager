use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Representação de um mod instalado no servidor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModEntry {
    pub mod_id: String,
    pub position: usize,
}

/// Lista os mods de um servidor (como lista de IDs ordenados).
#[tauri::command]
pub async fn list_mods(
    server_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let row: (Option<String>,) =
        sqlx::query_as("SELECT mods FROM am_servers WHERE id = ?")
            .bind(server_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| e.to_string())?;

    let ids = match row.0 {
        Some(s) if !s.is_empty() => s
            .split(',')
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .collect(),
        _ => vec![],
    };

    Ok(ids)
}

/// Adiciona um mod ao servidor (ao final da lista).
#[tauri::command]
pub async fn add_mod(
    server_id: u32,
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let mut ids = list_mods(server_id, state.clone()).await?;

    let mod_id = mod_id.trim().to_string();
    if !mod_id.is_empty() && !ids.contains(&mod_id) {
        ids.push(mod_id);
    }

    let mods_str = ids.join(",");
    sqlx::query("UPDATE am_servers SET mods = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&mods_str)
        .bind(server_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ids)
}

/// Remove um mod do servidor pelo ID.
#[tauri::command]
pub async fn remove_mod(
    server_id: u32,
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let mut ids = list_mods(server_id, state.clone()).await?;

    ids.retain(|id| id != mod_id.trim());

    let mods_str = ids.join(",");
    sqlx::query("UPDATE am_servers SET mods = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&mods_str)
        .bind(server_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ids)
}

/// Reordena os mods de um servidor.
#[tauri::command]
pub async fn reorder_mods(
    server_id: u32,
    ordered_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mods_str = ordered_ids
        .iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>()
        .join(",");

    sqlx::query("UPDATE am_servers SET mods = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&mods_str)
        .bind(server_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

