use crate::db::connection::{load_database_url_from_file, save_database_url_to_file, DbConfig};
use crate::db::{create_pool, migrations};

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
    let config = DbConfig::new(&url);
    match create_pool(&config).await {
        Ok(pool) => {
            pool.close().await;
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Cria o banco de dados caso não exista e executa as migrations.
/// Conecta primeiro ao servidor MySQL sem especificar o banco,
/// executa CREATE DATABASE IF NOT EXISTS, depois reconecta com o banco
/// e roda todas as migrations.
#[tauri::command]
pub async fn setup_database(url: String) -> Result<String, String> {
    // Extrai o nome do banco da URL
    // Formato: mysql://user:pass@host:port/db_name[?params]
    let db_name = url
        .trim_start_matches("mysql://")
        .split('/')
        .nth(1)
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .to_string();

    if db_name.is_empty() {
        return Err("Não foi possível extrair o nome do banco de dados da URL.".to_string());
    }

    // Monta URL sem o banco (para poder criá-lo)
    let url_without_db = {
        let base = url.trim_start_matches("mysql://");
        let authority_and_rest = base.split('/').next().unwrap_or(base);
        format!("mysql://{}/", authority_and_rest)
    };

    // 1. Conecta sem banco e cria o schema
    let cfg_no_db = DbConfig::new(&url_without_db);
    let pool_no_db = create_pool(&cfg_no_db).await.map_err(|e| {
        format!("Não foi possível conectar ao servidor MySQL: {}", e)
    })?;

    sqlx::query(&format!(
        "CREATE DATABASE IF NOT EXISTS `{}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
        db_name.replace('`', "")
    ))
    .execute(&pool_no_db)
    .await
    .map_err(|e| format!("Erro ao criar banco de dados: {}", e))?;

    pool_no_db.close().await;

    // 2. Conecta com o banco criado e roda migrations
    let cfg = DbConfig::new(&url);
    let pool = create_pool(&cfg).await.map_err(|e| {
        format!("Banco criado, mas falha ao conectar: {}", e)
    })?;

    migrations::run_migrations(&pool).await.map_err(|e| {
        format!("Banco criado, mas erro nas migrations: {}", e)
    })?;

    pool.close().await;

    Ok(format!("Banco '{}' criado e configurado com sucesso.", db_name))
}
