use std::io;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Tipos de pacotes do protocolo Source RCON.
const SERVERDATA_AUTH: i32 = 3;
const SERVERDATA_AUTH_RESPONSE: i32 = 2;
const SERVERDATA_EXECCOMMAND: i32 = 2;
#[allow(dead_code)]
const SERVERDATA_RESPONSE_VALUE: i32 = 0;

#[derive(Debug, Error)]
pub enum RconError {
    #[error("Falha ao conectar ao RCON em {0}:{1}: {2}")]
    Connection(String, u16, String),
    #[error("Autenticação RCON falhou — senha incorreta")]
    AuthFailed,
    #[error("Erro de I/O: {0}")]
    Io(#[from] io::Error),
    #[error("Resposta RCON inválida: {0}")]
    InvalidResponse(String),
}

/// Conexão RCON ativa com um servidor ARK.
pub struct RconConnection {
    stream: TcpStream,
    next_id: i32,
}

impl RconConnection {
    /// Conecta e autentica via RCON.
    pub async fn connect(host: &str, port: u16, password: &str) -> Result<Self, RconError> {
        let stream = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .map_err(|e| RconError::Connection(host.to_string(), port, e.to_string()))?;

        let mut conn = RconConnection {
            stream,
            next_id: 1,
        };

        // Envia pacote de autenticação
        conn.send_packet(SERVERDATA_AUTH, password).await?;

        // Lê resposta de autenticação
        let resp = conn.read_packet().await?;
        if resp.packet_type == SERVERDATA_AUTH_RESPONSE && resp.id == -1 {
            return Err(RconError::AuthFailed);
        }

        Ok(conn)
    }

    /// Envia um comando RCON e retorna a resposta em texto.
    pub async fn send_command(&mut self, command: &str) -> Result<String, RconError> {
        self.send_packet(SERVERDATA_EXECCOMMAND, command).await?;
        let resp = self.read_packet().await?;
        Ok(resp.body)
    }

    /// Envia um pacote RCON.
    async fn send_packet(&mut self, packet_type: i32, body: &str) -> Result<(), RconError> {
        let id = self.next_id;
        self.next_id += 1;

        let body_bytes = body.as_bytes();
        // size = id (4) + type (4) + body + null term + empty string null term
        let size = 4 + 4 + body_bytes.len() + 1 + 1;

        let mut packet = Vec::with_capacity(4 + size);
        packet.extend_from_slice(&(size as i32).to_le_bytes());
        packet.extend_from_slice(&id.to_le_bytes());
        packet.extend_from_slice(&packet_type.to_le_bytes());
        packet.extend_from_slice(body_bytes);
        packet.push(0); // null terminator do body
        packet.push(0); // null terminator extra

        self.stream.write_all(&packet).await?;
        Ok(())
    }

    /// Lê um pacote RCON da stream.
    async fn read_packet(&mut self) -> Result<RconPacket, RconError> {
        // Lê o tamanho (4 bytes LE)
        let mut size_buf = [0u8; 4];
        self.stream.read_exact(&mut size_buf).await?;
        let size = i32::from_le_bytes(size_buf) as usize;

        if size < 10 || size > 4096 {
            return Err(RconError::InvalidResponse(format!(
                "Tamanho de pacote inválido: {}",
                size
            )));
        }

        let mut data = vec![0u8; size];
        self.stream.read_exact(&mut data).await?;

        let id = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let packet_type = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        // Body é o que vem após os 8 bytes iniciais, até o null terminator
        let body_end = data[8..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(data.len() - 8);
        let body = String::from_utf8_lossy(&data[8..8 + body_end]).to_string();

        Ok(RconPacket {
            id,
            packet_type,
            body,
        })
    }
}

struct RconPacket {
    id: i32,
    packet_type: i32,
    body: String,
}

/// Executa um único comando RCON sem manter conexão persistente.
/// Útil para comandos pontuais como broadcasts do scheduler.
pub async fn execute_command(
    host: &str,
    port: u16,
    password: &str,
    command: &str,
) -> Result<String, RconError> {
    let mut conn = RconConnection::connect(host, port, password).await?;
    conn.send_command(command).await
}

