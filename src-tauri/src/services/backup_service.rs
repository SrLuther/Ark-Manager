use chrono::Local;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("Diretório de origem não encontrado: {0}")]
    SourceNotFound(String),
    #[error("Erro ao criar diretório de backup: {0}")]
    DirCreate(String),
    #[error("Erro ao copiar arquivos: {0}")]
    CopyError(String),
    #[error("Backup não encontrado: {0}")]
    BackupNotFound(String),
    #[error("Erro ao restaurar backup: {0}")]
    RestoreError(String),
    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),
}

/// Resultado de um backup criado.
#[derive(Debug)]
pub struct BackupResult {
    /// Caminho do diretório de backup criado.
    pub backup_path: PathBuf,
    /// Tamanho total em bytes.
    pub size_bytes: u64,
    /// Número de arquivos copiados.
    pub file_count: u64,
}

/// Cria um backup da pasta SavedArks com timestamp.
///
/// `saved_arks_dir`: `<install>/ShooterGame/Saved/SavedArks`
/// `backup_base_dir`: diretório raiz onde os backups são armazenados
/// `server_id`: identificador para nomear a pasta de backup
pub async fn create_backup(
    saved_arks_dir: &Path,
    backup_base_dir: &Path,
    server_id: &str,
) -> Result<BackupResult, BackupError> {
    if !saved_arks_dir.exists() {
        return Err(BackupError::SourceNotFound(
            saved_arks_dir.display().to_string(),
        ));
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("{}_{}", server_id, timestamp);
    let backup_path = backup_base_dir.join(&backup_name);

    tokio::fs::create_dir_all(&backup_path)
        .await
        .map_err(|e| BackupError::DirCreate(e.to_string()))?;

    let src = saved_arks_dir.to_path_buf();
    let dst = backup_path.clone();

    // Copia em blocking thread (operação pesada de I/O)
    let result = tokio::task::spawn_blocking(move || copy_dir_recursive(&src, &dst))
        .await
        .map_err(|e| BackupError::CopyError(e.to_string()))??;

    Ok(BackupResult {
        backup_path,
        size_bytes: result.0,
        file_count: result.1,
    })
}

/// Restaura um backup sobrescrevendo o diretório SavedArks.
pub async fn restore_backup(
    backup_path: &Path,
    saved_arks_dir: &Path,
) -> Result<(), BackupError> {
    if !backup_path.exists() {
        return Err(BackupError::BackupNotFound(
            backup_path.display().to_string(),
        ));
    }

    // Remove o diretório atual
    if saved_arks_dir.exists() {
        tokio::fs::remove_dir_all(saved_arks_dir)
            .await
            .map_err(|e| BackupError::RestoreError(e.to_string()))?;
    }

    tokio::fs::create_dir_all(saved_arks_dir)
        .await
        .map_err(|e| BackupError::RestoreError(e.to_string()))?;

    let src = backup_path.to_path_buf();
    let dst = saved_arks_dir.to_path_buf();

    tokio::task::spawn_blocking(move || copy_dir_recursive(&src, &dst))
        .await
        .map_err(|e| BackupError::RestoreError(e.to_string()))??;

    Ok(())
}

/// Copia recursivamente um diretório. Retorna (bytes_copiados, arquivos_copiados).
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(u64, u64), BackupError> {
    let mut total_bytes = 0u64;
    let mut total_files = 0u64;

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let src_path = entry.path();
        let relative = src_path
            .strip_prefix(src)
            .map_err(|e| BackupError::CopyError(e.to_string()))?;
        let dst_path = dst.join(relative);

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path)
                .map_err(|e| BackupError::CopyError(e.to_string()))?;
        } else {
            std::fs::copy(src_path, &dst_path)
                .map_err(|e| BackupError::CopyError(format!("{}: {}", src_path.display(), e)))?;
            total_bytes += src_path.metadata().map(|m| m.len()).unwrap_or(0);
            total_files += 1;
        }
    }

    Ok((total_bytes, total_files))
}

/// Lista os backups disponíveis em um diretório base para um servidor.
pub fn list_backups(backup_base_dir: &Path, server_id: &str) -> Vec<PathBuf> {
    if !backup_base_dir.exists() {
        return Vec::new();
    }

    let prefix = format!("{}_", server_id);
    let mut backups: Vec<PathBuf> = std::fs::read_dir(backup_base_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(&prefix))
                    .unwrap_or(false)
        })
        .collect();

    // Ordena por nome (que contém timestamp)
    backups.sort();
    backups
}

/// Remove backups antigos mantendo apenas os `keep_count` mais recentes.
pub fn prune_old_backups(
    backup_base_dir: &Path,
    server_id: &str,
    keep_count: usize,
) -> Result<usize, BackupError> {
    let mut backups = list_backups(backup_base_dir, server_id);
    if backups.len() <= keep_count {
        return Ok(0);
    }

    let to_remove = backups.len() - keep_count;
    let mut removed = 0;

    for path in backups.drain(..to_remove) {
        std::fs::remove_dir_all(&path)
            .map_err(|e| BackupError::CopyError(format!("{}: {}", path.display(), e)))?;
        removed += 1;
    }

    Ok(removed)
}

/// Retorna o caminho padrão do SavedArks dado o diretório de instalação.
pub fn saved_arks_path(install_dir: &Path) -> PathBuf {
    install_dir
        .join("ShooterGame")
        .join("Saved")
        .join("SavedArks")
}

