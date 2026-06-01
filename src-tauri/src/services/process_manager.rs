use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Script de lançamento não encontrado: {0}")]
    ScriptNotFound(String),
    #[error("Servidor já está em execução (PID {0})")]
    AlreadyRunning(u32),
    #[error("Servidor não está em execução")]
    NotRunning,
    #[error("Falha ao iniciar processo: {0}")]
    StartFailed(String),
    #[error("Falha ao encerrar processo: {0}")]
    StopFailed(String),
    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),
}

/// Mapa server_id → PID dos processos ativos.
pub type PidMap = Arc<Mutex<HashMap<String, u32>>>;

/// Cria um novo mapa de PIDs compartilhado.
pub fn new_pid_map() -> PidMap {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Inicia o servidor ARK usando o script RunServer.cmd.
/// Retorna o PID do processo criado.
pub async fn start_server(
    server_id: &str,
    script_path: &Path,
    pid_map: &PidMap,
) -> Result<u32, ProcessError> {
    if !script_path.exists() {
        return Err(ProcessError::ScriptNotFound(
            script_path.display().to_string(),
        ));
    }

    // Verifica se já está rodando
    {
        let map = pid_map.lock().await;
        if let Some(&pid) = map.get(server_id) {
            if is_pid_alive(pid) {
                return Err(ProcessError::AlreadyRunning(pid));
            }
        }
    }

    let child = tokio::process::Command::new("cmd")
        .args(["/c", &script_path.display().to_string()])
        .spawn()
        .map_err(|e| ProcessError::StartFailed(e.to_string()))?;

    let pid = child.id().ok_or_else(|| {
        ProcessError::StartFailed("Não foi possível obter PID do processo".to_string())
    })?;

    // Não esperamos o child encerrar — ele roda em background
    // O processo é gerenciado pelo PID
    std::mem::forget(child);

    pid_map.lock().await.insert(server_id.to_string(), pid);

    log::info!("Servidor {} iniciado com PID {}", server_id, pid);
    Ok(pid)
}

/// Para o servidor ARK pelo PID registrado.
pub async fn stop_server(server_id: &str, pid_map: &PidMap) -> Result<(), ProcessError> {
    let pid = {
        let map = pid_map.lock().await;
        *map.get(server_id).ok_or(ProcessError::NotRunning)?
    };

    kill_process(pid).map_err(|e| ProcessError::StopFailed(e.to_string()))?;

    pid_map.lock().await.remove(server_id);
    log::info!("Servidor {} (PID {}) encerrado.", server_id, pid);
    Ok(())
}

/// Reinicia o servidor: para + inicia.
pub async fn restart_server(
    server_id: &str,
    script_path: &Path,
    pid_map: &PidMap,
) -> Result<u32, ProcessError> {
    // Para se estiver rodando (ignora erro de "não está rodando")
    match stop_server(server_id, pid_map).await {
        Ok(_) | Err(ProcessError::NotRunning) => {}
        Err(e) => return Err(e),
    }

    // Pequeno delay para garantir que o processo encerrou
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    start_server(server_id, script_path, pid_map).await
}

/// Retorna o PID se o servidor estiver rodando, ou None.
pub async fn get_pid(server_id: &str, pid_map: &PidMap) -> Option<u32> {
    let map = pid_map.lock().await;
    let pid = *map.get(server_id)?;
    if is_pid_alive(pid) {
        Some(pid)
    } else {
        None
    }
}

/// Verifica se o servidor está em execução.
pub async fn is_running(server_id: &str, pid_map: &PidMap) -> bool {
    get_pid(server_id, pid_map).await.is_some()
}

/// Verifica se um PID está vivo no sistema operacional.
#[cfg(target_os = "windows")]
fn is_pid_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::System::Threading::GetExitCodeProcess;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return false;
        }
        let mut exit_code: u32 = 0;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);
        // STILL_ACTIVE = 259
        ok != 0 && exit_code == 259
    }
}

#[cfg(not(target_os = "windows"))]
fn is_pid_alive(_pid: u32) -> bool {
    false
}

/// Encerra um processo pelo PID.
#[cfg(target_os = "windows")]
fn kill_process(pid: u32) -> Result<(), String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return Err(format!("Não foi possível abrir processo PID {}", pid));
        }
        let result = TerminateProcess(handle, 1);
        CloseHandle(handle);
        if result == 0 {
            Err(format!("Falha ao encerrar processo PID {}", pid))
        } else {
            Ok(())
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn kill_process(_pid: u32) -> Result<(), String> {
    Err("Plataforma não suportada".to_string())
}

