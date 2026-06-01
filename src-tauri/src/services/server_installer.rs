use std::path::Path;
use thiserror::Error;

use super::steamcmd::{self, SteamCmdError, ARK_APP_ID};

#[derive(Debug, Error)]
pub enum InstallerError {
    #[error("Erro no SteamCMD: {0}")]
    SteamCmd(#[from] SteamCmdError),
    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),
}

/// Instala o SteamCMD caso não esteja presente, depois instala/atualiza o servidor ARK.
/// Emite progresso via callback `on_output(linha)`.
pub async fn install_server<F>(
    steamcmd_dir: &Path,
    install_dir: &Path,
    on_output: F,
) -> Result<(), InstallerError>
where
    F: Fn(String) + Clone + Send + 'static,
{
    // Garante que o SteamCMD está instalado
    if !steamcmd::is_installed(steamcmd_dir) {
        let cb = on_output.clone();
        steamcmd::install(steamcmd_dir, move |line| cb(line)).await?;
    }

    on_output(format!(
        "Iniciando instalação do servidor ARK (App ID {})...",
        ARK_APP_ID
    ));

    steamcmd::update_server(steamcmd_dir, install_dir, ARK_APP_ID, on_output).await?;

    Ok(())
}

/// Atualiza um servidor ARK já instalado.
pub async fn update_server<F>(
    steamcmd_dir: &Path,
    install_dir: &Path,
    on_output: F,
) -> Result<(), InstallerError>
where
    F: Fn(String) + Send + 'static,
{
    if !steamcmd::is_installed(steamcmd_dir) {
        return Err(InstallerError::SteamCmd(SteamCmdError::NotFound(
            steamcmd_dir.display().to_string(),
        )));
    }

    steamcmd::update_server(steamcmd_dir, install_dir, ARK_APP_ID, on_output).await?;
    Ok(())
}

/// Verifica se o servidor ARK está instalado no caminho informado.
pub fn is_server_installed(install_dir: &Path) -> bool {
    install_dir
        .join("ShooterGame")
        .join("Binaries")
        .join("Win64")
        .join("ShooterGameServer.exe")
        .exists()
}

/// Retorna o caminho do executável do servidor dado o diretório de instalação.
pub fn server_exe_path(install_dir: &Path) -> std::path::PathBuf {
    install_dir
        .join("ShooterGame")
        .join("Binaries")
        .join("Win64")
        .join("ShooterGameServer.exe")
}

