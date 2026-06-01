use crate::models::task::{ScheduledTask, TaskType};
use chrono::{DateTime, Utc};
use cron::Schedule;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("Expressão cron inválida '{0}': {1}")]
    InvalidCron(String, String),
    #[error("Tarefa não encontrada: {0}")]
    TaskNotFound(String),
}

/// Calcula a próxima data de execução de uma expressão cron.
pub fn next_run(cron_expr: &str) -> Result<Option<DateTime<Utc>>, SchedulerError> {
    let schedule = Schedule::from_str(cron_expr)
        .map_err(|e| SchedulerError::InvalidCron(cron_expr.to_string(), e.to_string()))?;

    Ok(schedule.upcoming(Utc).next())
}

/// Verifica se uma expressão cron é válida.
pub fn validate_cron(cron_expr: &str) -> bool {
    Schedule::from_str(cron_expr).is_ok()
}

/// Mapa de tarefas ativas: task_id → handle de shutdown.
pub type TaskHandleMap = Arc<Mutex<HashMap<String, tokio::sync::broadcast::Sender<()>>>>;

/// Cria um novo mapa de handles de tarefas.
pub fn new_task_handle_map() -> TaskHandleMap {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Agenda uma tarefa para execução automática conforme a expressão cron.
/// `on_execute`: callback chamado quando a tarefa deve ser executada.
pub async fn schedule_task<F>(
    task: ScheduledTask,
    handle_map: &TaskHandleMap,
    on_execute: F,
) -> Result<(), SchedulerError>
where
    F: Fn(String, TaskType) + Send + Sync + 'static,
{
    // Valida o cron antes de agendar
    next_run(&task.cron_expression)?;

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);
    let task_id = task.id.to_string();
    let cron_expr = task.cron_expression.clone();
    let task_type = task.task_type_enum();

    handle_map
        .lock()
        .await
        .insert(task_id.clone(), shutdown_tx);

    tokio::spawn(async move {
        loop {
            let schedule = match Schedule::from_str(&cron_expr) {
                Ok(s) => s,
                Err(_) => break,
            };

            let next = match schedule.upcoming(Utc).next() {
                Some(t) => t,
                None => break,
            };

            let now = Utc::now();
            let delay = next.signed_duration_since(now);
            let sleep_dur = if delay.num_milliseconds() > 0 {
                std::time::Duration::from_millis(delay.num_milliseconds() as u64)
            } else {
                std::time::Duration::from_secs(1)
            };

            tokio::select! {
                _ = shutdown_rx.recv() => {
                    log::info!("Tarefa {} encerrada.", task_id);
                    break;
                }
                _ = tokio::time::sleep(sleep_dur) => {
                    on_execute(task_id.clone(), task_type.clone());
                }
            }
        }
    });

    Ok(())
}

/// Para a execução de uma tarefa agendada.
pub async fn stop_task(task_id: &str, handle_map: &TaskHandleMap) -> Result<(), SchedulerError> {
    let mut map = handle_map.lock().await;
    match map.remove(task_id) {
        Some(tx) => {
            let _ = tx.send(());
            Ok(())
        }
        None => Err(SchedulerError::TaskNotFound(task_id.to_string())),
    }
}



