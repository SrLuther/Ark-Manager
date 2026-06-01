//! Módulo de banco de dados — MySQL via sqlx.
//!
//! Responsável por:
//! - Conexão e pool de conexões MySQL
//! - Inicialização e migrations do schema
//! - Tratamento de erros de banco
//! - Reconexão com backoff exponencial

pub mod connection;
pub mod migrations;

pub use connection::{create_pool, create_pool_with_retry, DbConfig, DbError, DbPool};
pub use migrations::run_migrations;

/// Inicializa o banco de dados completo: cria o pool e executa todas as migrations.
///
/// Esta é a função de entrada chamada pelo `lib.rs` durante a inicialização da aplicação.
pub async fn initialize(config: &DbConfig) -> Result<DbPool, DbError> {
    let pool = create_pool_with_retry(config, 5).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}
