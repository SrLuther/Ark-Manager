//! Módulo de conexão com o banco de dados SQLite (local).
//!
//! Sem configuração necessária — o banco é criado automaticamente em
//! %APPDATA%\com.arkmanager.app\ark-manager.db na primeira execução.

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use std::str::FromStr;
use thiserror::Error;

/// Erros do módulo de banco de dados.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Falha ao conectar ao banco de dados: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Falha ao executar migration: {0}")]
    Migration(String),
}

/// Alias para o tipo de pool SQLite utilizado em todo o projeto.
pub type DbPool = Pool<Sqlite>;

/// Retorna o caminho para o arquivo SQLite da aplicação.
///
/// Windows: `%APPDATA%\com.arkmanager.app\ark-manager.db`
pub fn get_db_path() -> std::path::PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = std::path::PathBuf::from(appdata).join("com.arkmanager.app");
    std::fs::create_dir_all(&dir).ok();
    dir.join("ark-manager.db")
}

/// Cria e retorna um pool de conexões SQLite.
pub async fn create_pool() -> Result<DbPool, DbError> {
    let path = get_db_path();
    log::info!("Abrindo banco de dados SQLite em: {}", path.display());

    let url = format!("sqlite://{}?mode=rwc", path.to_string_lossy());
    let opts = SqliteConnectOptions::from_str(&url)
        .map_err(DbError::Connection)?
        .foreign_keys(true)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .map_err(DbError::Connection)?;

    log::info!("Pool SQLite criado com sucesso.");
    Ok(pool)
}