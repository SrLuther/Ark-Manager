//! Integração com Discord via Incoming Webhooks.
//!
//! Envia notificações de eventos do servidor para um canal Discord configurado.

use serde::{Deserialize, Serialize};

/// Payload para Discord Incoming Webhook.
#[derive(Serialize)]
struct DiscordPayload<'a> {
    content: &'a str,
    username: &'static str,
}

/// Envia uma mensagem para um Discord Webhook.
///
/// `webhook_url` — URL completa do webhook (`https://discord.com/api/webhooks/...`).
/// `message` — texto plain da mensagem.
pub async fn send_webhook(webhook_url: &str, message: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let payload = DiscordPayload { content: message, username: "ARK Manager" };

    let res = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Falha ao enviar webhook: {}", e))?;

    if res.status().is_success() || res.status().as_u16() == 204 {
        Ok(())
    } else {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        Err(format!("Discord respondeu {} — {}", status, body))
    }
}

// ---------------------------------------------------------------------------
// Configuração salva no banco
// ---------------------------------------------------------------------------

/// Notificação Discord salva pelo usuário.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    pub id: Option<u32>,
    pub webhook_url: String,
    /// Eventos habilitados: ex. ["server_start","server_stop","backup","player_join"]
    pub events: Vec<String>,
}
