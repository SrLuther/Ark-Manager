use crate::models::server::{CreateServerRequest, ServerResponse, UpdateServerRequest};
use crate::services::{
    launch_builder::{self, LaunchParams},
    process_manager::{self, PidMap},
};
use crate::AppState;
use std::path::Path;
use tauri::State;

/// Lista todos os servidores cadastrados.
#[tauri::command]
pub async fn list_servers(state: State<'_, AppState>) -> Result<Vec<ServerResponse>, String> {
    let rows = sqlx::query_as::<_, crate::models::server::Server>(
        "SELECT * FROM servers ORDER BY startup_priority ASC, name ASC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(ServerResponse::from).collect())
}

/// Busca um servidor pelo ID.
#[tauri::command]
pub async fn get_server(id: u32, state: State<'_, AppState>) -> Result<ServerResponse, String> {
    let row = sqlx::query_as::<_, crate::models::server::Server>(
        "SELECT * FROM servers WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ServerResponse::from(row))
}

/// Cria um novo servidor.
#[tauri::command]
pub async fn create_server(
    req: CreateServerRequest,
    state: State<'_, AppState>,
) -> Result<ServerResponse, String> {
    sqlx::query(
        r#"INSERT INTO servers
        (name, install_path, map_name, session_name, game_port, query_port, rcon_port,
         rcon_enabled, max_players, server_password, admin_password, spectator_password,
         ip_address, mods, cluster_id, enable_pvp, enable_battleye, enable_crosshair,
         allow_third_person, allow_tribe_alliances, custom_args, auto_start, auto_restart,
         startup_delay, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'stopped', NOW(), NOW())"#,
    )
    .bind(&req.name)
    .bind(&req.install_path)
    .bind(&req.map_name)
    .bind(&req.session_name)
    .bind(req.game_port)
    .bind(req.query_port)
    .bind(req.rcon_port)
    .bind(req.rcon_enabled.unwrap_or(true))
    .bind(req.max_players.unwrap_or(70))
    .bind(&req.server_password)
    .bind(&req.admin_password)
    .bind(&req.spectator_password)
    .bind(&req.ip_address)
    .bind(&req.mods)
    .bind(req.cluster_id)
    .bind(req.enable_pvp.unwrap_or(false))
    .bind(req.enable_battleye.unwrap_or(true))
    .bind(req.enable_crosshair.unwrap_or(false))
    .bind(req.allow_third_person.unwrap_or(false))
    .bind(req.allow_tribe_alliances.unwrap_or(true))
    .bind(&req.custom_args)
    .bind(req.auto_start.unwrap_or(false))
    .bind(req.auto_restart.unwrap_or(false))
    .bind(req.startup_delay.unwrap_or(0))
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let row = sqlx::query_as::<_, crate::models::server::Server>(
        "SELECT * FROM servers ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ServerResponse::from(row))
}

/// Atualiza os dados de um servidor.
#[tauri::command]
pub async fn update_server(
    id: u32,
    req: UpdateServerRequest,
    state: State<'_, AppState>,
) -> Result<ServerResponse, String> {
    sqlx::query(
        r#"UPDATE servers SET
        name = COALESCE(?, name),
        install_path = COALESCE(?, install_path),
        map_name = COALESCE(?, map_name),
        session_name = COALESCE(?, session_name),
        game_port = COALESCE(?, game_port),
        query_port = COALESCE(?, query_port),
        rcon_port = COALESCE(?, rcon_port),
        rcon_enabled = COALESCE(?, rcon_enabled),
        max_players = COALESCE(?, max_players),
        server_password = COALESCE(?, server_password),
        admin_password = COALESCE(?, admin_password),
        spectator_password = COALESCE(?, spectator_password),
        ip_address = COALESCE(?, ip_address),
        mods = COALESCE(?, mods),
        cluster_id = COALESCE(?, cluster_id),
        enable_pvp = COALESCE(?, enable_pvp),
        enable_battleye = COALESCE(?, enable_battleye),
        enable_crosshair = COALESCE(?, enable_crosshair),
        allow_third_person = COALESCE(?, allow_third_person),
        allow_tribe_alliances = COALESCE(?, allow_tribe_alliances),
        custom_args = COALESCE(?, custom_args),
        auto_start = COALESCE(?, auto_start),
        auto_restart = COALESCE(?, auto_restart),
        startup_delay = COALESCE(?, startup_delay),
        startup_priority = COALESCE(?, startup_priority),
        intelligent_mode = COALESCE(?, intelligent_mode),
        updated_at = NOW()
        WHERE id = ?"#,
    )
    .bind(&req.name)
    .bind(&req.install_path)
    .bind(&req.map_name)
    .bind(&req.session_name)
    .bind(req.game_port)
    .bind(req.query_port)
    .bind(req.rcon_port)
    .bind(req.rcon_enabled)
    .bind(req.max_players)
    .bind(&req.server_password)
    .bind(&req.admin_password)
    .bind(&req.spectator_password)
    .bind(&req.ip_address)
    .bind(&req.mods)
    .bind(req.cluster_id)
    .bind(req.enable_pvp)
    .bind(req.enable_battleye)
    .bind(req.enable_crosshair)
    .bind(req.allow_third_person)
    .bind(req.allow_tribe_alliances)
    .bind(&req.custom_args)
    .bind(req.auto_start)
    .bind(req.auto_restart)
    .bind(req.startup_delay)
    .bind(req.startup_priority)
    .bind(req.intelligent_mode)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    get_server(id, state).await
}

/// Remove um servidor (apenas se estiver parado).
#[tauri::command]
pub async fn delete_server(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    sqlx::query("DELETE FROM servers WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Inicia o servidor (gera RunServer.cmd e executa).
#[tauri::command]
pub async fn start_server(
    id: u32,
    pid_map: State<'_, PidMap>,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let server = sqlx::query_as::<_, crate::models::server::Server>(
        "SELECT * FROM servers WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let install_dir = Path::new(&server.install_path);
    let script_dir = install_dir.join("ArkManager");

    let mods: Vec<String> = server.mod_ids();
    let params = LaunchParams {
        map: server.map_name.clone(),
        game_port: server.game_port,
        query_port: server.query_port,
        rcon_port: server.rcon_port,
        rcon_password: server.admin_password.clone(),
        max_players: server.max_players as u32,
        server_password: server.server_password.clone().unwrap_or_default(),
        admin_password: server.admin_password.clone(),
        mods,
        cluster_id: String::new(),
        cluster_dir: String::new(),
        pvp: server.enable_pvp,
        no_battleye: !server.enable_battleye,
        extra_args: Vec::new(),
    };

    let script_path = launch_builder::generate_launch_script(install_dir, &script_dir, &params)
        .await
        .map_err(|e| e.to_string())?;

    let pid = process_manager::start_server(&id.to_string(), &script_path, &pid_map)
        .await
        .map_err(|e| e.to_string())?;

    // Atualiza status e PID no banco
    sqlx::query("UPDATE servers SET status = 'starting', pid = ?, last_started = NOW(), updated_at = NOW() WHERE id = ?")
        .bind(pid)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(pid)
}

/// Para o servidor.
#[tauri::command]
pub async fn stop_server(
    id: u32,
    pid_map: State<'_, PidMap>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    process_manager::stop_server(&id.to_string(), &pid_map)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE servers SET status = 'stopped', pid = NULL, last_stopped = NOW(), updated_at = NOW() WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Reinicia o servidor.
#[tauri::command]
pub async fn restart_server(
    id: u32,
    pid_map: State<'_, PidMap>,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let server = sqlx::query_as::<_, crate::models::server::Server>(
        "SELECT * FROM servers WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let install_dir = Path::new(&server.install_path);
    let script_dir = install_dir.join("ArkManager");
    let script_path = launch_builder::script_path(&script_dir);

    let pid = process_manager::restart_server(&id.to_string(), &script_path, &pid_map)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("UPDATE servers SET status = 'restarting', pid = ?, updated_at = NOW() WHERE id = ?")
        .bind(pid)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(pid)
}

/// Verifica se o servidor está em execução.
#[tauri::command]
pub async fn server_status(
    id: u32,
    pid_map: State<'_, PidMap>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let running = process_manager::is_running(&id.to_string(), &pid_map).await;

    if !running {
        // Garante que o banco reflita o estado real
        sqlx::query(
            "UPDATE servers SET status = 'stopped', pid = NULL, updated_at = NOW() WHERE id = ? AND status NOT IN ('stopped')",
        )
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok("stopped".to_string())
    } else {
        let row = sqlx::query_as::<_, crate::models::server::Server>(
            "SELECT * FROM servers WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row.status)
    }
}

