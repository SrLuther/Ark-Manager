// ─────────────────────────────────────────────
// Servidor ARK
// ─────────────────────────────────────────────

export type ServerStatus =
  | 'stopped'
  | 'starting'
  | 'running'
  | 'stopping'
  | 'error'
  | 'updating'
  | 'installing'

export type ArkMap =
  | 'TheIsland'
  | 'ScorchedEarth'
  | 'Aberration'
  | 'Extinction'
  | 'Genesis'
  | 'Genesis2'
  | 'CrystalIsles'
  | 'Ragnarok'
  | 'Valguero'
  | 'LostIsland'
  | 'Fjordur'

export interface Server {
  id: number
  name: string
  map: ArkMap
  status: ServerStatus
  installDir: string
  steamcmdDir: string
  gamePort: number
  queryPort: number
  rconPort: number
  rconPassword: string
  adminPassword: string
  serverPassword: string | null
  maxPlayers: number
  clusterId: number | null
  activeMods: string | null
  pidCached: number | null
  lastStarted: string | null
  lastStopped: string | null
  lastBackup: string | null
  createdAt: string
  updatedAt: string
}

export interface CreateServerRequest {
  name: string
  map: ArkMap
  installDir: string
  steamcmdDir: string
  gamePort: number
  queryPort: number
  rconPort: number
  rconPassword: string
  adminPassword: string
  serverPassword?: string
  maxPlayers: number
}

export interface UpdateServerRequest {
  name?: string
  map?: ArkMap
  status?: ServerStatus
  installDir?: string
  steamcmdDir?: string
  gamePort?: number
  queryPort?: number
  rconPort?: number
  rconPassword?: string
  adminPassword?: string
  serverPassword?: string | null
  maxPlayers?: number
  clusterId?: number | null
  activeMods?: string | null
}

export type ServerResponse = Server

// ─────────────────────────────────────────────
// Backup
// ─────────────────────────────────────────────

export type BackupType = 'auto' | 'manual' | 'pre_update' | 'scheduled'
export type BackupStatus = 'pending' | 'in_progress' | 'completed' | 'failed'

export interface Backup {
  id: number
  serverId: number
  backupType: BackupType
  status: BackupStatus
  backupPath: string | null
  sizeBytes: number | null
  sizeHuman: string
  fileCount: number | null
  errorMessage: string | null
  startedAt: string | null
  completedAt: string | null
  createdAt: string
}

// ─────────────────────────────────────────────
// Tarefa agendada
// ─────────────────────────────────────────────

export type TaskType =
  | 'Restart'
  | 'Saveworld'
  | 'DestroyWildDinos'
  | 'Broadcast'
  | 'ExecuteCommand'
  | 'UpdateServer'
  | 'CreateBackup'

export type TaskResult = 'Success' | 'Failure' | 'Skipped' | 'Running'

export interface ScheduledTask {
  id: number
  serverId: number
  taskName: string | null
  displayName: string
  taskType: TaskType
  cronExpression: string
  command: string | null
  message: string | null
  preWarningMinutes: number
  enabled: boolean
  runCount: number
  lastRun: string | null
  nextRun: string | null
  lastResult: TaskResult | null
  lastError: string | null
  createdAt: string
  updatedAt: string
}

export interface CreateTaskRequest {
  serverId: number
  taskName?: string
  taskType: TaskType
  cronExpression: string
  command?: string
  message?: string
  preWarningMinutes?: number
  enabled?: boolean
}

export interface UpdateTaskRequest {
  taskName?: string
  taskType?: TaskType
  cronExpression?: string
  command?: string
  message?: string
  preWarningMinutes?: number
  enabled?: boolean
}

// ─────────────────────────────────────────────
// Cluster
// ─────────────────────────────────────────────

export interface Cluster {
  id: number
  name: string
  clusterId: string
  clusterDir: string
  description: string | null
}

export interface CreateClusterRequest {
  name: string
  clusterId: string
  clusterDir: string
  description?: string
}

// ─────────────────────────────────────────────
// Hardware / Métricas
// ─────────────────────────────────────────────

export interface SystemMetrics {
  cpuPercent: number
  totalMemoryBytes: number
  usedMemoryBytes: number
  memoryPercent: number
}

export interface ProcessMetrics {
  pid: number
  cpuPercent: number
  memoryBytes: number
  running: boolean
}

// ─────────────────────────────────────────────
// Logs
// ─────────────────────────────────────────────

export type LogLevel = 'Info' | 'Warning' | 'Error' | 'Debug'

export interface LogLine {
  timestamp: string
  level: LogLevel
  message: string
}

// ─────────────────────────────────────────────
// Configuração INI
// ─────────────────────────────────────────────

export interface ServerConfig {
  sessionName: string
  serverPassword: string
  adminPassword: string
  maxPlayers: number
  rconPort: number
  rconPassword: string
  rconEnabled: boolean
  gamePort: number
  queryPort: number
  xpMultiplier: number
  harvestMultiplier: number
  tameSpeedMultiplier: number
  serverPve: boolean
  allowThirdPersonPlayer: boolean
  alwaysNotifyPlayerJoined: boolean
  alwaysNotifyPlayerLeft: boolean
  serverHardcore: boolean
  enableCrossArk: boolean
  clusterId: string
  clusterDirOverride: string
  activeMods: string
  mapName: string
  serverAutoForceRespawnWildDinosCooldown: number
  enableRcon: boolean
}

// ─────────────────────────────────────────────
// Importação
// ─────────────────────────────────────────────

export interface DetectedServer {
  installPath: string
  isInstalled: boolean
  mapName: string | null
  sessionName: string | null
  gamePort: number | null
  queryPort: number | null
  rconPort: number | null
  maxPlayers: number | null
  adminPassword: string | null
  serverPassword: string | null
  mods: string | null
  enablePvp: boolean | null
}

// ─────────────────────────────────────────────
// Mod
// ─────────────────────────────────────────────

export interface ModEntry {
  modId: string
  position: number
  name: string | null
}

// ─────────────────────────────────────────────
// Sincronização (Fase 8–9)
// ─────────────────────────────────────────────

export type SyncStatus = 'synced' | 'syncing' | 'pending' | 'conflict' | 'offline' | 'error'

export type AgentStatus = 'online' | 'offline' | 'pairing'

export interface SyncAgent {
  id: number
  name: string
  address: string
  port: number
  paired: boolean
  lastSeenAt: string | null
  status: AgentStatus
}

export interface DiscoveredAgent {
  name: string
  address: string
  port: number
}

export interface SyncFolder {
  id: number
  name: string
  localPath: string
  agentId: number | null
  status: SyncStatus
  lastSyncAt: string | null
  bytesTransferred: number
  conflictCount: number
  createdAt: string
  updatedAt: string
}

export interface SyncEvent {
  id: number
  folderId: number
  eventType: 'transfer' | 'conflict' | 'error' | 'connected' | 'disconnected' | 'sync_complete'
  path: string | null
  bytes: number | null
  direction: 'upload' | 'download' | null
  message: string | null
  createdAt: string
}

export interface SyncConflict {
  id: number
  folderId: number
  path: string
  localMtime: number
  remoteMtime: number
  resolution: 'local' | 'remote'
  createdAt: string
}

// ─────────────────────────────────────────────
// Eventos Sazonais (Fase 11–12)
// ─────────────────────────────────────────────

export type EventStatus = 'scheduled' | 'active' | 'ended' | 'cancelled' | 'error'

export interface EventRate {
  xpMultiplier: number
  harvestMultiplier: number
  tameSpeedMultiplier: number
  breedingMultiplier: number
  hatchSpeedMultiplier: number
  matureSpeedMultiplier: number
}

export interface SeasonalEvent {
  id: number
  name: string
  description: string | null
  status: EventStatus
  startAt: string
  endAt: string
  rates: EventRate
  serverIds: number[]
  preWarningMinutes: number
  broadcastMessage: string | null
  createdAt: string
  updatedAt: string
}

export interface CreateEventRequest {
  name: string
  description?: string
  startAt: string
  endAt: string
  rates: EventRate
  serverIds: number[]
  preWarningMinutes?: number
  broadcastMessage?: string
}

export interface UpdateEventRequest {
  name?: string
  description?: string
  startAt?: string
  endAt?: string
  rates?: EventRate
  serverIds?: number[]
  preWarningMinutes?: number
  broadcastMessage?: string
}
