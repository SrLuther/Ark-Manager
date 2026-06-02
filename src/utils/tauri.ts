import { invoke as _invoke } from '@tauri-apps/api/core'
import type {
  ServerResponse,
  CreateServerRequest,
  UpdateServerRequest,
  Backup,
  ScheduledTask,
  CreateTaskRequest,
  UpdateTaskRequest,
  Cluster,
  CreateClusterRequest,
  SystemMetrics,
  ProcessMetrics,
  ServerConfig,
  DetectedServer,
  ModEntry,
  SyncAgent,
  DiscoveredAgent,
  SyncFolder,
  SyncEvent,
  SyncConflict,
  SeasonalEvent,
  CreateEventRequest,
} from '../types'

// ─────────────────────────────────────────────
// Wrapper de invoke com erros em PT-BR
// ─────────────────────────────────────────────

function traduzirErro(msg: string): string {
  if (msg.includes('state not managed') || msg.includes('You must call `.manage()`'))
    return 'Banco de dados não pronto. Aguarde a inicialização do aplicativo.'
  if (msg.includes('Communications link failure') || msg.includes('timed out'))
    return 'Tempo limite de conexão com o banco de dados esgotado.'
  if (msg.includes('server not found') || msg.includes('Servidor não encontrado'))
    return 'Servidor não encontrado.'
  if (msg.includes('Not Found') || msg.includes('not found'))
    return 'Recurso não encontrado.'
  return msg
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await _invoke<T>(cmd, args)
  } catch (err) {
    throw new Error(traduzirErro(String(err)))
  }
}

// ─────────────────────────────────────────────
// Servidor
// ─────────────────────────────────────────────

export const listServers = () =>
  invoke<ServerResponse[]>('list_servers')

export const getServer = (id: number) =>
  invoke<ServerResponse>('get_server', { id })

export const createServer = (req: CreateServerRequest) =>
  invoke<ServerResponse>('create_server', { req })

export const updateServer = (id: number, req: UpdateServerRequest) =>
  invoke<ServerResponse>('update_server', { id, req })

export const deleteServer = (id: number) =>
  invoke<void>('delete_server', { id })

export const startServer = (id: number) =>
  invoke<void>('start_server', { id })

export const stopServer = (id: number) =>
  invoke<void>('stop_server', { id })

export const restartServer = (id: number) =>
  invoke<void>('restart_server', { id })

export const serverStatus = (id: number) =>
  invoke<ServerResponse>('server_status', { id })

// ─────────────────────────────────────────────
// Instalação
// ─────────────────────────────────────────────

export const installSteamcmd = (steamcmdDir: string) =>
  invoke<void>('install_steamcmd', { steamcmdDir })

export const isSteamcmdInstalled = (steamcmdDir: string) =>
  invoke<boolean>('is_steamcmd_installed', { steamcmdDir })

export const installArkServer = (steamcmdDir: string, installDir: string) =>
  invoke<void>('install_ark_server', { steamcmdDir, installDir })

export const updateArkServer = (steamcmdDir: string, installDir: string) =>
  invoke<void>('update_ark_server', { steamcmdDir, installDir })

export const isServerInstalled = (installDir: string) =>
  invoke<boolean>('is_server_installed', { installDir })

// ─────────────────────────────────────────────
// Configuração INI
// ─────────────────────────────────────────────

export const readGameUserSettings = (installDir: string) =>
  invoke<Record<string, Record<string, string>>>('read_game_user_settings', { installDir })

export const readGameIni = (installDir: string) =>
  invoke<Record<string, Record<string, string>>>('read_game_ini', { installDir })

export const saveServerConfig = (installDir: string, config: ServerConfig) =>
  invoke<void>('save_server_config', { installDir, config })

export const getConfigDir = (installDir: string) =>
  invoke<string>('get_config_dir', { installDir })

// ─────────────────────────────────────────────
// RCON
// ─────────────────────────────────────────────

export const rconConnect = (serverId: number, host: string, port: number, password: string) =>
  invoke<void>('rcon_connect', { serverId, host, port, password })

export const rconSendCommand = (serverId: number, command: string) =>
  invoke<string>('rcon_send_command', { serverId, command })

export const rconDisconnect = (serverId: number) =>
  invoke<void>('rcon_disconnect', { serverId })

export const rconExecute = (host: string, port: number, password: string, command: string) =>
  invoke<string>('rcon_execute', { host, port, password, command })

export const rconIsConnected = (serverId: number) =>
  invoke<boolean>('rcon_is_connected', { serverId })

// ─────────────────────────────────────────────
// Logs
// ─────────────────────────────────────────────

export const startLogWatcher = (serverId: number, installDir: string) =>
  invoke<void>('start_log_watcher', { serverId, installDir })

export const stopLogWatcher = (serverId: number) =>
  invoke<void>('stop_log_watcher', { serverId })

export const isLogWatcherActive = (serverId: number) =>
  invoke<boolean>('is_log_watcher_active', { serverId })

// ─────────────────────────────────────────────
// Mods
// ─────────────────────────────────────────────

export const listMods = (serverId: number) =>
  invoke<ModEntry[]>('list_mods', { serverId })

export const addMod = (serverId: number, modId: string) =>
  invoke<void>('add_mod', { serverId, modId })

export const removeMod = (serverId: number, modId: string) =>
  invoke<void>('remove_mod', { serverId, modId })

export const reorderMods = (serverId: number, orderedIds: string[]) =>
  invoke<void>('reorder_mods', { serverId, orderedIds })

// ─────────────────────────────────────────────
// Cluster
// ─────────────────────────────────────────────

export const listClusters = () =>
  invoke<Cluster[]>('list_clusters')

export const getCluster = (id: number) =>
  invoke<Cluster>('get_cluster', { id })

export const createCluster = (req: CreateClusterRequest) =>
  invoke<Cluster>('create_cluster', { req })

export const deleteCluster = (id: number) =>
  invoke<void>('delete_cluster', { id })

export const assignServerToCluster = (serverId: number, clusterId: number) =>
  invoke<void>('assign_server_to_cluster', { serverId, clusterId })

export const unassignServerFromCluster = (serverId: number) =>
  invoke<void>('unassign_server_from_cluster', { serverId })

// ─────────────────────────────────────────────
// Backup
// ─────────────────────────────────────────────

export const listBackups = (serverId: number) =>
  invoke<Backup[]>('list_backups', { serverId })

export const createBackup = (serverId: number, installDir: string, backupBaseDir: string) =>
  invoke<void>('create_backup', { serverId, installDir, backupBaseDir })

export const restoreBackup = (backupPath: string, installDir: string) =>
  invoke<void>('restore_backup', { backupPath, installDir })

export const pruneBackups = (backupBaseDir: string, serverId: number, keepCount: number) =>
  invoke<void>('prune_backups', { backupBaseDir, serverId, keepCount })

// ─────────────────────────────────────────────
// Scheduler
// ─────────────────────────────────────────────

export const listTasks = (serverId: number) =>
  invoke<ScheduledTask[]>('list_tasks', { serverId })

export const createTask = (req: CreateTaskRequest) =>
  invoke<ScheduledTask>('create_task', { req })

export const updateTask = (id: number, req: UpdateTaskRequest) =>
  invoke<ScheduledTask>('update_task', { id, req })

export const deleteTask = (id: number) =>
  invoke<void>('delete_task', { id })

export const validateCronExpression = (cronExpr: string) =>
  invoke<string | null>('validate_cron_expression', { cronExpr })

// ─────────────────────────────────────────────
// Hardware
// ─────────────────────────────────────────────

export const getSystemMetrics = () =>
  invoke<SystemMetrics>('get_system_metrics')

export const getProcessMetrics = (pid: number) =>
  invoke<ProcessMetrics>('get_process_metrics', { pid })

export const findServerProcess = (processName?: string) =>
  invoke<number | null>('find_server_process', { processName })

// ─────────────────────────────────────────────
// Importação
// ─────────────────────────────────────────────

export const detectExistingServer = (installDir: string) =>
  invoke<DetectedServer>('detect_existing_server', { installDir })

export const importServer = (req: CreateServerRequest) =>
  invoke<ServerResponse>('import_server', { req })

// ─────────────────────────────────────────────
// Rede
// ─────────────────────────────────────────────

export const detectPortConflicts = (gamePort: number, queryPort: number, rconPort: number) =>
  invoke<number[]>('detect_port_conflicts', { gamePort, queryPort, rconPort })

export const suggestAvailablePort = (base: number) =>
  invoke<number>('suggest_available_port', { base })

// ─────────────────────────────────────────────
// Agente de rede
// ─────────────────────────────────────────────

export const discoverAgents = () =>
  invoke<DiscoveredAgent[]>('discover_agents')

export const listAgents = () =>
  invoke<SyncAgent[]>('list_agents')

export const pairAgent = (address: string, port: number, code: string) =>
  invoke<SyncAgent>('pair_agent', { address, port, code })

export const removeAgent = (id: number) =>
  invoke<void>('remove_agent', { id })

export const getAgentStatus = (address: string, port: number) =>
  invoke<boolean>('get_agent_status', { address, port })

export const generatePairingCode = () =>
  invoke<string>('generate_pairing_code')

// ─────────────────────────────────────────────
// Sincronização de Pastas
// ─────────────────────────────────────────────

export const listSyncFolders = () =>
  invoke<SyncFolder[]>('list_sync_folders')

export const addSyncFolder = (name: string, localPath: string, agentId?: number) =>
  invoke<SyncFolder>('add_sync_folder', { name, localPath, agentId: agentId ?? null })

export const removeSyncFolder = (id: number) =>
  invoke<void>('remove_sync_folder', { id })

export const forceSync = (folderId: number) =>
  invoke<void>('force_sync', { folderId })

export const getSyncEvents = (folderId: number, limit?: number) =>
  invoke<SyncEvent[]>('get_sync_events', { folderId, limit: limit ?? null })

export const getSyncConflicts = (folderId: number) =>
  invoke<SyncConflict[]>('get_sync_conflicts', { folderId })

// ─────────────────────────────────────────────
// Discord Webhook
// ─────────────────────────────────────────────

export interface DiscordSettings {
  webhookUrl: string
  enabledEvents: string[]
}

export const saveDiscordConfig = (webhookUrl: string, enabledEvents: string[]) =>
  invoke<void>('save_discord_config', { webhookUrl, enabledEvents })

export const getDiscordConfig = () =>
  invoke<DiscordSettings | null>('get_discord_config')

export const testDiscordWebhook = (webhookUrl: string) =>
  invoke<void>('test_discord_webhook', { webhookUrl })

// ─────────────────────────────────────────────
// Eventos Sazonais
// ─────────────────────────────────────────────

export const listSeasonalEvents = () =>
  invoke<SeasonalEvent[]>('list_seasonal_events')

export const getSeasonalEvent = (id: number) =>
  invoke<SeasonalEvent>('get_seasonal_event', { id })

export const createSeasonalEvent = (req: CreateEventRequest) =>
  invoke<SeasonalEvent>('create_seasonal_event', { req })

export const cancelSeasonalEvent = (id: number) =>
  invoke<void>('cancel_seasonal_event', { id })

export const forceStartEvent = (id: number) =>
  invoke<void>('force_start_event', { id })

export const forceEndEvent = (id: number) =>
  invoke<void>('force_end_event', { id })

export const getEventStatus = (id: number) =>
  invoke<string>('get_event_status', { id })


