use crate::models::server::CreateServerRequest;
use crate::services::{
    config_generator,
    ini_parser::{self, get_bool, get_i64, get_value},
    server_installer,
};
use crate::AppState;
use serde::Serialize;
use std::path::Path;
use tauri::State;

/// Resultado da detecção de um servidor existente no disco.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedServer {
    pub install_path: String,
    pub is_installed: bool,
    pub map_name: Option<String>,
    pub session_name: Option<String>,
    pub game_port: Option<u16>,
    pub query_port: Option<u16>,
    pub rcon_port: Option<u16>,
    pub max_players: Option<u16>,
    pub admin_password: Option<String>,
    pub server_password: Option<String>,
    pub mods: Option<String>,
    pub enable_pvp: Option<bool>,
}

/// Detecta e lê as configurações de um servidor existente no diretório informado.
#[tauri::command]
pub async fn detect_existing_server(install_dir: String) -> Result<DetectedServer, String> {
    let install_path = Path::new(&install_dir);
    let is_installed = server_installer::is_server_installed(install_path);

    if !is_installed {
        return Ok(DetectedServer {
            install_path: install_dir,
            is_installed: false,
            map_name: None,
            session_name: None,
            game_port: None,
            query_port: None,
            rcon_port: None,
            max_players: None,
            admin_password: None,
            server_password: None,
            mods: None,
            enable_pvp: None,
        });
    }

    let config_dir = config_generator::config_dir(install_path);
    let gus_path = config_dir.join("GameUserSettings.ini");

    let ini = ini_parser::read_ini(&gus_path).await.ok();

    let (session_name, game_port, query_port, rcon_port, max_players, admin_password,
        server_password, mods, enable_pvp) = if let Some(ref data) = ini {
        (
            get_value(data, "SessionSettings", "SessionName").map(|s| s.to_string()),
            get_i64(data, "SessionSettings", "Port", 7777) as u16,
            get_i64(data, "SessionSettings", "QueryPort", 27015) as u16,
            get_i64(data, "SessionSettings", "RCONPort", 32330) as u16,
            get_i64(data, "SessionSettings", "MaxPlayers", 70) as u16,
            get_value(data, "ServerSettings", "ServerAdminPassword").map(|s| s.to_string()),
            get_value(data, "ServerSettings", "ServerPassword").map(|s| s.to_string()),
            get_value(data, "ServerSettings", "ActiveMods").map(|s| s.to_string()),
            Some(get_bool(data, "ServerSettings", "ServerPVE", true) == false),
        )
    } else {
        (None, 7777, 27015, 32330, 70, None, None, None, None)
    };

    Ok(DetectedServer {
        install_path: install_dir,
        is_installed: true,
        map_name: None, // Mapa não é armazenado no INI — usuário informa
        session_name,
        game_port: Some(game_port),
        query_port: Some(query_port),
        rcon_port: Some(rcon_port),
        max_players: Some(max_players),
        admin_password,
        server_password,
        mods,
        enable_pvp,
    })
}

/// Importa um servidor existente criando o registro no banco.
#[tauri::command]
pub async fn import_server(
    req: CreateServerRequest,
    state: State<'_, AppState>,
) -> Result<crate::models::server::ServerResponse, String> {
    crate::commands::server::create_server(req, state).await
}

