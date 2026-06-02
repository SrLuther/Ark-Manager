//! Módulo de banco de dados — SQLite local via sqlx.
//!
//! Sem configuração necessária. O banco é criado automaticamente em
//! %APPDATA%\com.arkmanager.app\ark-manager.db na primeira execução.

pub mod connection;
pub mod migrations;

pub use connection::{create_pool, DbError, DbPool};
pub use migrations::run_migrations;

/// Inicializa o banco de dados: cria o pool SQLite e executa todas as migrations.
///
/// Esta é a função de entrada chamada pelo `lib.rs` durante a inicialização da aplicação.
pub async fn initialize() -> Result<DbPool, DbError> {
    let pool = create_pool().await?;
    run_migrations(&pool).await?;
    Ok(pool)
}
