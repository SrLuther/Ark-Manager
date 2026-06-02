//! Migrations do banco de dados SQLite.
//!
//! Executa criacao e evolucao de schema de forma idempotente.
//! Cada migration verifica se a alteracao ja foi aplicada antes de executa-la.

use super::connection::{DbError, DbPool};

/// Executa todas as migrations na ordem correta.
pub async fn run_migrations(pool: &DbPool) -> Result<(), DbError> {
    log::info!("Executando migrations do banco de dados...");

    create_migrations_table(pool).await?;
    migrate_v1_initial_schema(pool).await?;
    migrate_v2_server_columns(pool).await?;
    migrate_v3_backup_columns(pool).await?;
    migrate_v4_scheduler_columns(pool).await?;
    migrate_v5_agent_tables(pool).await?;
    migrate_v6_sync_tables(pool).await?;
    migrate_v7_seasonal_events(pool).await?;

    log::info!("Migrations concluidas.");
    Ok(())
}

async fn create_migrations_table(pool: &DbPool) -> Result<(), DbError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_migrations (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn is_applied(pool: &DbPool, name: &str) -> Result<bool, DbError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM am_migrations WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(row.0 > 0)
}

async fn mark_applied(pool: &DbPool, name: &str) -> Result<(), DbError> {
    sqlx::query("INSERT OR IGNORE INTO am_migrations (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

// v1 - Schema inicial
async fn migrate_v1_initial_schema(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v1_initial_schema";
    if is_applied(pool, NAME).await? { return Ok(()); }

    log::info!("Migration {}: criando schema inicial...", NAME);

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_servers (
            id                   INTEGER PRIMARY KEY AUTOINCREMENT,
            name                 TEXT NOT NULL UNIQUE,
            install_path         TEXT NOT NULL,
            map_name             TEXT NOT NULL DEFAULT 'TheIsland',
            session_name         TEXT NOT NULL,
            game_port            INTEGER NOT NULL DEFAULT 7777,
            query_port           INTEGER NOT NULL DEFAULT 27015,
            rcon_port            INTEGER NOT NULL DEFAULT 32330,
            rcon_enabled         INTEGER NOT NULL DEFAULT 1,
            max_players          INTEGER NOT NULL DEFAULT 70,
            server_password      TEXT,
            admin_password       TEXT NOT NULL,
            spectator_password   TEXT,
            ip_address           TEXT,
            mods                 TEXT,
            cluster_id           INTEGER,
            enable_pvp           INTEGER NOT NULL DEFAULT 1,
            enable_battleye      INTEGER NOT NULL DEFAULT 1,
            enable_crosshair     INTEGER NOT NULL DEFAULT 0,
            allow_third_person   INTEGER NOT NULL DEFAULT 0,
            allow_tribe_alliances INTEGER NOT NULL DEFAULT 1,
            custom_args          TEXT,
            auto_start           INTEGER NOT NULL DEFAULT 0,
            auto_restart         INTEGER NOT NULL DEFAULT 0,
            startup_delay        INTEGER NOT NULL DEFAULT 0,
            status               TEXT NOT NULL DEFAULT 'stopped',
            pid                  INTEGER,
            last_started         TEXT,
            last_stopped         TEXT,
            created_at           TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at           TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_servers: {}", e)))?;

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_servers_status ON am_servers (status)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_servers_cluster ON am_servers (cluster_id)").execute(pool).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_clusters (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            name         TEXT NOT NULL UNIQUE,
            cluster_id   TEXT NOT NULL,
            cluster_path TEXT NOT NULL,
            description  TEXT,
            created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_clusters: {}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_cluster_servers (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            cluster_id INTEGER NOT NULL,
            server_id  INTEGER NOT NULL,
            added_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (cluster_id, server_id)
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_cluster_servers: {}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_mods (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            server_id    INTEGER NOT NULL,
            mod_id       TEXT NOT NULL,
            name         TEXT NOT NULL,
            version      TEXT,
            description  TEXT,
            workshop_url TEXT,
            enabled      INTEGER NOT NULL DEFAULT 1,
            load_order   INTEGER NOT NULL DEFAULT 0,
            installed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (server_id, mod_id)
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_mods: {}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_backups (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            server_id         INTEGER NOT NULL,
            backup_type       TEXT NOT NULL DEFAULT 'manual',
            file_path         TEXT NOT NULL,
            size_bytes        INTEGER NOT NULL DEFAULT 0,
            includes_configs  INTEGER NOT NULL DEFAULT 1,
            includes_mods     INTEGER NOT NULL DEFAULT 0,
            includes_saves    INTEGER NOT NULL DEFAULT 1,
            includes_cluster  INTEGER NOT NULL DEFAULT 0,
            label             TEXT,
            notes             TEXT,
            is_protected      INTEGER NOT NULL DEFAULT 0,
            status            TEXT NOT NULL DEFAULT 'completed',
            hash              TEXT,
            verified          INTEGER NOT NULL DEFAULT 0,
            created_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_backups: {}", e)))?;

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_backup_server ON am_backups (server_id)").execute(pool).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_backup_policies (
            server_id             INTEGER PRIMARY KEY,
            enabled               INTEGER NOT NULL DEFAULT 0,
            interval_hours        INTEGER NOT NULL DEFAULT 24,
            retention_days        INTEGER NOT NULL DEFAULT 7,
            retention_count       INTEGER NOT NULL DEFAULT 10,
            storage_quota_gb      REAL NOT NULL DEFAULT 50.0,
            backup_before_update  INTEGER NOT NULL DEFAULT 1,
            backup_before_restart INTEGER NOT NULL DEFAULT 1,
            compression_enabled   INTEGER NOT NULL DEFAULT 1
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_backup_policies: {}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_scheduled_tasks (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            server_id           INTEGER NOT NULL,
            task_name           TEXT,
            task_type           TEXT NOT NULL,
            cron_expression     TEXT NOT NULL,
            command             TEXT,
            message             TEXT,
            pre_warning_minutes INTEGER NOT NULL DEFAULT 5,
            enabled             INTEGER NOT NULL DEFAULT 1,
            last_run            TEXT,
            next_run            TEXT,
            created_at          TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at          TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_scheduled_tasks: {}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_settings (
            key        TEXT PRIMARY KEY,
            value      TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_settings: {}", e)))?;

    let defaults = [
        ("steamcmd_path", ""),
        ("default_install_path", ""),
        ("theme", "dark"),
        ("language", "pt-BR"),
        ("startup_timeout", "1800"),
        ("global_auto_start_enabled", "false"),
        ("global_boot_delay", "0"),
        ("start_minimized_to_tray", "false"),
    ];

    for (key, value) in defaults {
        sqlx::query("INSERT OR IGNORE INTO am_settings (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(pool)
            .await
            .map_err(|e| DbError::Migration(format!("settings default '{}': {}", key, e)))?;
    }

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// v2 - Colunas adicionais em servers
async fn migrate_v2_server_columns(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v2_server_columns";
    if is_applied(pool, NAME).await? { return Ok(()); }

    log::info!("Migration {}: adicionando colunas extras em am_servers...", NAME);

    let _ = sqlx::query("ALTER TABLE am_servers ADD COLUMN hardware_config TEXT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE am_servers ADD COLUMN startup_priority INTEGER NOT NULL DEFAULT 100").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE am_servers ADD COLUMN intelligent_mode INTEGER NOT NULL DEFAULT 0").execute(pool).await;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// v3 - Colunas adicionais em backups
async fn migrate_v3_backup_columns(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v3_backup_columns";
    if is_applied(pool, NAME).await? { return Ok(()); }

    log::info!("Migration {}: adicionando colunas extras em am_backups...", NAME);

    let _ = sqlx::query("ALTER TABLE am_backups ADD COLUMN compression_level INTEGER DEFAULT 6").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE am_backups ADD COLUMN duration_secs INTEGER").execute(pool).await;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// v4 - Colunas adicionais em scheduled_tasks
async fn migrate_v4_scheduler_columns(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v4_scheduler_columns";
    if is_applied(pool, NAME).await? { return Ok(()); }

    log::info!("Migration {}: adicionando colunas extras em am_scheduled_tasks...", NAME);

    let _ = sqlx::query("ALTER TABLE am_scheduled_tasks ADD COLUMN run_count INTEGER NOT NULL DEFAULT 0").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE am_scheduled_tasks ADD COLUMN last_result TEXT").execute(pool).await;
    let _ = sqlx::query("ALTER TABLE am_scheduled_tasks ADD COLUMN last_error TEXT").execute(pool).await;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// v5 - Tabelas de agente e sync
async fn migrate_v5_agent_tables(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v5_agent_tables";
    if is_applied(pool, NAME).await? { return Ok(()); }

    log::info!("Migration {}: criando tabelas de agentes e sync...", NAME);

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_sync_agents (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            name         TEXT NOT NULL,
            address      TEXT NOT NULL,
            port         INTEGER NOT NULL DEFAULT 45678,
            paired       INTEGER NOT NULL DEFAULT 0,
            token_hash   TEXT,
            last_seen_at TEXT,
            created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (address, port)
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_sync_agents: {}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_sync_folders (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            name              TEXT NOT NULL,
            local_path        TEXT NOT NULL,
            agent_id          INTEGER,
            status            TEXT NOT NULL DEFAULT 'pending',
            last_sync_at      TEXT,
            bytes_transferred INTEGER NOT NULL DEFAULT 0,
            conflict_count    INTEGER NOT NULL DEFAULT 0,
            created_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_sync_folders: {}", e)))?;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// v6 - Tabelas de eventos e conflitos de sync
async fn migrate_v6_sync_tables(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v6_sync_tables";
    if is_applied(pool, NAME).await? { return Ok(()); }

    log::info!("Migration {}: criando tabelas de eventos e conflitos de sync...", NAME);

    let _ = sqlx::query("ALTER TABLE am_sync_agents ADD COLUMN session_token TEXT").execute(pool).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_sync_events (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id  INTEGER NOT NULL,
            event_type TEXT NOT NULL DEFAULT 'transfer',
            path       TEXT,
            bytes      INTEGER,
            direction  TEXT,
            message    TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_sync_events: {}", e)))?;

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_se_folder ON am_sync_events (folder_id)").execute(pool).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_sync_conflicts (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id    INTEGER NOT NULL,
            path         TEXT NOT NULL,
            local_mtime  INTEGER NOT NULL,
            remote_mtime INTEGER NOT NULL,
            resolution   TEXT NOT NULL DEFAULT 'local',
            created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_sync_conflicts: {}", e)))?;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// v7 - Eventos Sazonais
async fn migrate_v7_seasonal_events(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v7_seasonal_events";
    if is_applied(pool, NAME).await? { return Ok(()); }

    log::info!("Migration {}: criando tabelas de eventos sazonais...", NAME);

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_seasonal_events (
            id                          INTEGER PRIMARY KEY AUTOINCREMENT,
            name                        TEXT NOT NULL,
            description                 TEXT,
            start_at                    TEXT NOT NULL,
            end_at                      TEXT NOT NULL,
            status                      TEXT NOT NULL DEFAULT 'scheduled',
            broadcast_interval_seconds  INTEGER NOT NULL DEFAULT 300,
            created_at                  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at                  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_seasonal_events: {}", e)))?;

    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_se_status ON am_seasonal_events (status)").execute(pool).await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_se_start ON am_seasonal_events (start_at)").execute(pool).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_seasonal_event_rates (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id            INTEGER NOT NULL UNIQUE,
            xp_multiplier       REAL NOT NULL DEFAULT 1.0,
            harvest_multiplier  REAL NOT NULL DEFAULT 1.0,
            taming_multiplier   REAL NOT NULL DEFAULT 1.0,
            breeding_multiplier REAL NOT NULL DEFAULT 1.0,
            quality_multiplier  REAL NOT NULL DEFAULT 1.0
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_seasonal_event_rates: {}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_seasonal_event_servers (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id  INTEGER NOT NULL,
            server_id INTEGER NOT NULL,
            UNIQUE (event_id, server_id)
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_seasonal_event_servers: {}", e)))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS am_seasonal_event_backups (
            id                   INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id             INTEGER NOT NULL,
            server_id            INTEGER NOT NULL,
            gus_backup_path      TEXT NOT NULL,
            game_ini_backup_path TEXT NOT NULL,
            created_at           TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE (event_id, server_id)
        )",
    )
    .execute(pool).await
    .map_err(|e| DbError::Migration(format!("am_seasonal_event_backups: {}", e)))?;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}