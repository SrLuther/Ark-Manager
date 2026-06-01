//! Protocolo de sincronização sobre WebSocket entre agentes ARK Manager.
//!
//! Fluxo de sincronização bidirecional (A = iniciador, B = receptor):
//!   1. A → B: `ReconcileRequest { folder_id }`
//!   2. B → A: `FileList { folder_id, files }`
//!   3. A calcula diff, envia arquivos que B precisa: `TransferStart / Chunk / Done`
//!   4. B confirma cada transferência: `TransferAck`
//!   5. A solicita arquivos que precisa de B: `RequestFiles { paths }`
//!   6. B envia os arquivos solicitados
//!   7. A confirma: `TransferAck`
//!   8. B → A: `SyncComplete { folder_id }`

use serde::{Deserialize, Serialize};

/// Entrada de arquivo para reconciliação (caminho relativo + metadados).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    /// Timestamp Unix em segundos (mtime).
    pub mtime: i64,
}

/// Mensagens trocadas via WebSocket durante a sincronização.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncMessage {
    // --- Reconciliação ---
    /// Inicia o processo de reconciliação para uma pasta.
    ReconcileRequest { folder_id: u32 },
    /// Resposta com a lista de arquivos locais do receptor.
    FileList { folder_id: u32, files: Vec<FileEntry> },
    /// Solicita arquivos específicos do outro lado.
    RequestFiles { folder_id: u32, paths: Vec<String> },

    // --- Transferência ---
    /// Inicia a transferência de um arquivo.
    TransferStart {
        transfer_id: String,
        folder_id: u32,
        path: String,
        size: u64,
        checksum: String,
    },
    /// Chunk de dados em base64.
    TransferChunk {
        transfer_id: String,
        offset: u64,
        data: String,
    },
    /// Sinaliza fim da transferência.
    TransferDone { transfer_id: String },
    /// Confirmação/rejeição da transferência.
    TransferAck {
        transfer_id: String,
        ok: bool,
        error: Option<String>,
    },

    // --- Controle ---
    /// Deleção de arquivo remoto.
    DeleteFile { folder_id: u32, path: String },
    /// Sincronização concluída.
    SyncComplete { folder_id: u32 },
    /// Erro genérico.
    Error { message: String },
    Ping,
    Pong,
}

impl SyncMessage {
    /// Serializa para JSON (nunca falha — retorna mensagem de erro genérica em caso extremo).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| r#"{"type":"error","message":"serialization error"}"#.to_string())
    }

    /// Desserializa a partir de JSON. Retorna `None` em caso de erro.
    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}
