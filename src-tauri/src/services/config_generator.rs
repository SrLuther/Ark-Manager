use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Configuração completa de um servidor ARK para geração dos INIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    // [SessionSettings]
    pub session_name: String,
    pub server_password: String,
    pub admin_password: String,
    pub max_players: u32,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub rcon_enabled: bool,

    // Taxas gerais [/Script/ShooterGame.ShooterGameMode]
    pub xp_multiplier: f64,
    pub harvest_amount_multiplier: f64,
    pub taming_speed_multiplier: f64,
    pub mating_interval_multiplier: f64,
    pub egg_hatch_speed_multiplier: f64,
    pub baby_mature_speed_multiplier: f64,
    pub player_character_food_drain_multiplier: f64,
    pub player_character_water_drain_multiplier: f64,
    pub player_character_stamina_drain_multiplier: f64,
    pub player_character_health_recovery_multiplier: f64,
    pub dino_character_food_drain_multiplier: f64,
    pub dino_character_stamina_drain_multiplier: f64,
    pub dino_character_health_recovery_multiplier: f64,
    pub player_damage_multiplier: f64,
    pub dino_damage_multiplier: f64,
    pub structure_damage_multiplier: f64,
    pub player_resistance_multiplier: f64,
    pub dino_resistance_multiplier: f64,
    pub structure_resistance_multiplier: f64,
    pub resource_no_replenish_radius_players: f64,
    pub resource_no_replenish_radius_structures: f64,
    pub dino_count_multiplier: f64,
    pub max_tamed_dinos: u32,

    // Cluster
    pub cluster_id: String,
    pub cluster_override_upload_cooldown: bool,

    // PvP
    pub pvp: bool,
    pub force_offline_pvp: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            session_name: String::new(),
            server_password: String::new(),
            admin_password: String::from("adminpassword"),
            max_players: 70,
            rcon_port: 32330,
            rcon_password: String::from("rconpassword"),
            rcon_enabled: true,
            xp_multiplier: 1.0,
            harvest_amount_multiplier: 1.0,
            taming_speed_multiplier: 1.0,
            mating_interval_multiplier: 1.0,
            egg_hatch_speed_multiplier: 1.0,
            baby_mature_speed_multiplier: 1.0,
            player_character_food_drain_multiplier: 1.0,
            player_character_water_drain_multiplier: 1.0,
            player_character_stamina_drain_multiplier: 1.0,
            player_character_health_recovery_multiplier: 1.0,
            dino_character_food_drain_multiplier: 1.0,
            dino_character_stamina_drain_multiplier: 1.0,
            dino_character_health_recovery_multiplier: 1.0,
            player_damage_multiplier: 1.0,
            dino_damage_multiplier: 1.0,
            structure_damage_multiplier: 1.0,
            player_resistance_multiplier: 1.0,
            dino_resistance_multiplier: 1.0,
            structure_resistance_multiplier: 1.0,
            resource_no_replenish_radius_players: 1.0,
            resource_no_replenish_radius_structures: 1.0,
            dino_count_multiplier: 1.0,
            max_tamed_dinos: 5000,
            cluster_id: String::new(),
            cluster_override_upload_cooldown: false,
            pvp: false,
            force_offline_pvp: false,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigGeneratorError {
    #[error("Erro ao criar diretório de configuração: {0}")]
    DirCreate(String),
    #[error("Erro ao escrever arquivo de configuração: {0}")]
    WriteError(String),
}

/// Gera GameUserSettings.ini e Game.ini no diretório de config do servidor.
/// `config_dir` deve ser: `<install_dir>/ShooterGame/Saved/Config/WindowsServer/`
pub async fn generate_configs(
    config_dir: &Path,
    cfg: &ServerConfig,
) -> Result<(), ConfigGeneratorError> {
    tokio::fs::create_dir_all(config_dir)
        .await
        .map_err(|e| ConfigGeneratorError::DirCreate(e.to_string()))?;

    let gus = build_game_user_settings(cfg);
    let game_ini = build_game_ini(cfg);

    write_utf16le(config_dir.join("GameUserSettings.ini"), &gus).await?;
    write_utf16le(config_dir.join("Game.ini"), &game_ini).await?;

    Ok(())
}

/// Constrói o conteúdo do GameUserSettings.ini.
fn build_game_user_settings(cfg: &ServerConfig) -> String {
    let mut s = String::new();

    // [SessionSettings] — SessionName OBRIGATORIAMENTE aqui, nunca na CLI
    s.push_str("[SessionSettings]\n");
    s.push_str(&format!("SessionName={}\n", cfg.session_name));
    s.push_str(&format!("MaxPlayers={}\n", cfg.max_players));
    if !cfg.server_password.is_empty() {
        s.push_str(&format!("ServerPassword={}\n", cfg.server_password));
    }
    s.push('\n');

    // [ServerSettings]
    s.push_str("[ServerSettings]\n");
    s.push_str(&format!("ServerAdminPassword={}\n", cfg.admin_password));
    s.push_str(&format!(
        "RCONEnabled={}\n",
        if cfg.rcon_enabled { "True" } else { "False" }
    ));
    s.push_str(&format!("RCONPort={}\n", cfg.rcon_port));
    s.push_str(&format!("AdminLogging=True\n"));
    s.push_str(&format!(
        "AllowThirdPersonPlayer={}\n",
        if cfg.pvp { "False" } else { "True" }
    ));
    s.push('\n');

    // [/Script/ShooterGame.ShooterGameUserSettings]
    s.push_str("[/Script/ShooterGame.ShooterGameUserSettings]\n");
    s.push_str("MasterSoundVolume=1.000000\n");
    s.push('\n');

    s
}

/// Constrói o conteúdo do Game.ini.
fn build_game_ini(cfg: &ServerConfig) -> String {
    let mut s = String::new();

    s.push_str("[/Script/ShooterGame.ShooterGameMode]\n");
    s.push_str(&format!("XPMultiplier={:.6}\n", cfg.xp_multiplier));
    s.push_str(&format!(
        "HarvestAmountMultiplier={:.6}\n",
        cfg.harvest_amount_multiplier
    ));
    s.push_str(&format!(
        "TamingSpeedMultiplier={:.6}\n",
        cfg.taming_speed_multiplier
    ));
    s.push_str(&format!(
        "MatingIntervalMultiplier={:.6}\n",
        cfg.mating_interval_multiplier
    ));
    s.push_str(&format!(
        "EggHatchSpeedMultiplier={:.6}\n",
        cfg.egg_hatch_speed_multiplier
    ));
    s.push_str(&format!(
        "BabyMatureSpeedMultiplier={:.6}\n",
        cfg.baby_mature_speed_multiplier
    ));
    s.push_str(&format!(
        "PlayerCharacterFoodDrainMultiplier={:.6}\n",
        cfg.player_character_food_drain_multiplier
    ));
    s.push_str(&format!(
        "PlayerCharacterWaterDrainMultiplier={:.6}\n",
        cfg.player_character_water_drain_multiplier
    ));
    s.push_str(&format!(
        "PlayerCharacterStaminaDrainMultiplier={:.6}\n",
        cfg.player_character_stamina_drain_multiplier
    ));
    s.push_str(&format!(
        "PlayerCharacterHealthRecoveryMultiplier={:.6}\n",
        cfg.player_character_health_recovery_multiplier
    ));
    s.push_str(&format!(
        "DinoCharacterFoodDrainMultiplier={:.6}\n",
        cfg.dino_character_food_drain_multiplier
    ));
    s.push_str(&format!(
        "DinoCharacterStaminaDrainMultiplier={:.6}\n",
        cfg.dino_character_stamina_drain_multiplier
    ));
    s.push_str(&format!(
        "DinoCharacterHealthRecoveryMultiplier={:.6}\n",
        cfg.dino_character_health_recovery_multiplier
    ));
    s.push_str(&format!(
        "PlayerDamageMultiplier={:.6}\n",
        cfg.player_damage_multiplier
    ));
    s.push_str(&format!(
        "DinoDamageMultiplier={:.6}\n",
        cfg.dino_damage_multiplier
    ));
    s.push_str(&format!(
        "StructureDamageMultiplier={:.6}\n",
        cfg.structure_damage_multiplier
    ));
    s.push_str(&format!(
        "PlayerResistanceMultiplier={:.6}\n",
        cfg.player_resistance_multiplier
    ));
    s.push_str(&format!(
        "DinoResistanceMultiplier={:.6}\n",
        cfg.dino_resistance_multiplier
    ));
    s.push_str(&format!(
        "StructureResistanceMultiplier={:.6}\n",
        cfg.structure_resistance_multiplier
    ));
    s.push_str(&format!(
        "ResourceNoReplenishRadiusPlayers={:.6}\n",
        cfg.resource_no_replenish_radius_players
    ));
    s.push_str(&format!(
        "ResourceNoReplenishRadiusStructures={:.6}\n",
        cfg.resource_no_replenish_radius_structures
    ));
    s.push_str(&format!(
        "DinoCountMultiplier={:.6}\n",
        cfg.dino_count_multiplier
    ));
    s.push_str(&format!("MaxTamedDinos={}\n", cfg.max_tamed_dinos));

    // Cluster
    if !cfg.cluster_id.is_empty() {
        s.push_str(&format!("ClusterId={}\n", cfg.cluster_id));
        s.push_str(&format!(
            "ClusterOverrideUploadCooldown={}\n",
            if cfg.cluster_override_upload_cooldown {
                "True"
            } else {
                "False"
            }
        ));
    }

    s.push('\n');
    s
}

/// Escreve um arquivo com encoding UTF-16 LE + BOM (obrigatório para ARK).
async fn write_utf16le(
    path: impl AsRef<Path>,
    content: &str,
) -> Result<(), ConfigGeneratorError> {
    let path = path.as_ref();

    // BOM UTF-16 LE: FF FE
    let mut bytes: Vec<u8> = vec![0xFF, 0xFE];

    // Converte cada char em UTF-16 LE
    for unit in content.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }

    tokio::fs::write(path, &bytes)
        .await
        .map_err(|e| ConfigGeneratorError::WriteError(format!("{}: {}", path.display(), e)))?;

    Ok(())
}

/// Retorna o caminho do diretório de configuração dado o diretório de instalação do servidor.
pub fn config_dir(install_dir: &Path) -> std::path::PathBuf {
    install_dir
        .join("ShooterGame")
        .join("Saved")
        .join("Config")
        .join("WindowsServer")
}

