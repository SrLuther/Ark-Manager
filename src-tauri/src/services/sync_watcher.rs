//! Watcher de pastas usando a crate `notify`.
//!
//! Detecta criações, modificações e deleções de arquivos em tempo real.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum WatchEventKind {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone)]
pub struct WatchEvent {
    pub path: PathBuf,
    pub kind: WatchEventKind,
}

/// Watcher recursivo de uma pasta.
/// O `_watcher` deve permanecer em memória para manter o watch ativo.
pub struct FolderWatcher {
    _watcher: RecommendedWatcher,
    pub events: Arc<Mutex<Vec<WatchEvent>>>,
}

impl FolderWatcher {
    /// Inicia o watch recursivo de `path`. Retorna erro se a pasta não existir.
    pub fn watch(path: &Path) -> Result<Self, String> {
        let events: Arc<Mutex<Vec<WatchEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            let Ok(event) = res else { return };
            let kind = match event.kind {
                EventKind::Create(_) => WatchEventKind::Created,
                EventKind::Modify(_) => WatchEventKind::Modified,
                EventKind::Remove(_) => WatchEventKind::Deleted,
                _ => return,
            };
            let mut guard = events_clone.lock().unwrap();
            for path in event.paths {
                guard.push(WatchEvent { path, kind: kind.clone() });
            }
        })
        .map_err(|e| format!("Falha ao criar watcher: {}", e))?;

        watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| format!("Falha ao observar {:?}: {}", path, e))?;

        Ok(Self { _watcher: watcher, events })
    }

    /// Drena e retorna todos os eventos acumulados desde a última chamada.
    pub fn drain_events(&self) -> Vec<WatchEvent> {
        let mut guard = self.events.lock().unwrap();
        std::mem::take(&mut *guard)
    }
}
