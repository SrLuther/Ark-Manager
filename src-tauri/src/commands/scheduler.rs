use crate::models::task::{CreateTaskRequest, ScheduledTask, UpdateTaskRequest};
use crate::services::scheduler;
use crate::AppState;
use tauri::State;

/// Lista as tarefas agendadas de um servidor.
#[tauri::command]
pub async fn list_tasks(
    server_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<ScheduledTask>, String> {
    sqlx::query_as::<_, ScheduledTask>(
        "SELECT * FROM am_scheduled_tasks WHERE server_id = ? ORDER BY task_name ASC",
    )
    .bind(server_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Cria uma nova tarefa agendada.
#[tauri::command]
pub async fn create_task(
    req: CreateTaskRequest,
    state: State<'_, AppState>,
) -> Result<ScheduledTask, String> {
    // Valida a expressão cron antes de salvar
    if !scheduler::validate_cron(&req.cron_expression) {
        return Err(format!("Expressão cron inválida: '{}'", req.cron_expression));
    }

    sqlx::query(
        r#"INSERT INTO am_scheduled_tasks
        (server_id, task_name, task_type, cron_expression, command, message,
         pre_warning_minutes, enabled, run_count, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
    )
    .bind(req.server_id)
    .bind(&req.task_name)
    .bind(&req.task_type.to_string())
    .bind(&req.cron_expression)
    .bind(&req.command)
    .bind(&req.message)
    .bind(req.pre_warning_minutes.unwrap_or(0))
    .bind(req.enabled.unwrap_or(true))
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, ScheduledTask>(
        "SELECT * FROM am_scheduled_tasks ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())
}

/// Atualiza uma tarefa agendada.
#[tauri::command]
pub async fn update_task(
    id: u32,
    req: UpdateTaskRequest,
    state: State<'_, AppState>,
) -> Result<ScheduledTask, String> {
    if let Some(ref expr) = req.cron_expression {
        if !scheduler::validate_cron(expr) {
            return Err(format!("Expressão cron inválida: '{}'", expr));
        }
    }

    sqlx::query(
        r#"UPDATE am_scheduled_tasks SET
        task_name = COALESCE(?, task_name),
        task_type = COALESCE(?, task_type),
        cron_expression = COALESCE(?, cron_expression),
        command = COALESCE(?, command),
        message = COALESCE(?, message),
        pre_warning_minutes = COALESCE(?, pre_warning_minutes),
        enabled = COALESCE(?, enabled),
        updated_at = CURRENT_TIMESTAMP
        WHERE id = ?"#,
    )
    .bind(&req.task_name)
    .bind(req.task_type.as_ref().map(|t| t.to_string()))
    .bind(&req.cron_expression)
    .bind(&req.command)
    .bind(&req.message)
    .bind(req.pre_warning_minutes)
    .bind(req.enabled)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query_as::<_, ScheduledTask>("SELECT * FROM am_scheduled_tasks WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())
}

/// Remove uma tarefa agendada.
#[tauri::command]
pub async fn delete_task(id: u32, state: State<'_, AppState>) -> Result<(), String> {
    sqlx::query("DELETE FROM am_scheduled_tasks WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Valida uma expressão cron e retorna a próxima data de execução.
#[tauri::command]
pub fn validate_cron_expression(cron_expr: String) -> Result<Option<String>, String> {
    let next = scheduler::next_run(&cron_expr).map_err(|e| e.to_string())?;
    Ok(next.map(|dt| dt.to_rfc3339()))
}

