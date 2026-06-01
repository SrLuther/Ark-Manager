use crate::services::{server_installer, steamcmd};
use std::path::Path;
use tauri::{AppHandle, Emitter};

/// Instala o SteamCMD no diretório informado.
/// Emite eventos `install:output` com cada linha de progresso.
#[tauri::command]
pub async fn install_steamcmd(
    steamcmd_dir: String,
    app: AppHandle,
) -> Result<(), String> {
    let dir = Path::new(&steamcmd_dir);
    let handle = app.clone();

    steamcmd::install(dir, move |line| {
        let _ = handle.emit("install:output", line);
    })
    .await
    .map_err(|e| e.to_string())
}

/// Verifica se o SteamCMD está instalado no diretório informado.
#[tauri::command]
pub fn is_steamcmd_installed(steamcmd_dir: String) -> bool {
    steamcmd::is_installed(Path::new(&steamcmd_dir))
}

/// Instala o servidor ARK via SteamCMD.
/// Emite eventos `install:output` com cada linha de progresso.
#[tauri::command]
pub async fn install_ark_server(
    steamcmd_dir: String,
    install_dir: String,
    app: AppHandle,
) -> Result<(), String> {
    let handle = app.clone();
    server_installer::install_server(
        Path::new(&steamcmd_dir),
        Path::new(&install_dir),
        move |line| {
            let _ = handle.emit("install:output", line);
        },
    )
    .await
    .map_err(|e| e.to_string())
}

/// Atualiza o servidor ARK via SteamCMD.
/// Emite eventos `install:output` com cada linha de progresso.
#[tauri::command]
pub async fn update_ark_server(
    steamcmd_dir: String,
    install_dir: String,
    app: AppHandle,
) -> Result<(), String> {
    let handle = app.clone();
    server_installer::update_server(
        Path::new(&steamcmd_dir),
        Path::new(&install_dir),
        move |line| {
            let _ = handle.emit("install:output", line);
        },
    )
    .await
    .map_err(|e| e.to_string())
}

/// Verifica se o executável do servidor ARK está presente no diretório.
#[tauri::command]
pub fn is_server_installed(install_dir: String) -> bool {
    server_installer::is_server_installed(Path::new(&install_dir))
}

