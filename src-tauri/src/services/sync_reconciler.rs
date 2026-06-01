//! Listagem recursiva de arquivos e cálculo de diferenças para reconciliação.

use std::collections::HashMap;
use std::path::Path;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

use crate::services::sync_protocol::FileEntry;

/// Entrada de arquivo local (caminho relativo + metadados para comparação).
#[derive(Debug, Clone)]
pub struct LocalFileEntry {
    /// Caminho relativo com barras `/` (plataforma-neutral).
    pub path: String,
    pub size: u64,
    pub mtime: i64,
}

/// Lista recursivamente todos os arquivos em `root`.
/// Caminhos retornados são relativos a `root` com separador `/`.
pub fn list_files(root: &Path) -> Result<Vec<LocalFileEntry>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let rel = entry.path().strip_prefix(root).map_err(|e| e.to_string())?;
        let path = rel.to_string_lossy().replace('\\', "/");
        entries.push(LocalFileEntry { path, size: meta.len(), mtime });
    }
    Ok(entries)
}

/// Resultado da comparação entre listas locais e remotas.
pub struct Diff {
    /// Arquivos locais que devem ser enviados ao peer (mais novos ou ausentes no peer).
    pub to_send: Vec<LocalFileEntry>,
    /// Caminhos de arquivos que devem ser solicitados ao peer (mais novos ou ausentes localmente).
    pub to_request: Vec<String>,
}

/// Calcula o diff entre a lista local e a lista do peer.
pub fn compute_diff(local: &[LocalFileEntry], remote: &[FileEntry]) -> Diff {
    let local_map: HashMap<&str, &LocalFileEntry> =
        local.iter().map(|e| (e.path.as_str(), e)).collect();
    let remote_map: HashMap<&str, &FileEntry> =
        remote.iter().map(|e| (e.path.as_str(), e)).collect();

    let mut to_send = Vec::new();
    let mut to_request = Vec::new();

    for (path, le) in &local_map {
        match remote_map.get(path) {
            None => to_send.push((*le).clone()),
            Some(re) => {
                if le.mtime > re.mtime {
                    to_send.push((*le).clone());
                } else if re.mtime > le.mtime {
                    to_request.push(path.to_string());
                }
            }
        }
    }

    for path in remote_map.keys() {
        if !local_map.contains_key(path) {
            to_request.push(path.to_string());
        }
    }

    Diff { to_send, to_request }
}
