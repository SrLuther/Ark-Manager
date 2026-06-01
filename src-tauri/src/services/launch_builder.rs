use std::path::Path;
use thiserror::Error;

/// Parâmetros de lançamento do servidor ARK.
#[derive(Debug, Clone)]
pub struct LaunchParams {
    pub map: String,
    pub game_port: u16,
    pub query_port: u16,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub max_players: u32,
    pub server_password: String,
    pub admin_password: String,
    pub mods: Vec<String>,
    pub cluster_id: String,
    pub cluster_dir: String,
    pub pvp: bool,
    pub no_battleye: bool,
    pub extra_args: Vec<String>,
}

impl Default for LaunchParams {
    fn default() -> Self {
        Self {
            map: String::from("TheIsland"),
            game_port: 7777,
            query_port: 27015,
            rcon_port: 32330,
            rcon_password: String::from("rconpassword"),
            max_players: 70,
            server_password: String::new(),
            admin_password: String::from("adminpassword"),
            mods: Vec::new(),
            cluster_id: String::new(),
            cluster_dir: String::new(),
            pvp: false,
            no_battleye: false,
            extra_args: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum LaunchBuilderError {
    #[error("Executável do servidor não encontrado: {0}")]
    ExeNotFound(String),
    #[error("Erro ao escrever script de lançamento: {0}")]
    WriteError(String),
}

/// Gera o arquivo `RunServer.cmd` no diretório informado.
///
/// **Regras críticas:**
/// - `SessionName` NUNCA é passado na linha de comando (sempre no INI).
/// - Caminhos com espaços são cercados com aspas.
/// - O script usa `start /wait` para aguardar o processo encerrar.
pub async fn generate_launch_script(
    install_dir: &Path,
    script_dir: &Path,
    params: &LaunchParams,
) -> Result<std::path::PathBuf, LaunchBuilderError> {
    let exe = install_dir
        .join("ShooterGame")
        .join("Binaries")
        .join("Win64")
        .join("ShooterGameServer.exe");

    if !exe.exists() {
        return Err(LaunchBuilderError::ExeNotFound(exe.display().to_string()));
    }

    tokio::fs::create_dir_all(script_dir)
        .await
        .map_err(|e| LaunchBuilderError::WriteError(e.to_string()))?;

    let script_path = script_dir.join("RunServer.cmd");
    let content = build_script_content(&exe, params);

    tokio::fs::write(&script_path, content.as_bytes())
        .await
        .map_err(|e| LaunchBuilderError::WriteError(e.to_string()))?;

    Ok(script_path)
}

/// Constrói o conteúdo do script .cmd.
fn build_script_content(exe: &Path, p: &LaunchParams) -> String {
    let mut args: Vec<String> = Vec::new();

    // Mapa é o primeiro argumento posicional
    args.push(format!("{}?listen", p.map));

    // Portas
    args.push(format!("?Port={}", p.game_port));
    args.push(format!("?QueryPort={}", p.query_port));
    args.push(format!("?RCONPort={}", p.rcon_port));
    args.push(format!("?RCONEnabled=True"));

    // Senhas
    args.push(format!("?ServerAdminPassword={}", p.admin_password));
    if !p.server_password.is_empty() {
        args.push(format!("?ServerPassword={}", p.server_password));
    }

    // MaxPlayers
    args.push(format!("?MaxPlayers={}", p.max_players));

    // ⚠️ SessionName NUNCA aqui — sempre no GameUserSettings.ini

    // Mods (lista separada por vírgula)
    if !p.mods.is_empty() {
        args.push(format!("?GameModIds={}", p.mods.join(",")));
    }

    // Cluster
    if !p.cluster_id.is_empty() {
        args.push(format!("?ClusterId={}", p.cluster_id));
    }
    if !p.cluster_dir.is_empty() {
        args.push(format!(
            "?ClusterDirOverride=\"{}\"",
            p.cluster_dir.replace('\\', "/")
        ));
    }

    // Flags de linha de comando (após --)
    let mut flags: Vec<String> = Vec::new();
    flags.push("-server".to_string());
    flags.push("-log".to_string());
    flags.push("-nosteamclient".to_string());
    flags.push("-game".to_string());
    flags.push("-messaging".to_string());
    if p.pvp {
        // PvP é definido pelo mapa; sem flag dedicada
    }
    if p.no_battleye {
        flags.push("-NoBattlEye".to_string());
    }

    // Args extras do usuário
    for extra in &p.extra_args {
        flags.push(extra.clone());
    }

    let exe_str = format!("\"{}\"", exe.display());
    let map_args = args.join("");
    let flag_str = flags.join(" ");

    format!(
        "@echo off\ncd /d \"{}\"\nstart /wait \"\" {} {} {}\n",
        exe.parent().unwrap_or(exe).display(),
        exe_str,
        map_args,
        flag_str
    )
}

/// Retorna o caminho esperado do script de lançamento dado o diretório do servidor.
pub fn script_path(script_dir: &Path) -> std::path::PathBuf {
    script_dir.join("RunServer.cmd")
}

