pub mod commands;
pub mod db;
pub mod models;
pub mod services;
pub mod utils;

use commands::{
    agent::AgentRuntimeState,
    logs::new_watcher_map,
    rcon::new_rcon_map,
};
use db::DbPool;
use services::{
    agent_auth::PairingState,
    event_scheduler::{SchedulerState, SchedulerStateArc},
    process_manager::new_pid_map,
    sync_engine::SyncEngineState,
};
use std::sync::Arc;
use tauri::{Emitter, Manager};

/// Estado global da aplicação, compartilhado via Tauri State.
pub struct AppState {
    pub db: DbPool,
}

/// Retorna o nome identificador deste agente ARK Manager na rede.
fn local_agent_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .map(|h| format!("ARK Manager on {}", h))
        .unwrap_or_else(|_| "ARK Manager".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Cria estado de runtime do agente antes do setup Tauri
    let agent_name = local_agent_name();
    let pairing = Arc::new(PairingState::new());
    let agent_runtime = AgentRuntimeState::new(agent_name.clone(), pairing.clone());
    let discovered_arc = agent_runtime.discovered.clone();
    let sessions_arc = agent_runtime.sessions.clone();

    // shared_db: preenchido após inicialização do banco, usado pelo servidor WS
    let shared_db: Arc<tokio::sync::RwLock<Option<DbPool>>> =
        Arc::new(tokio::sync::RwLock::new(None));
    let shared_db_server = shared_db.clone();
    let shared_db_setup = shared_db.clone();

    // Motor de sincronização
    let sync_engine = Arc::new(SyncEngineState::new());

    // Estado do scheduler de eventos sazonais
    let event_scheduler_state: SchedulerStateArc = Arc::new(SchedulerState::new());
    let event_scheduler_for_spawn = event_scheduler_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .manage(new_pid_map())
        .manage(new_rcon_map())
        .manage(new_watcher_map())
        .manage(agent_runtime)
        .manage(sync_engine)
        .manage(event_scheduler_state)
        .invoke_handler(tauri::generate_handler![
            // Servidor
            commands::server::list_servers,
            commands::server::get_server,
            commands::server::create_server,
            commands::server::update_server,
            commands::server::delete_server,
            commands::server::start_server,
            commands::server::stop_server,
            commands::server::restart_server,
            commands::server::server_status,
            // Instalação
            commands::install::install_steamcmd,
            commands::install::is_steamcmd_installed,
            commands::install::install_ark_server,
            commands::install::update_ark_server,
            commands::install::is_server_installed,
            // Configuração INI
            commands::config::read_game_user_settings,
            commands::config::read_game_ini,
            commands::config::save_server_config,
            commands::config::get_config_dir,
            // RCON
            commands::rcon::rcon_connect,
            commands::rcon::rcon_send_command,
            commands::rcon::rcon_disconnect,
            commands::rcon::rcon_execute,
            commands::rcon::rcon_is_connected,
            // Logs
            commands::logs::start_log_watcher,
            commands::logs::stop_log_watcher,
            commands::logs::is_log_watcher_active,
            // Mods
            commands::mods::list_mods,
            commands::mods::add_mod,
            commands::mods::remove_mod,
            commands::mods::reorder_mods,
            // Cluster
            commands::cluster::list_clusters,
            commands::cluster::get_cluster,
            commands::cluster::create_cluster,
            commands::cluster::delete_cluster,
            commands::cluster::assign_server_to_cluster,
            commands::cluster::unassign_server_from_cluster,
            // Backup
            commands::backup::list_backups,
            commands::backup::create_backup,
            commands::backup::restore_backup,
            commands::backup::prune_backups,
            // Scheduler
            commands::scheduler::list_tasks,
            commands::scheduler::create_task,
            commands::scheduler::update_task,
            commands::scheduler::delete_task,
            commands::scheduler::validate_cron_expression,
            // Hardware
            commands::hardware::get_system_metrics,
            commands::hardware::get_process_metrics,
            commands::hardware::find_server_process,
            // Importação
            commands::import::detect_existing_server,
            commands::import::import_server,
            // Agente de rede
            commands::agent::discover_agents,
            commands::agent::list_agents,
            commands::agent::pair_agent,
            commands::agent::remove_agent,
            commands::agent::get_agent_status,
            commands::agent::generate_pairing_code,
            // Sincronização
            commands::sync::list_sync_folders,
            commands::sync::add_sync_folder,
            commands::sync::remove_sync_folder,
            commands::sync::force_sync,
            commands::sync::get_sync_events,
            commands::sync::get_sync_conflicts,
            // Discord
            commands::discord::save_discord_config,
            commands::discord::get_discord_config,
            commands::discord::test_discord_webhook,
            // Banco de dados
            commands::database::setup_database,
            // Eventos sazonais
            commands::seasonal_events::list_seasonal_events,
            commands::seasonal_events::get_seasonal_event,
            commands::seasonal_events::create_seasonal_event,
            commands::seasonal_events::update_seasonal_event,
            commands::seasonal_events::cancel_seasonal_event,
            commands::seasonal_events::force_start_event,
            commands::seasonal_events::force_end_event,
            commands::seasonal_events::get_event_status,
            // Rede
            commands::network::detect_port_conflicts,
            commands::network::suggest_available_port,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // Inicia servidor HTTP/WebSocket do agente (independente do banco)
            let agent_name_server = agent_name.clone();
            let pairing_server = pairing.clone();
            let sessions_server = sessions_arc.clone();
            tauri::async_runtime::spawn(async move {
                services::agent_server::start_agent_server(
                    agent_name_server,
                    pairing_server,
                    sessions_server,
                    shared_db_server,
                )
                .await;
            });

            // Inicia discovery UDP
            let agent_name_disc = agent_name.clone();
            tauri::async_runtime::spawn(async move {
                services::agent_discovery::start_discovery(
                    agent_name_disc,
                    services::agent_server::AGENT_SERVER_PORT,
                    discovered_arc,
                )
                .await;
            });

            // Inicializa o banco de dados SQLite de forma assíncrona
            tauri::async_runtime::spawn(async move {
                match db::initialize().await {
                    Ok(pool) => {
                        log::info!("Banco de dados SQLite inicializado com sucesso.");
                        *shared_db_setup.write().await = Some(pool.clone());
                        services::event_scheduler::start_event_scheduler(
                            pool.clone(),
                            event_scheduler_for_spawn,
                        );
                        handle.manage(AppState { db: pool });
                        let _ = handle.emit("db:ready", ());
                    }
                    Err(e) => {
                        log::error!("Falha ao inicializar banco de dados: {}", e);
                        let _ = handle.emit("db:error", format!("{}", e));
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar Ark Manager");
}

