use crate::services::{
    config_generator::{self, ServerConfig},
    ini_parser::{self, IniData},
};
use std::path::Path;

/// Lê o arquivo GameUserSettings.ini do servidor e retorna como JSON.
#[tauri::command]
pub async fn read_game_user_settings(install_dir: String) -> Result<IniData, String> {
    let config_dir = config_generator::config_dir(Path::new(&install_dir));
    let path = config_dir.join("GameUserSettings.ini");
    ini_parser::read_ini(&path).await.map_err(|e| e.to_string())
}

/// Lê o arquivo Game.ini do servidor e retorna como JSON.
#[tauri::command]
pub async fn read_game_ini(install_dir: String) -> Result<IniData, String> {
    let config_dir = config_generator::config_dir(Path::new(&install_dir));
    let path = config_dir.join("Game.ini");
    ini_parser::read_ini(&path).await.map_err(|e| e.to_string())
}

/// Gera (ou sobrescreve) os arquivos GameUserSettings.ini e Game.ini
/// com base na configuração fornecida.
#[tauri::command]
pub async fn save_server_config(
    install_dir: String,
    config: ServerConfig,
) -> Result<(), String> {
    let config_dir = config_generator::config_dir(Path::new(&install_dir));
    config_generator::generate_configs(&config_dir, &config)
        .await
        .map_err(|e| e.to_string())
}

/// Retorna o caminho do diretório de configuração do servidor.
#[tauri::command]
pub fn get_config_dir(install_dir: String) -> String {
    config_generator::config_dir(Path::new(&install_dir))
        .display()
        .to_string()
}

