//! Transferência de arquivos via mensagens do protocolo de sync.
//!
//! - `prepare_transfer` → gera a sequência TransferStart/Chunk/Done para envio.
//! - `FileReceiver` → acumula chunks recebidos e verifica checksum SHA-256 ao final.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::services::sync_protocol::SyncMessage;

const CHUNK_SIZE: usize = 65_536; // 64 KB

// ---------------------------------------------------------------------------
// Checksum
// ---------------------------------------------------------------------------

/// Calcula SHA-256 de um arquivo e retorna o hex-string.
pub fn compute_checksum(path: &Path) -> Result<String, String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Erro ao abrir {:?}: {}", path, e))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

// ---------------------------------------------------------------------------
// Envio
// ---------------------------------------------------------------------------

/// Gera a sequência completa de mensagens `SyncMessage` para transferir um arquivo.
/// Ordem: `TransferStart` → N × `TransferChunk` → `TransferDone`.
pub fn prepare_transfer(
    transfer_id: &str,
    folder_id: u32,
    rel_path: &str,
    abs_path: &Path,
) -> Result<Vec<SyncMessage>, String> {
    let meta = std::fs::metadata(abs_path)
        .map_err(|e| format!("Metadata de {:?}: {}", abs_path, e))?;
    let checksum = compute_checksum(abs_path)?;

    let mut msgs = vec![SyncMessage::TransferStart {
        transfer_id: transfer_id.to_string(),
        folder_id,
        path: rel_path.to_string(),
        size: meta.len(),
        checksum,
    }];

    let mut file = std::fs::File::open(abs_path)
        .map_err(|e| format!("Erro ao abrir arquivo: {}", e))?;
    let mut offset = 0u64;
    let mut buf = vec![0u8; CHUNK_SIZE];

    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        msgs.push(SyncMessage::TransferChunk {
            transfer_id: transfer_id.to_string(),
            offset,
            data: BASE64.encode(&buf[..n]),
        });
        offset += n as u64;
    }

    msgs.push(SyncMessage::TransferDone { transfer_id: transfer_id.to_string() });
    Ok(msgs)
}

// ---------------------------------------------------------------------------
// Recebimento
// ---------------------------------------------------------------------------

/// Receptor de arquivo: acumula chunks e verifica o checksum ao finalizar.
pub struct FileReceiver {
    pub dest_path: PathBuf,
    expected_checksum: String,
    file: std::fs::File,
}

impl FileReceiver {
    /// Cria o arquivo de destino e prepara o receptor.
    pub fn new(
        dest_dir: &Path,
        rel_path: &str,
        expected_checksum: String,
        _expected_size: u64,
    ) -> Result<Self, String> {
        let sep = std::path::MAIN_SEPARATOR_STR;
        let dest_path = dest_dir.join(rel_path.replace('/', sep));
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Erro ao criar diretórios: {}", e))?;
        }
        let file = std::fs::File::create(&dest_path)
            .map_err(|e| format!("Erro ao criar {:?}: {}", dest_path, e))?;
        Ok(Self { dest_path, expected_checksum, file })
    }

    /// Grava um chunk recebido (base64) no arquivo destino.
    pub fn write_chunk(&mut self, _offset: u64, data: &str) -> Result<(), String> {
        let bytes = BASE64
            .decode(data)
            .map_err(|e| format!("Erro ao decodificar chunk base64: {}", e))?;
        self.file.write_all(&bytes).map_err(|e| e.to_string())
    }

    /// Fecha o arquivo, verifica o checksum SHA-256 e retorna erro se divergir.
    pub fn finish(self) -> Result<(), String> {
        drop(self.file);
        let actual = compute_checksum(&self.dest_path)?;
        if actual != self.expected_checksum {
            std::fs::remove_file(&self.dest_path).ok();
            return Err(format!(
                "Checksum inválido para {:?}: esperado {}, recebido {}",
                self.dest_path, self.expected_checksum, actual
            ));
        }
        Ok(())
    }
}
