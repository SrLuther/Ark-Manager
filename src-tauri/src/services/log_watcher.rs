use std::path::PathBuf;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;

/// Nível de log detectado na linha.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

/// Uma linha de log com metadados.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LogLine {
    pub server_id: String,
    pub line: String,
    pub level: LogLevel,
}

#[derive(Debug, Error)]
pub enum LogWatcherError {
    #[error("Arquivo de log não encontrado: {0}")]
    NotFound(String),
    #[error("Erro ao abrir arquivo de log: {0}")]
    OpenError(String),
}

/// Inicia o watcher de log em uma task assíncrona.
/// Emite cada nova linha via `on_line(LogLine)`.
/// Retorna um sender de shutdown: chamar `shutdown_tx.send(())` encerra o watcher.
pub fn start_watcher<F>(
    server_id: String,
    log_path: PathBuf,
    on_line: F,
) -> broadcast::Sender<()>
where
    F: Fn(LogLine) + Send + 'static,
{
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);

    tokio::spawn(async move {
        // Abre o arquivo e busca o final (tail)
        let file = match tokio::fs::File::open(&log_path).await {
            Ok(f) => f,
            Err(e) => {
                log::warn!("LogWatcher: não foi possível abrir {}: {}", log_path.display(), e);
                return;
            }
        };

        let mut reader = BufReader::new(file);

        // Avança para o final do arquivo existente (sem reemitir histórico)
        use tokio::io::AsyncSeekExt;
        let _ = reader.seek(std::io::SeekFrom::End(0)).await;

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    log::info!("LogWatcher do servidor {} encerrado.", server_id);
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {
                    let mut line = String::new();
                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => break, // EOF — aguarda próximo tick
                            Ok(_) => {
                                let trimmed = line.trim_end().to_string();
                                if !trimmed.is_empty() {
                                    let level = detect_level(&trimmed);
                                    on_line(LogLine {
                                        server_id: server_id.clone(),
                                        line: trimmed,
                                        level,
                                    });
                                }
                            }
                            Err(e) => {
                                log::warn!("LogWatcher erro de leitura: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    shutdown_tx
}

/// Detecta o nível de log com base no conteúdo da linha.
fn detect_level(line: &str) -> LogLevel {
    let lower = line.to_lowercase();
    if lower.contains("error") || lower.contains("fatal") || lower.contains("crash") {
        LogLevel::Error
    } else if lower.contains("warning") || lower.contains("warn") {
        LogLevel::Warning
    } else if lower.contains("debug") || lower.contains("verbose") {
        LogLevel::Debug
    } else {
        LogLevel::Info
    }
}

/// Retorna o caminho padrão do log do ARK dado o diretório de instalação.
pub fn default_log_path(install_dir: &std::path::Path) -> PathBuf {
    install_dir
        .join("ShooterGame")
        .join("Saved")
        .join("Logs")
        .join("ShooterGame.log")
}

