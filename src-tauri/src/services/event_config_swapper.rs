/// Serviço responsável por:
/// 1. Fazer backup dos INIs originais de um servidor antes de aplicar um evento.
/// 2. Aplicar as taxas do evento sobrescrevendo os INIs via write_utf16le.
/// 3. Restaurar os INIs originais ao encerrar o evento.
///
/// Proteção de integridade: apply_event_rates só avança se o backup for confirmado.

use std::path::{Path, PathBuf};
use tokio::fs;

use crate::db::DbPool;
use crate::models::seasonal_event::EventRate;

/// Erros do config swapper.
#[derive(Debug)]
pub enum ConfigSwapError {
    Io(String),
    DbError(String),
    BackupAlreadyExists,
    BackupNotFound,
}

impl std::fmt::Display for ConfigSwapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O: {}", e),
            Self::DbError(e) => write!(f, "DB: {}", e),
            Self::BackupAlreadyExists => write!(f, "Backup de INI já existe para este evento/servidor"),
            Self::BackupNotFound => write!(f, "Backup de INI não encontrado para restauração"),
        }
    }
}

/// Retorna o diretório de configuração do servidor ARK.
fn config_dir(install_path: &str) -> PathBuf {
    PathBuf::from(install_path)
        .join("ShooterGame")
        .join("Saved")
        .join("Config")
        .join("WindowsServer")
}

/// Faz backup dos INIs e registra os caminhos no banco.
/// Proteção: se backup já existe no banco para (event_id, server_id), retorna erro.
pub async fn backup_ini_files(
    pool: &DbPool,
    event_id: u32,
    server_id: u32,
    install_path: &str,
) -> Result<(), ConfigSwapError> {
    // Verifica se backup já existe
    let existing: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM seasonal_event_backups WHERE event_id = ? AND server_id = ?",
    )
    .bind(event_id)
    .bind(server_id)
    .fetch_one(pool)
    .await
    .map_err(|e| ConfigSwapError::DbError(e.to_string()))?;

    if existing.0 > 0 {
        return Err(ConfigSwapError::BackupAlreadyExists);
    }

    let cfg_dir = config_dir(install_path);
    let gus_src = cfg_dir.join("GameUserSettings.ini");
    let game_ini_src = cfg_dir.join("Game.ini");

    let gus_bak = cfg_dir.join(format!("GameUserSettings.ini.event_{}.bak", event_id));
    let game_ini_bak = cfg_dir.join(format!("Game.ini.event_{}.bak", event_id));

    fs::copy(&gus_src, &gus_bak)
        .await
        .map_err(|e| ConfigSwapError::Io(format!("cópia GUS: {}", e)))?;

    fs::copy(&game_ini_src, &game_ini_bak)
        .await
        .map_err(|e| ConfigSwapError::Io(format!("cópia Game.ini: {}", e)))?;

    // Registra no banco
    sqlx::query(
        "INSERT INTO seasonal_event_backups (event_id, server_id, gus_backup_path, game_ini_backup_path)
         VALUES (?, ?, ?, ?)",
    )
    .bind(event_id)
    .bind(server_id)
    .bind(gus_bak.to_string_lossy().as_ref())
    .bind(game_ini_bak.to_string_lossy().as_ref())
    .execute(pool)
    .await
    .map_err(|e| ConfigSwapError::DbError(e.to_string()))?;

    log::info!(
        "Backup INI: evento {} servidor {} em {:?}",
        event_id, server_id, cfg_dir
    );
    Ok(())
}

/// Aplica as taxas do evento sobrescrevendo os INIs do servidor.
/// Só deve ser chamado após `backup_ini_files` confirmar sucesso.
pub async fn apply_event_rates(
    install_path: &str,
    rates: &EventRate,
) -> Result<(), ConfigSwapError> {
    let cfg_dir = config_dir(install_path);
    let gus_path = cfg_dir.join("GameUserSettings.ini");
    let game_ini_path = cfg_dir.join("Game.ini");

    let gus_content = build_gus_with_rates(rates);
    let game_ini_content = build_game_ini_with_rates(rates);

    write_utf16le(&gus_path, &gus_content)
        .await
        .map_err(|e| ConfigSwapError::Io(e))?;

    write_utf16le(&game_ini_path, &game_ini_content)
        .await
        .map_err(|e| ConfigSwapError::Io(e))?;

    log::info!(
        "Taxas do evento aplicadas em {:?}: XP={} Harvest={} Taming={} Breeding={} Quality={}",
        cfg_dir,
        rates.xp_multiplier,
        rates.harvest_multiplier,
        rates.taming_multiplier,
        rates.breeding_multiplier,
        rates.quality_multiplier,
    );
    Ok(())
}

/// Restaura os INIs originais a partir do backup registrado no banco.
/// Remove o backup após restauração bem-sucedida.
pub async fn restore_ini_files(
    pool: &DbPool,
    event_id: u32,
    server_id: u32,
) -> Result<(), ConfigSwapError> {
    let backup: Option<(String, String)> = sqlx::query_as(
        "SELECT gus_backup_path, game_ini_backup_path FROM seasonal_event_backups
         WHERE event_id = ? AND server_id = ?",
    )
    .bind(event_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ConfigSwapError::DbError(e.to_string()))?;

    let (gus_bak, game_ini_bak) = backup.ok_or(ConfigSwapError::BackupNotFound)?;
    let gus_bak = PathBuf::from(&gus_bak);
    let game_ini_bak = PathBuf::from(&game_ini_bak);

    if !gus_bak.exists() || !game_ini_bak.exists() {
        return Err(ConfigSwapError::BackupNotFound);
    }

    let gus_dst = gus_bak.with_extension("").with_extension("ini");
    let game_ini_dst = game_ini_bak.with_extension("").with_extension("ini");

    // Restaura removendo o sufixo .event_N.bak
    // Destinos corretos: GameUserSettings.ini e Game.ini no mesmo diretório
    let cfg_dir = gus_bak.parent().unwrap_or(Path::new("."));
    let gus_orig = cfg_dir.join("GameUserSettings.ini");
    let game_ini_orig = cfg_dir.join("Game.ini");

    let _ = gus_dst; // silencia unused warning
    let _ = game_ini_dst;

    fs::copy(&gus_bak, &gus_orig)
        .await
        .map_err(|e| ConfigSwapError::Io(format!("restaurar GUS: {}", e)))?;

    fs::copy(&game_ini_bak, &game_ini_orig)
        .await
        .map_err(|e| ConfigSwapError::Io(format!("restaurar Game.ini: {}", e)))?;

    // Remove backups
    let _ = fs::remove_file(&gus_bak).await;
    let _ = fs::remove_file(&game_ini_bak).await;

    // Remove registro do banco
    sqlx::query(
        "DELETE FROM seasonal_event_backups WHERE event_id = ? AND server_id = ?",
    )
    .bind(event_id)
    .bind(server_id)
    .execute(pool)
    .await
    .map_err(|e| ConfigSwapError::DbError(e.to_string()))?;

    log::info!(
        "INIs restaurados: evento {} servidor {}",
        event_id, server_id
    );
    Ok(())
}

// ─────────────────────────────────────────────
// Helpers de geração de conteúdo INI
// ─────────────────────────────────────────────

fn build_gus_with_rates(rates: &EventRate) -> String {
    format!(
        "[ServerSettings]\r\nXPMultiplier={xp}\r\nHarvestAmountMultiplier={harvest}\r\nTamingSpeedMultiplier={taming}\r\nMatingIntervalMultiplier={breeding_inv}\r\nEggHatchSpeedMultiplier={breeding}\r\nBabyMatureSpeedMultiplier={breeding}\r\n",
        xp = rates.xp_multiplier,
        harvest = rates.harvest_multiplier,
        taming = rates.taming_multiplier,
        breeding = rates.breeding_multiplier,
        breeding_inv = 1.0 / rates.breeding_multiplier.max(0.01),
    )
}

fn build_game_ini_with_rates(rates: &EventRate) -> String {
    format!(
        "[/script/shootergame.shootergamemode]\r\nSupplyDropQualityMultiplier={quality}\r\n",
        quality = rates.quality_multiplier,
    )
}

/// Escreve um arquivo como UTF-16 LE com BOM (padrão dos INIs ARK).
async fn write_utf16le(path: &Path, content: &str) -> Result<(), String> {
    let mut bytes: Vec<u8> = vec![0xFF, 0xFE]; // BOM UTF-16 LE
    for c in content.encode_utf16() {
        bytes.extend_from_slice(&c.to_le_bytes());
    }
    fs::write(path, &bytes)
        .await
        .map_err(|e| e.to_string())
}
