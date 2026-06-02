use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;

/// Cluster Cross-ARK.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    pub id: u32,
    pub name: String,
    pub cluster_id: String,
    pub cluster_dir: String,
    pub description: Option<String>,
}

/// Payload para criação de cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClusterRequest {
    pub name: String,
    pub cluster_id: String,
    pub cluster_dir: String,
    pub description: Option<String>,
}

/// Lista todos os clusters.
#[tauri::command]
pub async fn list_clusters(state: State<'_, AppState>) -> Result<Vec<Cluster>, String> {
    sqlx::query_as::<_, Cluster>("SELECT id, name, cluster_id, cluster_dir, description FROM am_clusters ORDER BY name ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())
}

/// Busca um cluster pelo ID.
#[tauri::command]
pub async fn get_cluster(id: u32, state: State<'_, AppState>) -> Result<Cluster, String> {
    sqlx::query_as::<_, Cluster>(
        "SELECT id, name, cluster_id, cluster_dir, description FROM am_clusters WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Cria um novo cluster.
#[tauri::command]
pub async fn create_cluster(
    req: CreateClusterRequest,
    state: State<'_, AppState>,
) -> Result<Cluster, String> {
    sqlx::query(
        "INSERT INTO am_clusters (name, cluster_id, cluster_dir, description, created_at, updated_at) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(&req.name)
    .bind(&req.cluster_id)
    .bind(&req.cluster_dir)
    .bind(&req.description)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, Cluster>(
        "SELECT id, name, cluster_id, cluster_dir, description FROM am_clusters ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Remove um cluster. Os servidores vinculados têm cluster_id zerado.
#[tauri::command]
pub async fn delete_cluster(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    sqlx::query("UPDATE am_servers SET cluster_id = NULL, updated_at = CURRENT_TIMESTAMP WHERE cluster_id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM am_clusters WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Vincula um servidor a um cluster.
#[tauri::command]
pub async fn assign_server_to_cluster(
    server_id: u32,
    cluster_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("UPDATE am_servers SET cluster_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(cluster_id)
        .bind(server_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Desvincula um servidor de qualquer cluster.
#[tauri::command]
pub async fn unassign_server_from_cluster(
    server_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query("UPDATE am_servers SET cluster_id = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(server_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

