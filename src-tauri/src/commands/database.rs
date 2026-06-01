use crate::db::connection::{load_database_url_from_file, save_database_url_to_file, DbConfig};
use crate::db::create_pool;

/// Retorna a DATABASE_URL atualmente salva no arquivo de configuração.
/// Retorna None se não houver configuração persistida.
#[tauri::command]
pub async fn get_database_url() -> Result<Option<String>, String> {
    Ok(load_database_url_from_file())
}

/// Salva a DATABASE_URL no arquivo de configuração persistente.
/// O app precisa ser reiniciado para que a nova URL seja usada.
#[tauri::command]
pub async fn save_database_url(url: String) -> Result<(), String> {
    save_database_url_to_file(&url)
}

/// Testa se é possível conectar ao banco de dados com a URL fornecida.
/// Retorna Ok(true) se a conexão for bem-sucedida.
#[tauri::command]
pub async fn test_database_connection(url: String) -> Result<bool, String> {
    let config = DbConfig::new(url);
    match create_pool(&config).await {
        Ok(pool) => {
            pool.close().await;
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}
