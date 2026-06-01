use serde::Serialize;
use sysinfo::{Pid, System};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemAnalyzerError {
    #[error("Processo não encontrado (PID {0})")]
    ProcessNotFound(u32),
}

/// Métricas globais do sistema.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetrics {
    /// Percentual total de uso de CPU (0.0–100.0).
    pub cpu_percent: f32,
    /// RAM total em bytes.
    pub total_memory_bytes: u64,
    /// RAM em uso em bytes.
    pub used_memory_bytes: u64,
    /// Percentual de RAM em uso (0.0–100.0).
    pub memory_percent: f32,
}

/// Métricas de um processo específico (servidor ARK).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetrics {
    pub pid: u32,
    /// Percentual de CPU usado pelo processo (0.0–100.0).
    pub cpu_percent: f32,
    /// RAM usada pelo processo em bytes.
    pub memory_bytes: u64,
    /// Se o processo está em execução.
    pub running: bool,
}

/// Coleta métricas globais do sistema.
pub fn get_system_metrics() -> SystemMetrics {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_percent = sys.global_cpu_usage();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_percent = if total_memory > 0 {
        (used_memory as f32 / total_memory as f32) * 100.0
    } else {
        0.0
    };

    SystemMetrics {
        cpu_percent,
        total_memory_bytes: total_memory,
        used_memory_bytes: used_memory,
        memory_percent,
    }
}

/// Coleta métricas de um processo específico pelo PID.
pub fn get_process_metrics(pid: u32) -> Result<ProcessMetrics, SystemAnalyzerError> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let process = sys
        .process(Pid::from_u32(pid))
        .ok_or(SystemAnalyzerError::ProcessNotFound(pid))?;

    Ok(ProcessMetrics {
        pid,
        cpu_percent: process.cpu_usage(),
        memory_bytes: process.memory(),
        running: true,
    })
}

/// Busca o PID de um processo pelo nome do executável.
/// Retorna o primeiro PID encontrado, ou None.
pub fn find_process_by_name(name: &str) -> Option<u32> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    sys.processes()
        .values()
        .find(|p| {
            p.name()
                .to_string_lossy()
                .to_lowercase()
                .contains(&name.to_lowercase())
        })
        .map(|p| p.pid().as_u32())
}

