//! Comandos de banco de dados — SQLite local, sem configuracao necessaria.
//!
//! O banco e inicializado automaticamente em
//! %APPDATA%\com.arkmanager.app\ark-manager.db ao iniciar o app.

/// Retorna o caminho do banco SQLite atual.
#[tauri::command]
pub async fn setup_database() -> Result<String, String> {
    let path = crate::db::connection::get_db_path();
    Ok(format!("Banco SQLite em uso: {}", path.display()))
}