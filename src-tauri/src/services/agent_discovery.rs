//! Descoberta de agentes ARK Manager na rede local via UDP broadcast.
//!
//! - Porta de descoberta: 45679
//! - Protocolo: JSON broadcast a cada 10 segundos
//! - Formato: `{"name":"...","port":45678,"version":"1.0.0"}`

use crate::models::agent::{AgentAnnouncement, DiscoveredAgent};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::UdpSocket;

pub const DISCOVERY_PORT: u16 = 45679;
const BROADCAST_INTERVAL: Duration = Duration::from_secs(10);

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Inicia as tarefas de descoberta: listener UDP e broadcaster periódico.
pub async fn start_discovery(
    agent_name: String,
    server_port: u16,
    discovered: Arc<Mutex<Vec<DiscoveredAgent>>>,
) {
    let name_for_listen = agent_name.clone();
    let discovered_for_listen = discovered.clone();

    tokio::spawn(async move {
        listen_for_agents(name_for_listen, server_port, discovered_for_listen).await;
    });

    tokio::spawn(async move {
        broadcast_presence(agent_name, server_port).await;
    });
}

// ---------------------------------------------------------------------------
// Listener UDP
// ---------------------------------------------------------------------------

async fn listen_for_agents(
    my_name: String,
    my_port: u16,
    discovered: Arc<Mutex<Vec<DiscoveredAgent>>>,
) {
    let sock = match UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await {
        Ok(s) => s,
        Err(e) => {
            // Porta já em uso (outro ARK Manager na mesma máquina) — não fatal
            log::warn!("Listener UDP de descoberta: porta {} em uso: {}", DISCOVERY_PORT, e);
            return;
        }
    };

    if let Err(e) = sock.set_broadcast(true) {
        log::warn!("Falha ao habilitar broadcast no socket UDP: {}", e);
    }

    log::info!("Listener UDP de descoberta ativo na porta {}", DISCOVERY_PORT);

    let mut buf = [0u8; 2048];
    loop {
        match sock.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                if let Ok(ann) = serde_json::from_slice::<AgentAnnouncement>(&buf[..len]) {
                    // Ignora o próprio anúncio
                    if ann.name == my_name && ann.port == my_port as u32 {
                        continue;
                    }

                    let peer_ip = addr.ip().to_string();
                    let mut guard = discovered.lock().unwrap();

                    if let Some(existing) = guard
                        .iter_mut()
                        .find(|a| a.address == peer_ip && a.port == ann.port)
                    {
                        existing.name = ann.name;
                    } else {
                        log::info!("Agente descoberto: {} em {}:{}", ann.name, peer_ip, ann.port);
                        guard.push(DiscoveredAgent {
                            name: ann.name,
                            address: peer_ip,
                            port: ann.port,
                        });
                    }
                }
            }
            Err(e) => {
                log::warn!("Erro no listener UDP de descoberta: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Broadcaster UDP
// ---------------------------------------------------------------------------

async fn broadcast_presence(agent_name: String, server_port: u16) {
    let sock = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Falha ao criar socket UDP para broadcast: {}", e);
            return;
        }
    };

    if let Err(e) = sock.set_broadcast(true) {
        log::warn!("Falha ao habilitar broadcast: {}", e);
        return;
    }

    let announcement = AgentAnnouncement {
        name: agent_name,
        port: server_port as u32,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let payload = match serde_json::to_vec(&announcement) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Falha ao serializar anúncio UDP: {}", e);
            return;
        }
    };

    let broadcast_addr = SocketAddrV4::new(Ipv4Addr::BROADCAST, DISCOVERY_PORT);

    loop {
        if let Err(e) = sock.send_to(&payload, broadcast_addr).await {
            log::warn!("Falha ao enviar broadcast UDP: {}", e);
        }
        tokio::time::sleep(BROADCAST_INTERVAL).await;
    }
}
