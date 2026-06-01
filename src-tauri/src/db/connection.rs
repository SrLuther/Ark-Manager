//! Módulo de conexão com o banco de dados MySQL.
//!
//! Gerencia o pool de conexões via sqlx, inicialização, reconexão
//! e exposição do pool como estado gerenciado pelo Tauri.

use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::time::Duration;
use thiserror::Error;

/// Erros do módulo de banco de dados.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Falha ao conectar ao banco de dados: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("String de conexão não configurada")]
    MissingConnectionString,

    #[error("Falha ao executar migration: {0}")]
    Migration(String),
}

/// Alias para o tipo de pool MySQL utilizado em todo o projeto.
pub type DbPool = Pool<MySql>;

/// Parâmetros de configuração da conexão.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// URL de conexão no formato `mysql://user:password@host:port/database`
    pub url: String,
    /// Número máximo de conexões no pool (padrão: 10).
    pub max_connections: u32,
    /// Número mínimo de conexões ociosas mantidas no pool (padrão: 2).
    pub min_connections: u32,
    /// Timeout para aquisição de conexão em segundos (padrão: 30).
    pub acquire_timeout_secs: u64,
    /// Tempo máximo de vida de uma conexão em segundos (padrão: 1800).
    pub max_lifetime_secs: u64,
    /// Tempo de inatividade antes de fechar a conexão em segundos (padrão: 600).
    pub idle_timeout_secs: u64,
}

impl DbConfig {
    /// Cria uma configuração a partir da URL de conexão com valores padrão.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout_secs: 30,
            max_lifetime_secs: 1800,
            idle_timeout_secs: 600,
        }
    }

    /// Cria a configuração a partir de variáveis de ambiente.
    ///
    /// Variáveis suportadas:
    /// - `DATABASE_URL` — URL completa (prioridade máxima).
    /// - `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASSWORD`, `DB_NAME` — componentes individuais.
    pub fn from_env() -> Result<Self, DbError> {
        if let Ok(url) = std::env::var("DATABASE_URL") {
            return Ok(Self::new(url));
        }

        let host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
        let user = std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
        let password = std::env::var("DB_PASSWORD").unwrap_or_default();
        let name = std::env::var("DB_NAME").unwrap_or_else(|_| "ark_manager".to_string());

        let url = format!("mysql://{}:{}@{}:{}/{}", user, password, host, port, name);
        Ok(Self::new(url))
    }
}

// ---------------------------------------------------------------------------
// Arquivo de configuração persistente (%APPDATA%\com.arkmanager.app\database.json)
// ---------------------------------------------------------------------------

/// Retorna o caminho para o arquivo de configuração do banco.
pub fn config_file_path() -> Option<std::path::PathBuf> {
    std::env::var("APPDATA").ok().map(|appdata| {
        std::path::PathBuf::from(appdata)
            .join("com.arkmanager.app")
            .join("database.json")
    })
}

/// Lê a DATABASE_URL salva no arquivo de configuração persistente.
pub fn load_database_url_from_file() -> Option<String> {
    let path = config_file_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json["database_url"].as_str().map(|s| s.to_string())
}

/// Salva a DATABASE_URL no arquivo de configuração persistente.
pub fn save_database_url_to_file(url: &str) -> Result<(), String> {
    let path = config_file_path().ok_or_else(|| "Não foi possível determinar o caminho de configuração".to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::json!({ "database_url": url });
    std::fs::write(&path, serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

/// Carrega a DbConfig tentando: variáveis de ambiente → arquivo de config.
/// Retorna Err se não houver nenhuma configuração disponível.
pub fn load_db_config() -> Result<DbConfig, DbError> {
    // 1. Variáveis de ambiente têm prioridade
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Ok(DbConfig::new(url));
    }
    if std::env::var("DB_HOST").is_ok() || std::env::var("DB_USER").is_ok() {
        return DbConfig::from_env();
    }
    // 2. Arquivo de configuração persistente
    if let Some(url) = load_database_url_from_file() {
        return Ok(DbConfig::new(url));
    }
    // 3. Nenhuma configuração disponível
    Err(DbError::MissingConnectionString)
}

/// Cria e retorna um pool de conexões MySQL com as opções configuradas.
///
/// Executa um `SELECT 1` de verificação após criar o pool.
pub async fn create_pool(config: &DbConfig) -> Result<DbPool, DbError> {
    log::info!(
        "Conectando ao banco de dados MySQL (max_connections={})...",
        config.max_connections
    );

    let pool = MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
        .max_lifetime(Duration::from_secs(config.max_lifetime_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .connect(&config.url)
        .await?;

    // Verificação de conectividade
    sqlx::query("SELECT 1").execute(&pool).await.map_err(|e| {
        log::error!("Falha na verificação de conectividade: {}", e);
        DbError::Connection(e)
    })?;

    log::info!("Pool MySQL criado com sucesso.");
    Ok(pool)
}

/// Tenta criar o pool com retentativas em caso de falha transitória.
///
/// Útil durante a inicialização da aplicação quando o MySQL pode ainda
/// estar subindo. Realiza até `max_retries` tentativas com espera exponencial.
pub async fn create_pool_with_retry(
    config: &DbConfig,
    max_retries: u32,
) -> Result<DbPool, DbError> {
    let mut attempt = 0u32;

    loop {
        attempt += 1;
        match create_pool(config).await {
            Ok(pool) => return Ok(pool),
            Err(e) if attempt < max_retries => {
                let wait_secs = 2u64.pow(attempt.min(6)); // backoff exponencial, máx 64s
                log::warn!(
                    "Tentativa {}/{} falhou: {}. Aguardando {}s...",
                    attempt,
                    max_retries,
                    e,
                    wait_secs
                );
                tokio::time::sleep(Duration::from_secs(wait_secs)).await;
            }
            Err(e) => {
                log::error!(
                    "Falha ao conectar após {} tentativas: {}",
                    max_retries,
                    e
                );
                return Err(e);
            }
        }
    }
}

