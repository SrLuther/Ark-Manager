//! Migrations do banco de dados MySQL.
//!
//! Executa criação e evolução de schema de forma idempotente.
//! Cada migration verifica se a alteração já foi aplicada antes de executá-la,
//! garantindo segurança em múltiplas execuções.

use super::connection::{DbError, DbPool};

/// Executa todas as migrations na ordem correta.
///
/// Seguro para ser chamado múltiplas vezes — operações são idempotentes.
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

    log::info!("Migrations concluídas.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Controle de versão de migrations
// ---------------------------------------------------------------------------

/// Cria a tabela de controle de migrations, caso não exista.
async fn create_migrations_table(pool: &DbPool) -> Result<(), DbError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_migrations (
            id          INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            name        VARCHAR(255) NOT NULL UNIQUE,
            applied_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Retorna `true` se a migration com esse nome já foi aplicada.
async fn is_applied(pool: &DbPool, name: &str) -> Result<bool, DbError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM am_migrations WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await?;
    Ok(row.0 > 0)
}

/// Registra a migration como aplicada.
async fn mark_applied(pool: &DbPool, name: &str) -> Result<(), DbError> {
    sqlx::query("INSERT IGNORE INTO am_migrations (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// v1 — Schema inicial
// ---------------------------------------------------------------------------

async fn migrate_v1_initial_schema(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v1_initial_schema";
    if is_applied(pool, NAME).await? {
        return Ok(());
    }

    log::info!("Migration {}: criando schema inicial...", NAME);

    // --- Tabela: servers ---------------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_servers (
            id                  INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            name                VARCHAR(255) NOT NULL,
            install_path        TEXT NOT NULL,
            map_name            VARCHAR(100) NOT NULL DEFAULT 'TheIsland',
            session_name        VARCHAR(255) NOT NULL,
            game_port           SMALLINT UNSIGNED NOT NULL DEFAULT 7777,
            query_port          SMALLINT UNSIGNED NOT NULL DEFAULT 27015,
            rcon_port           SMALLINT UNSIGNED NOT NULL DEFAULT 32330,
            rcon_enabled        TINYINT(1) NOT NULL DEFAULT 1,
            max_players         SMALLINT UNSIGNED NOT NULL DEFAULT 70,
            server_password     VARCHAR(255),
            admin_password      VARCHAR(255) NOT NULL,
            spectator_password  VARCHAR(255),
            ip_address          VARCHAR(45),
            mods                TEXT COMMENT 'IDs de mods separados por vírgula',
            cluster_id          INT UNSIGNED,
            enable_pvp          TINYINT(1) NOT NULL DEFAULT 1,
            enable_battleye     TINYINT(1) NOT NULL DEFAULT 1,
            enable_crosshair    TINYINT(1) NOT NULL DEFAULT 0,
            allow_third_person  TINYINT(1) NOT NULL DEFAULT 0,
            allow_tribe_alliances TINYINT(1) NOT NULL DEFAULT 1,
            custom_args         TEXT COMMENT 'Argumentos CLI adicionais',
            auto_start          TINYINT(1) NOT NULL DEFAULT 0,
            auto_restart        TINYINT(1) NOT NULL DEFAULT 0,
            startup_delay       SMALLINT UNSIGNED NOT NULL DEFAULT 0,
            status              ENUM(
                'stopped','starting','running','online',
                'crashed','updating','restarting','stopping'
            ) NOT NULL DEFAULT 'stopped',
            pid                 INT UNSIGNED,
            last_started        DATETIME,
            last_stopped        DATETIME,
            created_at          DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at          DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            UNIQUE KEY uq_server_name (name),
            KEY idx_status (status),
            KEY idx_cluster (cluster_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("servers: {}", e)))?;

    // --- Tabela: clusters --------------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_clusters (
            id              INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            name            VARCHAR(255) NOT NULL,
            cluster_id      VARCHAR(255) NOT NULL COMMENT 'ClusterId ARK',
            cluster_path    TEXT NOT NULL COMMENT 'Caminho compartilhado CrossARK',
            description     TEXT,
            created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            UNIQUE KEY uq_cluster_name (name)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("clusters: {}", e)))?;

    // --- Tabela: cluster_servers -------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_cluster_servers (
            id          INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            cluster_id  INT UNSIGNED NOT NULL,
            server_id   INT UNSIGNED NOT NULL,
            added_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY uq_cluster_server (cluster_id, server_id),
            KEY idx_cs_cluster (cluster_id),
            KEY idx_cs_server (server_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("cluster_servers: {}", e)))?;

    // --- Tabela: mods ------------------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_mods (
            id              INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            server_id       INT UNSIGNED NOT NULL,
            mod_id          VARCHAR(50) NOT NULL COMMENT 'ID Steam Workshop',
            name            VARCHAR(255) NOT NULL,
            version         VARCHAR(50),
            description     TEXT,
            workshop_url    VARCHAR(512),
            enabled         TINYINT(1) NOT NULL DEFAULT 1,
            load_order      SMALLINT UNSIGNED NOT NULL DEFAULT 0,
            installed_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            UNIQUE KEY uq_mod_server (server_id, mod_id),
            KEY idx_mods_server (server_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("mods: {}", e)))?;

    // --- Tabela: backups ---------------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_backups (
            id                  INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            server_id           INT UNSIGNED NOT NULL,
            backup_type         ENUM('manual','auto','pre-update','pre-restart') NOT NULL DEFAULT 'manual',
            file_path           TEXT NOT NULL,
            size_bytes          BIGINT UNSIGNED NOT NULL DEFAULT 0,
            includes_configs    TINYINT(1) NOT NULL DEFAULT 1,
            includes_mods       TINYINT(1) NOT NULL DEFAULT 0,
            includes_saves      TINYINT(1) NOT NULL DEFAULT 1,
            includes_cluster    TINYINT(1) NOT NULL DEFAULT 0,
            label               VARCHAR(255),
            notes               TEXT,
            is_protected        TINYINT(1) NOT NULL DEFAULT 0,
            status              ENUM('pending','running','completed','failed') NOT NULL DEFAULT 'completed',
            hash                VARCHAR(128) COMMENT 'SHA-256 do arquivo de backup',
            verified            TINYINT(1) NOT NULL DEFAULT 0,
            created_at          DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            KEY idx_backup_server (server_id),
            KEY idx_backup_type (backup_type),
            KEY idx_backup_status (status)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("backups: {}", e)))?;

    // --- Tabela: backup_policies ------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_backup_policies (
            server_id               INT UNSIGNED PRIMARY KEY,
            enabled                 TINYINT(1) NOT NULL DEFAULT 0,
            interval_hours          SMALLINT UNSIGNED NOT NULL DEFAULT 24,
            retention_days          SMALLINT UNSIGNED NOT NULL DEFAULT 7,
            retention_count         SMALLINT UNSIGNED NOT NULL DEFAULT 10,
            storage_quota_gb        FLOAT NOT NULL DEFAULT 50.0,
            backup_before_update    TINYINT(1) NOT NULL DEFAULT 1,
            backup_before_restart   TINYINT(1) NOT NULL DEFAULT 1,
            compression_enabled     TINYINT(1) NOT NULL DEFAULT 1
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("backup_policies: {}", e)))?;

    // --- Tabela: scheduled_tasks ------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_scheduled_tasks (
            id                      INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            server_id               INT UNSIGNED NOT NULL,
            task_name               VARCHAR(255),
            task_type               ENUM(
                'restart','backup','rcon-command','announcement',
                'save-world','destroy-wild-dinos','update'
            ) NOT NULL,
            cron_expression         VARCHAR(100) NOT NULL,
            command                 TEXT COMMENT 'Comando RCON (para task_type = rcon-command)',
            message                 TEXT COMMENT 'Mensagem de broadcast',
            pre_warning_minutes     SMALLINT UNSIGNED NOT NULL DEFAULT 5,
            enabled                 TINYINT(1) NOT NULL DEFAULT 1,
            last_run                DATETIME,
            next_run                DATETIME,
            created_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            KEY idx_task_server (server_id),
            KEY idx_task_enabled (enabled),
            KEY idx_task_next_run (next_run)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("scheduled_tasks: {}", e)))?;

    // --- Tabela: settings (key-value global) -------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_settings (
            `key`       VARCHAR(255) PRIMARY KEY,
            value       TEXT NOT NULL,
            updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("settings: {}", e)))?;

    // --- Índices adicionais ------------------------------------------------
    // Executados com IF NOT EXISTS equivalente via IGNORE em CREATE INDEX (MySQL não suporta
    // IF NOT EXISTS nativamente antes do 8.0.31 — usamos tabelas separadas como guard)
    let _ = sqlx::query(
        "CREATE INDEX idx_servers_status ON am_servers (status)",
    )
    .execute(pool)
    .await; // Ignora erro caso já exista

    // --- Valores padrão de configuração ------------------------------------
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
        sqlx::query(
            "INSERT IGNORE INTO am_settings (`key`, value) VALUES (?, ?)",
        )
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

// ---------------------------------------------------------------------------
// v2 — Colunas adicionais em servers
// ---------------------------------------------------------------------------

async fn migrate_v2_server_columns(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v2_server_columns";
    if is_applied(pool, NAME).await? {
        return Ok(());
    }

    log::info!("Migration {}: adicionando colunas extras em servers...", NAME);

    // hardware_allocation embutido em servers como JSON
    let _ = sqlx::query(
        "ALTER TABLE am_servers ADD COLUMN hardware_config JSON COMMENT 'CPU affinity e prioridade'",
    )
    .execute(pool)
    .await; // Ignora caso coluna já exista

    let _ = sqlx::query(
        "ALTER TABLE am_servers ADD COLUMN startup_priority SMALLINT UNSIGNED NOT NULL DEFAULT 100",
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE am_servers ADD COLUMN intelligent_mode TINYINT(1) NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// ---------------------------------------------------------------------------
// v3 — Colunas adicionais em backups
// ---------------------------------------------------------------------------

async fn migrate_v3_backup_columns(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v3_backup_columns";
    if is_applied(pool, NAME).await? {
        return Ok(());
    }

    log::info!("Migration {}: adicionando colunas extras em backups...", NAME);

    // compression_level — para futura compressão configurável
    let _ = sqlx::query(
        "ALTER TABLE am_backups ADD COLUMN compression_level TINYINT UNSIGNED DEFAULT 6",
    )
    .execute(pool)
    .await;

    // duration_secs — quanto tempo durou o backup
    let _ = sqlx::query(
        "ALTER TABLE am_backups ADD COLUMN duration_secs SMALLINT UNSIGNED",
    )
    .execute(pool)
    .await;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// ---------------------------------------------------------------------------
// v4 — Colunas adicionais em scheduled_tasks
// ---------------------------------------------------------------------------

async fn migrate_v4_scheduler_columns(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v4_scheduler_columns";
    if is_applied(pool, NAME).await? {
        return Ok(());
    }

    log::info!("Migration {}: adicionando colunas extras em scheduled_tasks...", NAME);

    let _ = sqlx::query(
        "ALTER TABLE am_scheduled_tasks ADD COLUMN run_count INT UNSIGNED NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE am_scheduled_tasks ADD COLUMN last_result ENUM('success','failure','skipped')",
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE am_scheduled_tasks ADD COLUMN last_error TEXT",
    )
    .execute(pool)
    .await;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// ---------------------------------------------------------------------------
// v5 — Tabelas do agente de rede e sincronização
// ---------------------------------------------------------------------------

async fn migrate_v5_agent_tables(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v5_agent_tables";
    if is_applied(pool, NAME).await? {
        return Ok(());
    }

    log::info!("Migration {}: criando tabelas de agentes e sync...", NAME);

    // --- Tabela: sync_agents -----------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_sync_agents (
            id              INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            name            VARCHAR(255) NOT NULL,
            address         VARCHAR(45) NOT NULL COMMENT 'IP do agente remoto',
            port            INT UNSIGNED NOT NULL DEFAULT 45678,
            paired          TINYINT(1) NOT NULL DEFAULT 0,
            token_hash      VARCHAR(128) COMMENT 'SHA-256 do token de sessão',
            last_seen_at    DATETIME,
            created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY uq_agent_addr_port (address, port)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("sync_agents: {}", e)))?;

    // --- Tabela: sync_folders ----------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_sync_folders (
            id              INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            name            VARCHAR(255) NOT NULL,
            local_path      TEXT NOT NULL,
            agent_id        INT UNSIGNED COMMENT 'Agente peer vinculado',
            status          ENUM('synced','syncing','pending','conflict','offline','error')
                            NOT NULL DEFAULT 'pending',
            last_sync_at    DATETIME,
            bytes_transferred BIGINT UNSIGNED NOT NULL DEFAULT 0,
            conflict_count  INT UNSIGNED NOT NULL DEFAULT 0,
            created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            KEY idx_sf_agent (agent_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("sync_folders: {}", e)))?;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// ---------------------------------------------------------------------------
// v6 — Tabelas de eventos e conflitos de sincronização
// ---------------------------------------------------------------------------

async fn migrate_v6_sync_tables(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v6_sync_tables";
    if is_applied(pool, NAME).await? {
        return Ok(());
    }

    log::info!("Migration {}: criando tabelas de eventos e conflitos de sync...", NAME);

    // Adiciona coluna session_token a sync_agents (token original para WS)
    let _ = sqlx::query(
        "ALTER TABLE am_sync_agents ADD COLUMN session_token VARCHAR(255) COMMENT 'Token WS em claro'",
    )
    .execute(pool)
    .await;

    // --- Tabela: sync_events -----------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_sync_events (
            id          INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            folder_id   INT UNSIGNED NOT NULL,
            event_type  ENUM('transfer','conflict','error','connected','disconnected','sync_complete')
                        NOT NULL DEFAULT 'transfer',
            path        TEXT,
            bytes       BIGINT,
            direction   ENUM('upload','download'),
            message     TEXT,
            created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            KEY idx_se_folder (folder_id),
            KEY idx_se_created (created_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("sync_events: {}", e)))?;

    // --- Tabela: sync_conflicts --------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_sync_conflicts (
            id              INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            folder_id       INT UNSIGNED NOT NULL,
            path            TEXT NOT NULL,
            local_mtime     BIGINT NOT NULL,
            remote_mtime    BIGINT NOT NULL,
            resolution      ENUM('local','remote') NOT NULL DEFAULT 'local',
            created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            KEY idx_sc_folder (folder_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("sync_conflicts: {}", e)))?;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}

// ---------------------------------------------------------------------------
// v7 — Eventos Sazonais
// ---------------------------------------------------------------------------

async fn migrate_v7_seasonal_events(pool: &DbPool) -> Result<(), DbError> {
    const NAME: &str = "v7_seasonal_events";
    if is_applied(pool, NAME).await? {
        return Ok(());
    }

    log::info!("Migration {}: criando tabelas de eventos sazonais...", NAME);

    // --- Tabela: seasonal_events ------------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_seasonal_events (
            id                          INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            name                        VARCHAR(255) NOT NULL,
            description                 TEXT,
            start_at                    DATETIME NOT NULL,
            end_at                      DATETIME NOT NULL,
            status                      ENUM('scheduled','active','finished','cancelled')
                                        NOT NULL DEFAULT 'scheduled',
            broadcast_interval_seconds  INT UNSIGNED NOT NULL DEFAULT 300,
            created_at                  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at                  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            KEY idx_se_status (status),
            KEY idx_se_start (start_at),
            KEY idx_se_end (end_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("seasonal_events: {}", e)))?;

    // --- Tabela: seasonal_event_rates ------------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_seasonal_event_rates (
            id                  INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            event_id            INT UNSIGNED NOT NULL,
            xp_multiplier       DOUBLE NOT NULL DEFAULT 1.0,
            harvest_multiplier  DOUBLE NOT NULL DEFAULT 1.0,
            taming_multiplier   DOUBLE NOT NULL DEFAULT 1.0,
            breeding_multiplier DOUBLE NOT NULL DEFAULT 1.0,
            quality_multiplier  DOUBLE NOT NULL DEFAULT 1.0,
            UNIQUE KEY uq_event_rates (event_id),
            KEY idx_er_event (event_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("seasonal_event_rates: {}", e)))?;

    // --- Tabela: seasonal_event_servers ----------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_seasonal_event_servers (
            id          INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            event_id    INT UNSIGNED NOT NULL,
            server_id   INT UNSIGNED NOT NULL,
            UNIQUE KEY uq_event_server (event_id, server_id),
            KEY idx_ess_event (event_id),
            KEY idx_ess_server (server_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("seasonal_event_servers: {}", e)))?;

    // --- Tabela: seasonal_event_backups ----------------------------------
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS am_seasonal_event_backups (
            id                      INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            event_id                INT UNSIGNED NOT NULL,
            server_id               INT UNSIGNED NOT NULL,
            gus_backup_path         TEXT NOT NULL,
            game_ini_backup_path    TEXT NOT NULL,
            created_at              DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY uq_event_server_backup (event_id, server_id),
            KEY idx_eb_event (event_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::Migration(format!("seasonal_event_backups: {}", e)))?;

    mark_applied(pool, NAME).await?;
    log::info!("Migration {} aplicada.", NAME);
    Ok(())
}