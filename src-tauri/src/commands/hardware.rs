use crate::services::system_analyzer::{self, ProcessMetrics, SystemMetrics};

/// Retorna métricas globais do sistema (CPU, RAM).
#[tauri::command]
pub fn get_system_metrics() -> SystemMetrics {
    system_analyzer::get_system_metrics()
}

/// Retorna métricas de um processo específico pelo PID.
#[tauri::command]
pub fn get_process_metrics(pid: u32) -> Result<ProcessMetrics, String> {
    system_analyzer::get_process_metrics(pid).map_err(|e| e.to_string())
}

/// Localiza o PID do processo ShooterGameServer.exe (ou outro nome informado).
#[tauri::command]
pub fn find_server_process(process_name: Option<String>) -> Option<u32> {
    let name = process_name
        .as_deref()
        .unwrap_or("ShooterGameServer.exe");
    system_analyzer::find_process_by_name(name)
}

