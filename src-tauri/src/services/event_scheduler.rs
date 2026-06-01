/// Motor de eventos sazonais: monitora start_at/end_at, executa broadcasts RCON,
/// para/inicia servidores e aplica/restaura taxas de INI.
///
/// Itens implementados:
/// 11.1 — ciclo de vida do evento (scheduled → active → finished)
/// 11.6 — integração com process_manager (stop + saveworld + start)
/// 11.7 — broadcasts RCON automáticos (5 min antes, durante, 5 min antes do fim)
/// 11.8 — proteção de integridade (backup obrigatório antes de apply)

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::DbPool;
use crate::models::seasonal_event::{EventRate, SeasonalEvent};
use crate::services::{
    event_config_swapper as swapper,
    rcon::execute_command as rcon_exec,
};

/// Intervalo de polling do scheduler (verifica eventos a cada 30s).
const POLL_INTERVAL_SECS: u64 = 30;
/// Broadcast 5 minutos antes do início/fim.
const WARN_BEFORE_SECS: i64 = 300;

// ─────────────────────────────────────────────
// Estado compartilhado
// ─────────────────────────────────────────────

/// Identificadores de eventos atualmente em progresso (broadcastando).
#[derive(Default)]
pub struct SchedulerState {
    /// event_id → handle do tokio::task de broadcast contínuo
    pub active_broadcasts: Mutex<std::collections::HashMap<u32, tokio::task::JoinHandle<()>>>,
}

impl SchedulerState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub type SchedulerStateArc = Arc<SchedulerState>;

// ─────────────────────────────────────────────
// API pública para força-start / força-end (comandos Tauri)
// ─────────────────────────────────────────────

/// Ativa um evento imediatamente por demanda (force_start_event).
pub async fn activate_event_manual(
    pool: &DbPool,
    state: &SchedulerState,
    ev: &SeasonalEvent,
) -> Result<(), String> {
    activate_event(pool, state, ev).await
}

/// Encerra um evento imediatamente por demanda (force_end_event / cancel).
pub async fn deactivate_event_manual(
    pool: &DbPool,
    state: &SchedulerState,
    event_id: u32,
) -> Result<(), String> {
    let ev: SeasonalEvent = sqlx::query_as(
        "SELECT id, name, description, start_at, end_at, status,
                broadcast_interval_seconds, created_at, updated_at
         FROM seasonal_events WHERE id = ?",
    )
    .bind(event_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    deactivate_event(pool, state, &ev).await
}

// ─────────────────────────────────────────────
// Entrypoint: inicia o loop de polling
// ─────────────────────────────────────────────

/// Inicia o scheduler em background. Deve ser chamado uma vez no setup do Tauri.
pub fn start_event_scheduler(pool: DbPool, state: SchedulerStateArc) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = poll_events(&pool, &state).await {
                log::error!("EventScheduler poll error: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    });
}

// ─────────────────────────────────────────────
// Polling principal
// ─────────────────────────────────────────────

async fn poll_events(pool: &DbPool, state: &SchedulerState) -> Result<(), String> {
    let now = chrono_now_str();

    // Eventos agendados cuja hora de início chegou → ativar
    let to_start: Vec<SeasonalEvent> = sqlx::query_as(
        "SELECT id, name, description, start_at, end_at, status,
                broadcast_interval_seconds, created_at, updated_at
         FROM seasonal_events
         WHERE status = 'scheduled' AND start_at <= ?",
    )
    .bind(&now)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for ev in to_start {
        if let Err(e) = activate_event(pool, state, &ev).await {
            log::error!("Falha ao ativar evento {} ({}): {}", ev.id, ev.name, e);
        }
    }

    // Eventos ativos cuja hora de fim chegou → encerrar
    let to_finish: Vec<SeasonalEvent> = sqlx::query_as(
        "SELECT id, name, description, start_at, end_at, status,
                broadcast_interval_seconds, created_at, updated_at
         FROM seasonal_events
         WHERE status = 'active' AND end_at <= ?",
    )
    .bind(&now)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for ev in to_finish {
        if let Err(e) = deactivate_event(pool, state, &ev).await {
            log::error!("Falha ao encerrar evento {} ({}): {}", ev.id, ev.name, e);
        }
    }

    // Avisos de "começa em 5 min"
    let warn_start_at = chrono_offset_str(WARN_BEFORE_SECS);
    let warn_end_at = chrono_offset_str(WARN_BEFORE_SECS);

    let upcoming: Vec<(u32, String)> = sqlx::query_as(
        "SELECT id, name FROM seasonal_events
         WHERE status = 'scheduled' AND start_at BETWEEN ? AND ?",
    )
    .bind(&now)
    .bind(&warn_start_at)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for (eid, name) in upcoming {
        broadcast_to_event_servers(
            pool,
            eid,
            &format!("ServerChat Evento \"{}\" começa em 5 minutos!", name),
        )
        .await;
    }

    let ending_soon: Vec<(u32, String)> = sqlx::query_as(
        "SELECT id, name FROM seasonal_events
         WHERE status = 'active' AND end_at BETWEEN ? AND ?",
    )
    .bind(&now)
    .bind(&warn_end_at)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    for (eid, name) in ending_soon {
        broadcast_to_event_servers(
            pool,
            eid,
            &format!("ServerChat Evento \"{}\" encerra em 5 minutos!", name),
        )
        .await;
    }

    Ok(())
}

// ─────────────────────────────────────────────
// Ativar evento (11.1, 11.6, 11.8)
// ─────────────────────────────────────────────

async fn activate_event(
    pool: &DbPool,
    state: &SchedulerState,
    ev: &SeasonalEvent,
) -> Result<(), String> {
    log::info!("Ativando evento sazonal: {} ({})", ev.name, ev.id);

    let server_ids = get_event_server_ids(pool, ev.id).await?;
    let rates = get_event_rates(pool, ev.id).await?;

    for server_id in &server_ids {
        let (install_path, rcon_port, rcon_password) =
            get_server_rcon_info(pool, *server_id).await?;

        // 11.7 — Broadcast de aviso de início
        broadcast_rcon(
            &install_path,
            rcon_port,
            &rcon_password,
            &format!("ServerChat Evento \"{}\" está começando!", ev.name),
        )
        .await;

        // 11.6 — SaveWorld antes de parar
        let _ = rcon_exec("127.0.0.1", rcon_port, &rcon_password, "SaveWorld").await;
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // 11.6 — Para o servidor para trocar INIs
        stop_server_by_id(pool, *server_id).await;

        // 11.8 — Backup obrigatório antes de aplicar taxas
        match swapper::backup_ini_files(pool, ev.id, *server_id, &install_path).await {
            Ok(_) => {}
            Err(swapper::ConfigSwapError::BackupAlreadyExists) => {
                log::warn!("Backup já existe para evento {} servidor {}", ev.id, server_id);
            }
            Err(e) => {
                log::error!(
                    "PROTEÇÃO: Backup falhou para evento {} servidor {}: {}. Abortando aplicação.",
                    ev.id, server_id, e
                );
                // Reinicia o servidor sem aplicar taxas (rollback automático)
                start_server_by_id(pool, *server_id).await;
                continue;
            }
        }

        // Aplica taxas
        if let Some(ref r) = rates {
            if let Err(e) = swapper::apply_event_rates(&install_path, r).await {
                log::error!("Falha ao aplicar taxas evento {} servidor {}: {}", ev.id, server_id, e);
                // Rollback: restaura backup
                let _ = swapper::restore_ini_files(pool, ev.id, *server_id).await;
            }
        }

        // 11.6 — Reinicia o servidor com as novas taxas
        start_server_by_id(pool, *server_id).await;
    }

    // Marca evento como active
    sqlx::query("UPDATE seasonal_events SET status = 'active' WHERE id = ?")
        .bind(ev.id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    // 11.7 — Inicia broadcasts periódicos durante o evento
    let pool_clone = pool.clone();
    let event_id = ev.id;
    let event_name = ev.name.clone();
    let interval = ev.broadcast_interval_seconds;
    let handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval as u64)).await;
            broadcast_to_event_servers(
                &pool_clone,
                event_id,
                &format!("ServerChat Evento sazonal \"{}\" ativo! Aproveite as taxas especiais.", event_name),
            )
            .await;
        }
    });

    state.active_broadcasts.lock().await.insert(ev.id, handle);
    log::info!("Evento {} ativado com sucesso.", ev.id);
    Ok(())
}

// ─────────────────────────────────────────────
// Encerrar evento (11.1, 11.6)
// ─────────────────────────────────────────────

async fn deactivate_event(
    pool: &DbPool,
    state: &SchedulerState,
    ev: &SeasonalEvent,
) -> Result<(), String> {
    log::info!("Encerrando evento sazonal: {} ({})", ev.name, ev.id);

    // Cancela broadcast periódico
    if let Some(handle) = state.active_broadcasts.lock().await.remove(&ev.id) {
        handle.abort();
    }

    let server_ids = get_event_server_ids(pool, ev.id).await?;

    for server_id in &server_ids {
        let (install_path, rcon_port, rcon_password) =
            get_server_rcon_info(pool, *server_id).await?;

        // 11.7 — Broadcast de aviso de encerramento
        broadcast_rcon(
            &install_path,
            rcon_port,
            &rcon_password,
            &format!("ServerChat Evento \"{}\" encerrou. Voltando às taxas normais.", ev.name),
        )
        .await;

        // 11.6 — SaveWorld + parar
        let _ = rcon_exec("127.0.0.1", rcon_port, &rcon_password, "SaveWorld").await;
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        stop_server_by_id(pool, *server_id).await;

        // Restaura INIs originais
        match swapper::restore_ini_files(pool, ev.id, *server_id).await {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "Falha ao restaurar INIs evento {} servidor {}: {}",
                    ev.id, server_id, e
                );
            }
        }

        // 11.6 — Reinicia o servidor com as taxas restauradas
        start_server_by_id(pool, *server_id).await;
    }

    // Marca como finished
    sqlx::query("UPDATE seasonal_events SET status = 'finished' WHERE id = ?")
        .bind(ev.id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    log::info!("Evento {} encerrado com sucesso.", ev.id);
    Ok(())
}

// ─────────────────────────────────────────────
// Helpers de banco
// ─────────────────────────────────────────────

async fn get_event_server_ids(pool: &DbPool, event_id: u32) -> Result<Vec<u32>, String> {
    let rows: Vec<(u32,)> =
        sqlx::query_as("SELECT server_id FROM seasonal_event_servers WHERE event_id = ?")
            .bind(event_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

async fn get_event_rates(pool: &DbPool, event_id: u32) -> Result<Option<EventRate>, String> {
    let rate: Option<EventRate> = sqlx::query_as(
        "SELECT id, event_id, xp_multiplier, harvest_multiplier, taming_multiplier,
                breeding_multiplier, quality_multiplier
         FROM seasonal_event_rates WHERE event_id = ?",
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rate)
}

/// Retorna (install_path, rcon_port, rcon_password) do servidor.
async fn get_server_rcon_info(
    pool: &DbPool,
    server_id: u32,
) -> Result<(String, u16, String), String> {
    let row: (String, i64, String) = sqlx::query_as(
        "SELECT install_path, rcon_port, admin_password FROM servers WHERE id = ?",
    )
    .bind(server_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("servidor {}: {}", server_id, e))?;

    Ok((row.0, row.1 as u16, row.2))
}

// ─────────────────────────────────────────────
// Helpers de processo (wrappers sem PidMap — via DB)
// ─────────────────────────────────────────────

/// Para o servidor pelo ID: envia SIGTERM via campo pid no banco.
/// Não usa PidMap pois o scheduler roda em contexto separado.
async fn stop_server_by_id(pool: &DbPool, server_id: u32) {
    let pid: Option<(Option<u32>,)> =
        sqlx::query_as("SELECT pid FROM servers WHERE id = ?")
            .bind(server_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if let Some((Some(pid),)) = pid {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .output();
        }
    }

    // Atualiza status no banco
    let _ = sqlx::query("UPDATE servers SET status = 'stopped', pid = NULL WHERE id = ?")
        .bind(server_id)
        .execute(pool)
        .await;

    // Aguarda o processo encerrar
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
}

/// Inicia o servidor pelo ID usando o script RunServer.cmd.
async fn start_server_by_id(pool: &DbPool, server_id: u32) {
    let info: Option<(String,)> =
        sqlx::query_as("SELECT install_path FROM servers WHERE id = ?")
            .bind(server_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if let Some((install_path,)) = info {
        let script = std::path::PathBuf::from(&install_path).join("RunServer.cmd");
        if script.exists() {
            match std::process::Command::new("cmd")
                .args(["/C", script.to_str().unwrap_or("")])
                .spawn()
            {
                Ok(child) => {
                    let pid = child.id();
                    let _ = sqlx::query(
                        "UPDATE servers SET status = 'starting', pid = ? WHERE id = ?",
                    )
                    .bind(pid)
                    .bind(server_id)
                    .execute(pool)
                    .await;
                }
                Err(e) => {
                    log::error!("Falha ao iniciar servidor {}: {}", server_id, e);
                }
            }
        }
    }
}

// ─────────────────────────────────────────────
// Helpers de RCON / Broadcast
// ─────────────────────────────────────────────

async fn broadcast_rcon(
    _install_path: &str,
    rcon_port: u16,
    rcon_password: &str,
    command: &str,
) {
    match rcon_exec("127.0.0.1", rcon_port, rcon_password, command).await {
        Ok(_) => {}
        Err(e) => log::warn!("Broadcast RCON porta {}: {}", rcon_port, e),
    }
}

async fn broadcast_to_event_servers(pool: &DbPool, event_id: u32, command: &str) {
    let server_ids = match get_event_server_ids(pool, event_id).await {
        Ok(ids) => ids,
        Err(e) => {
            log::warn!("broadcast_to_event_servers: {}", e);
            return;
        }
    };

    for sid in server_ids {
        if let Ok((ip, rcon_port, rcon_pass)) = get_server_rcon_info(pool, sid).await {
            let host = if ip.is_empty() { "127.0.0.1".to_string() } else { ip };
            broadcast_rcon(&host, rcon_port, &rcon_pass, command).await;
        }
    }
}

// ─────────────────────────────────────────────
// Utilitários de tempo
// ─────────────────────────────────────────────

fn chrono_now_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    unix_to_mysql_datetime(secs as i64)
}

/// Retorna "agora + offset_secs" como string MySQL DATETIME.
fn chrono_offset_str(offset_secs: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
        + offset_secs;
    unix_to_mysql_datetime(secs)
}

fn unix_to_mysql_datetime(unix: i64) -> String {
    // Conversão manual UTC: YYYY-MM-DD HH:MM:SS
    let secs = unix.max(0) as u64;
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Algoritmo civil de Howard Hinnant
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, m, s)
}
