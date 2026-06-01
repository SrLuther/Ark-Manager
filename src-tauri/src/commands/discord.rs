//! Comandos Tauri para configuração de notificações Discord.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;
use crate::services::discord::send_webhook;

// ---------------------------------------------------------------------------
// Persistência da configuração Discord (tabela `settings` — chave/valor)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordSettings {
    pub webhook_url: String,
    pub enabled_events: Vec<String>,
}

/// Salva a configuração do Discord Webhook no banco (tabela `settings`).
#[tauri::command]
pub async fn save_discord_config(
    webhook_url: String,
    enabled_events: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let events_json =
        serde_json::to_string(&enabled_events).map_err(|e| e.to_string())?;

    // Upsert na tabela settings (criada na migration v3 ou similar)
    sqlx::query(
        "INSERT INTO am_settings (`key`, `value`) VALUES ('discord_webhook', ?)
         ON DUPLICATE KEY UPDATE `value` = VALUES(`value`)",
    )
    .bind(&webhook_url)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO am_settings (`key`, `value`) VALUES ('discord_events', ?)
         ON DUPLICATE KEY UPDATE `value` = VALUES(`value`)",
    )
    .bind(&events_json)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Carrega a configuração Discord atual do banco.
#[tauri::command]
pub async fn get_discord_config(
    state: State<'_, AppState>,
) -> Result<Option<DiscordSettings>, String> {
    let webhook: Option<String> = sqlx::query_scalar(
        "SELECT `value` FROM am_settings WHERE `key` = 'discord_webhook' LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .flatten();

    let events_raw: Option<String> = sqlx::query_scalar(
        "SELECT `value` FROM am_settings WHERE `key` = 'discord_events' LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .flatten();

    match webhook {
        None => Ok(None),
        Some(url) => {
            let enabled_events: Vec<String> = events_raw
                .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
                .unwrap_or_default();
            Ok(Some(DiscordSettings { webhook_url: url, enabled_events }))
        }
    }
}

/// Envia uma mensagem de teste para o webhook Discord informado.
#[tauri::command]
pub async fn test_discord_webhook(webhook_url: String) -> Result<(), String> {
    send_webhook(
        &webhook_url,
        "✅ ARK Manager — webhook configurado com sucesso!",
    )
    .await
}
