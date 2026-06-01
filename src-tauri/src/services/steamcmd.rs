use std::path::{Path, PathBuf};
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// URL de download do SteamCMD para Windows.
const STEAMCMD_URL: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip";

/// App ID do ARK: Survival Evolved Dedicated Server no Steam.
pub const ARK_APP_ID: &str = "376030";

#[derive(Debug, Error)]
pub enum SteamCmdError {
    #[error("SteamCMD não encontrado em: {0}")]
    NotFound(String),
    #[error("Falha ao baixar SteamCMD: {0}")]
    DownloadFailed(String),
    #[error("Falha ao extrair SteamCMD: {0}")]
    ExtractFailed(String),
    #[error("Erro ao executar SteamCMD: {0}")]
    ExecutionFailed(String),
    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),
}

/// Verifica se o SteamCMD está instalado no caminho informado.
pub fn is_installed(steamcmd_dir: &Path) -> bool {
    steamcmd_dir.join("steamcmd.exe").exists()
}

/// Faz o download e extrai o SteamCMD no diretório informado.
/// Emite progresso via callback `on_output(linha)`.
pub async fn install<F>(steamcmd_dir: &Path, on_output: F) -> Result<(), SteamCmdError>
where
    F: Fn(String) + Send + 'static,
{
    tokio::fs::create_dir_all(steamcmd_dir).await?;

    on_output("Baixando SteamCMD...".to_string());

    let zip_path = steamcmd_dir.join("steamcmd.zip");

    // Download
    let response = reqwest::get(STEAMCMD_URL)
        .await
        .map_err(|e| SteamCmdError::DownloadFailed(e.to_string()))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| SteamCmdError::DownloadFailed(e.to_string()))?;

    tokio::fs::write(&zip_path, &bytes).await?;
    on_output("Download concluído. Extraindo...".to_string());

    // Extração em blocking thread para não bloquear o runtime
    let zip_path_clone = zip_path.clone();
    let dest_dir = steamcmd_dir.to_path_buf();
    tokio::task::spawn_blocking(move || -> Result<(), SteamCmdError> {
        let file = std::fs::File::open(&zip_path_clone)
            .map_err(|e| SteamCmdError::ExtractFailed(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| SteamCmdError::ExtractFailed(e.to_string()))?;
        archive
            .extract(&dest_dir)
            .map_err(|e| SteamCmdError::ExtractFailed(e.to_string()))?;
        Ok(())
    })
    .await
    .map_err(|e| SteamCmdError::ExtractFailed(e.to_string()))??;

    // Remove o zip após extração
    let _ = tokio::fs::remove_file(&zip_path).await;

    on_output("SteamCMD instalado com sucesso.".to_string());
    Ok(())
}

/// Instala ou atualiza um servidor via SteamCMD.
/// `install_dir`: destino da instalação do servidor.
/// `on_output`: callback chamado para cada linha de saída do SteamCMD.
pub async fn update_server<F>(
    steamcmd_dir: &Path,
    install_dir: &Path,
    app_id: &str,
    on_output: F,
) -> Result<(), SteamCmdError>
where
    F: Fn(String) + Send + 'static,
{
    let exe = steamcmd_dir.join("steamcmd.exe");
    if !exe.exists() {
        return Err(SteamCmdError::NotFound(exe.display().to_string()));
    }

    tokio::fs::create_dir_all(install_dir).await?;

    let mut child = Command::new(&exe)
        .args([
            "+force_install_dir",
            &install_dir.display().to_string(),
            "+login",
            "anonymous",
            "+app_update",
            app_id,
            "validate",
            "+quit",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| SteamCmdError::ExecutionFailed(e.to_string()))?;

    // Stream stdout em tempo real
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            on_output(line);
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| SteamCmdError::ExecutionFailed(e.to_string()))?;

    if !status.success() {
        return Err(SteamCmdError::ExecutionFailed(format!(
            "SteamCMD encerrou com código: {:?}",
            status.code()
        )));
    }

    Ok(())
}

/// Retorna o caminho do executável SteamCMD dado o diretório base.
pub fn exe_path(steamcmd_dir: &Path) -> PathBuf {
    steamcmd_dir.join("steamcmd.exe")
}

