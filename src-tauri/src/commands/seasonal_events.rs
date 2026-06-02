use tauri::State;

use crate::AppState;
use crate::db::DbPool;
use crate::models::seasonal_event::{
    CreateEventRequest, UpdateEventRequest,
    SeasonalEvent, SeasonalEventFull, EventRate,
};
use crate::services::event_scheduler::{SchedulerStateArc, activate_event_manual, deactivate_event_manual};

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// list_seasonal_events
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tauri::command]
pub async fn list_seasonal_events(
    state: State<'_, AppState>,
) -> Result<Vec<SeasonalEventFull>, String> {
    let pool = &state.db;
    let events: Vec<SeasonalEvent> = sqlx::query_as(
        "SELECT id, name, description, start_at, end_at, status,
                broadcast_interval_seconds, created_at, updated_at
         FROM am_seasonal_events
         ORDER BY start_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(events.len());
    for ev in events {
        let rates = get_rates(pool, ev.id).await?;
        let server_ids = get_server_ids(pool, ev.id).await?;
        result.push(SeasonalEventFull { event: ev, rates, server_ids });
    }
    Ok(result)
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// get_seasonal_event
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tauri::command]
pub async fn get_seasonal_event(
    id: u32,
    state: State<'_, AppState>,
) -> Result<SeasonalEventFull, String> {
    let pool = &state.db;
    let ev: SeasonalEvent = sqlx::query_as(
        "SELECT id, name, description, start_at, end_at, status,
                broadcast_interval_seconds, created_at, updated_at
         FROM am_seasonal_events WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let rates = get_rates(pool, ev.id).await?;
    let server_ids = get_server_ids(pool, ev.id).await?;
    Ok(SeasonalEventFull { event: ev, rates, server_ids })
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// create_seasonal_event
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tauri::command]
pub async fn create_seasonal_event(
    req: CreateEventRequest,
    state: State<'_, AppState>,
) -> Result<SeasonalEventFull, String> {
    validate_event_dates(&req.start_at, &req.end_at)?;

    let pool = &state.db;
    let interval = req.broadcast_interval_seconds.unwrap_or(300);

    let res = sqlx::query(
        "INSERT INTO am_seasonal_events (name, description, start_at, end_at, broadcast_interval_seconds)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.start_at)
    .bind(&req.end_at)
    .bind(interval)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let event_id = res.last_insert_rowid() as u32;

    // Insere rates
    sqlx::query(
        "INSERT INTO am_seasonal_event_rates
         (event_id, xp_multiplier, harvest_multiplier, taming_multiplier, breeding_multiplier, quality_multiplier)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(event_id)
    .bind(req.rates.xp_multiplier)
    .bind(req.rates.harvest_multiplier)
    .bind(req.rates.taming_multiplier)
    .bind(req.rates.breeding_multiplier)
    .bind(req.rates.quality_multiplier)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Insere servidores
    for sid in &req.server_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO am_seasonal_event_servers (event_id, server_id) VALUES (?, ?)",
        )
        .bind(event_id)
        .bind(sid)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    get_seasonal_event(event_id, state).await
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// update_seasonal_event
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tauri::command]
pub async fn update_seasonal_event(
    id: u32,
    req: UpdateEventRequest,
    state: State<'_, AppState>,
) -> Result<SeasonalEventFull, String> {
    let pool = &state.db;

    // Valida datas se fornecidas
    if let (Some(start), Some(end)) = (&req.start_at, &req.end_at) {
        validate_event_dates(start, end)?;
    }

    // Atualiza campos opcionais
    if let Some(name) = &req.name {
        sqlx::query("UPDATE am_seasonal_events SET name = ? WHERE id = ?")
            .bind(name).bind(id).execute(pool).await.map_err(|e| e.to_string())?;
    }
    if let Some(desc) = &req.description {
        sqlx::query("UPDATE am_seasonal_events SET description = ? WHERE id = ?")
            .bind(desc).bind(id).execute(pool).await.map_err(|e| e.to_string())?;
    }
    if let Some(start) = &req.start_at {
        sqlx::query("UPDATE am_seasonal_events SET start_at = ? WHERE id = ?")
            .bind(start).bind(id).execute(pool).await.map_err(|e| e.to_string())?;
    }
    if let Some(end) = &req.end_at {
        sqlx::query("UPDATE am_seasonal_events SET end_at = ? WHERE id = ?")
            .bind(end).bind(id).execute(pool).await.map_err(|e| e.to_string())?;
    }
    if let Some(interval) = req.broadcast_interval_seconds {
        sqlx::query("UPDATE am_seasonal_events SET broadcast_interval_seconds = ? WHERE id = ?")
            .bind(interval).bind(id).execute(pool).await.map_err(|e| e.to_string())?;
    }

    // Atualiza rates
    if let Some(rates) = &req.rates {
        sqlx::query(
            "UPDATE am_seasonal_event_rates SET
             xp_multiplier = ?, harvest_multiplier = ?, taming_multiplier = ?,
             breeding_multiplier = ?, quality_multiplier = ?
             WHERE event_id = ?",
        )
        .bind(rates.xp_multiplier)
        .bind(rates.harvest_multiplier)
        .bind(rates.taming_multiplier)
        .bind(rates.breeding_multiplier)
        .bind(rates.quality_multiplier)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Atualiza servidores
    if let Some(server_ids) = &req.server_ids {
        sqlx::query("DELETE FROM am_seasonal_event_servers WHERE event_id = ?")
            .bind(id).execute(pool).await.map_err(|e| e.to_string())?;
        for sid in server_ids {
            sqlx::query(
"INSERT OR IGNORE INTO am_seasonal_event_servers (event_id, server_id) VALUES (?, ?)",
            )
            .bind(id).bind(sid).execute(pool).await.map_err(|e| e.to_string())?;
        }
    }

    get_seasonal_event(id, state).await
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// cancel_seasonal_event
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tauri::command]
pub async fn cancel_seasonal_event(
    id: u32,
    state: State<'_, AppState>,
    scheduler: State<'_, SchedulerStateArc>,
) -> Result<(), String> {
    let pool = &state.db;

    // Se estiver ativo, encerra e restaura INIs
    let status: Option<(String,)> =
        sqlx::query_as("SELECT status FROM am_seasonal_events WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

    if matches!(status, Some((ref s,)) if s == "active") {
        deactivate_event_manual(pool, &scheduler, id).await?;
    }

    sqlx::query("UPDATE am_seasonal_events SET status = 'cancelled' WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// force_start_event / force_end_event
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tauri::command]
pub async fn force_start_event(
    id: u32,
    state: State<'_, AppState>,
    scheduler: State<'_, SchedulerStateArc>,
) -> Result<(), String> {
    let pool = &state.db;
    let ev: SeasonalEvent = sqlx::query_as(
        "SELECT id, name, description, start_at, end_at, status,
                broadcast_interval_seconds, created_at, updated_at
         FROM am_seasonal_events WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    activate_event_manual(pool, &scheduler, &ev).await
}

#[tauri::command]
pub async fn force_end_event(
    id: u32,
    state: State<'_, AppState>,
    scheduler: State<'_, SchedulerStateArc>,
) -> Result<(), String> {
    let pool = &state.db;
    deactivate_event_manual(pool, &scheduler, id).await
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// get_event_status
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[tauri::command]
pub async fn get_event_status(
    id: u32,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let row: (String,) = sqlx::query_as("SELECT status FROM am_seasonal_events WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.0)
}

// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
// Helpers
// â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

async fn get_rates(pool: &DbPool, event_id: u32) -> Result<Option<EventRate>, String> {
    sqlx::query_as(
        "SELECT id, event_id, xp_multiplier, harvest_multiplier, taming_multiplier,
                breeding_multiplier, quality_multiplier
         FROM am_seasonal_event_rates WHERE event_id = ?",
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())
}

async fn get_server_ids(pool: &DbPool, event_id: u32) -> Result<Vec<u32>, String> {
    let rows: Vec<(u32,)> =
        sqlx::query_as("SELECT server_id FROM am_seasonal_event_servers WHERE event_id = ?")
            .bind(event_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

fn validate_event_dates(start: &str, end: &str) -> Result<(), String> {
    if end <= start {
        return Err("A data de fim deve ser posterior Ã  data de inÃ­cio".to_string());
    }
    Ok(())
}
