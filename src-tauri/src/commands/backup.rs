use crate::models::backup::Backup;
use crate::services::backup_service::{self, saved_arks_path};
use crate::AppState;
use std::path::Path;
use tauri::{AppHandle, Emitter, State};

/// Lista os backups de um servidor armazenados no banco.
#[tauri::command]
pub async fn list_backups(
    server_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<Backup>, String> {
    sqlx::query_as::<_, Backup>(
        "SELECT * FROM am_backups WHERE server_id = ? ORDER BY created_at DESC",
    )
    .bind(server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Cria um backup manual do SavedArks.
/// Emite `backup:started` e `backup:completed` / `backup:failed`.
#[tauri::command]
pub async fn create_backup(
    server_id: u32,
    install_dir: String,
    backup_base_dir: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let _ = app.emit("backup:started", server_id);

    let saved_arks = saved_arks_path(Path::new(&install_dir));
    let result = backup_service::create_backup(
        &saved_arks,
        Path::new(&backup_base_dir),
        &server_id.to_string(),
    )
    .await
    .map_err(|e| {
        let _ = app.emit("backup:failed", e.to_string());
        e.to_string()
    })?;

    let backup_path_str = result.backup_path.display().to_string();

    // Registra no banco
    sqlx::query(
        r#"INSERT INTO am_backups
        (server_id, backup_type, file_path, size_bytes, includes_saves, status, created_at)
        VALUES (?, 'manual', ?, ?, 1, 'completed', NOW())"#,
    )
    .bind(server_id)
    .bind(&backup_path_str)
    .bind(result.size_bytes)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let _ = app.emit("backup:completed", server_id);
    Ok(backup_path_str)
}

/// Restaura um backup sobrescrevendo o SavedArks.
#[tauri::command]
pub async fn restore_backup(
    backup_path: String,
    install_dir: String,
    app: AppHandle,
) -> Result<(), String> {
    let saved_arks = saved_arks_path(Path::new(&install_dir));

    backup_service::restore_backup(Path::new(&backup_path), &saved_arks)
        .await
        .map_err(|e| {
            let _ = app.emit("backup:failed", e.to_string());
            e.to_string()
        })?;

    Ok(())
}

/// Remove backups antigos mantendo apenas os `keep_count` mais recentes.
#[tauri::command]
pub fn prune_backups(
    backup_base_dir: String,
    server_id: u32,
    keep_count: usize,
) -> Result<usize, String> {
    backup_service::prune_old_backups(
        Path::new(&backup_base_dir),
        &server_id.to_string(),
        keep_count,
    )
    .map_err(|e| e.to_string())
}

