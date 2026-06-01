import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  Server, Play, Square, RotateCcw, RefreshCw,
  Cpu, MemoryStick, HardDrive, Circle, FolderSync, AlertTriangle, CalendarDays,
} from 'lucide-react'
import { useServerStore } from '../stores/serverStore'
import { useSyncStore } from '../stores/syncStore'
import { useSeasonalEventStore } from '../stores/seasonalEventStore'
import { getSystemMetrics } from '../utils/tauri'
import { formatBytes, statusLabel, mapLabel } from '../utils/helpers'
import { Badge, Button, Card, CardHeader, CardTitle } from '../components/ui'
import type { ServerResponse, SystemMetrics } from '../types'
import toast from 'react-hot-toast'

function statusBadgeVariant(s: string) {
  if (s === 'running') return 'success'
  if (s === 'error') return 'error'
  if (s === 'starting' || s === 'stopping') return 'warning'
  if (s === 'updating' || s === 'installing') return 'purple'
  return 'default' as const
}

function ServerCard({ server }: { server: ServerResponse }) {
  const { startServer, stopServer, restartServer } = useServerStore()
  const navigate = useNavigate()
  const [busy, setBusy] = useState(false)

  const act = async (fn: () => Promise<void>, label: string) => {
    setBusy(true)
    try {
      await fn()
      toast.success(label)
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(false)
    }
  }

  const running = server.status === 'running'
  const stopped = server.status === 'stopped'

  return (
    <Card className="flex flex-col gap-3">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-2">
          <Server size={16} className="text-ark-400 shrink-0" />
          <span className="text-sm font-semibold text-slate-100 truncate max-w-[160px]">
            {server.name}
          </span>
        </div>
        <Badge variant={statusBadgeVariant(server.status)}>
          {statusLabel(server.status)}
        </Badge>
      </div>

      <div className="text-xs text-slate-400 space-y-1">
        <div className="flex justify-between">
          <span>Mapa</span>
          <span className="text-slate-300">{mapLabel(server.map as any)}</span>
        </div>
        <div className="flex justify-between">
          <span>Porta</span>
          <span className="text-slate-300">{server.gamePort}</span>
        </div>
        <div className="flex justify-between">
          <span>Máx. jogadores</span>
          <span className="text-slate-300">{server.maxPlayers}</span>
        </div>
      </div>

      <div className="flex gap-1.5 mt-auto pt-1 flex-wrap">
        {stopped && (
          <Button size="sm" variant="primary" loading={busy} onClick={() => act(() => startServer(server.id), 'Servidor iniciado')}>
            <Play size={12} /> Iniciar
          </Button>
        )}
        {running && (
          <>
            <Button size="sm" variant="danger" loading={busy} onClick={() => act(() => stopServer(server.id), 'Servidor parado')}>
              <Square size={12} /> Parar
            </Button>
            <Button size="sm" variant="secondary" loading={busy} onClick={() => act(() => restartServer(server.id), 'Servidor reiniciado')}>
              <RotateCcw size={12} /> Reiniciar
            </Button>
          </>
        )}
        <Button size="sm" variant="ghost" onClick={() => navigate(`/rcon/${server.id}`)}>
          RCON
        </Button>
        <Button size="sm" variant="ghost" onClick={() => navigate(`/logs/${server.id}`)}>
          Logs
        </Button>
      </div>
    </Card>
  )
}

export default function Dashboard() {
  const { servers, fetchServers, loading } = useServerStore()
  const { folders, fetchFolders } = useSyncStore()
  const { events: seasonalEvents, fetchEvents: fetchSeasonalEvents } = useSeasonalEventStore()
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  const navigate = useNavigate()

  useEffect(() => {
    fetchServers()
    fetchFolders()
    fetchSeasonalEvents()
    getSystemMetrics().then(setMetrics).catch(() => {})
    const interval = setInterval(() => {
      getSystemMetrics().then(setMetrics).catch(() => {})
    }, 5000)
    return () => clearInterval(interval)
  }, [fetchServers, fetchFolders, fetchSeasonalEvents])

  const running = servers.filter(s => s.status === 'running').length
  const stopped = servers.filter(s => s.status === 'stopped').length
  const foldersWithConflict = folders.filter(f => f.conflictCount > 0).length
  const foldersOffline = folders.filter(f => f.status === 'offline' || f.status === 'error').length
  const activeEvents = seasonalEvents.filter(e => e.status === 'active')
  const scheduledEvents = seasonalEvents.filter(e => e.status === 'scheduled')

  return (
    <div className="p-6 space-y-6 h-full overflow-auto">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-slate-100">Dashboard</h1>
          <p className="text-xs text-slate-500 mt-0.5">Visão geral dos servidores ARK</p>
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={() => fetchServers()} loading={loading}>
            <RefreshCw size={13} /> Atualizar
          </Button>
          <Button size="sm" onClick={() => navigate('/servers')}>
            <Server size={13} /> Gerenciar servidores
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <Card className="flex items-center gap-3">
          <div className="p-2 bg-ark-900/50 rounded-lg"><Server size={18} className="text-ark-400" /></div>
          <div>
            <p className="text-xs text-slate-500">Total</p>
            <p className="text-xl font-bold text-slate-100">{servers.length}</p>
          </div>
        </Card>
        <Card className="flex items-center gap-3">
          <div className="p-2 bg-emerald-900/50 rounded-lg"><Circle size={18} className="text-emerald-400 fill-emerald-400" /></div>
          <div>
            <p className="text-xs text-slate-500">Rodando</p>
            <p className="text-xl font-bold text-emerald-400">{running}</p>
          </div>
        </Card>
        <Card className="flex items-center gap-3">
          <div className="p-2 bg-surface-700 rounded-lg"><Circle size={18} className="text-slate-500" /></div>
          <div>
            <p className="text-xs text-slate-500">Parados</p>
            <p className="text-xl font-bold text-slate-400">{stopped}</p>
          </div>
        </Card>
        {metrics && (
          <Card className="flex items-center gap-3">
            <div className="p-2 bg-blue-900/50 rounded-lg"><Cpu size={18} className="text-blue-400" /></div>
            <div>
              <p className="text-xs text-slate-500">CPU</p>
              <p className="text-xl font-bold text-blue-400">{metrics.cpuPercent.toFixed(1)}%</p>
            </div>
          </Card>
        )}
      </div>

      {/* Sincronização */}
      {folders.length > 0 && (
        <div>
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold text-slate-300 flex items-center gap-2">
              <FolderSync size={14} className="text-ark-400" /> Sincronização
            </h2>
            <Button size="sm" variant="ghost" onClick={() => navigate('/sync')}>
              Gerenciar
            </Button>
          </div>
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
            {foldersWithConflict > 0 && (
              <Card className="flex items-center gap-3 border-orange-800/60">
                <div className="p-2 bg-orange-900/40 rounded-lg">
                  <AlertTriangle size={16} className="text-orange-400" />
                </div>
                <div>
                  <p className="text-xs text-slate-500">Conflitos</p>
                  <p className="text-lg font-bold text-orange-400">{foldersWithConflict}</p>
                </div>
              </Card>
            )}
            {foldersOffline > 0 && (
              <Card className="flex items-center gap-3 border-red-900/60">
                <div className="p-2 bg-red-900/40 rounded-lg">
                  <FolderSync size={16} className="text-red-400" />
                </div>
                <div>
                  <p className="text-xs text-slate-500">Offline / Erro</p>
                  <p className="text-lg font-bold text-red-400">{foldersOffline}</p>
                </div>
              </Card>
            )}
            <Card className="flex items-center gap-3">
              <div className="p-2 bg-ark-900/50 rounded-lg">
                <FolderSync size={16} className="text-ark-400" />
              </div>
              <div>
                <p className="text-xs text-slate-500">Pastas</p>
                <p className="text-lg font-bold text-slate-100">{folders.length}/5</p>
              </div>
            </Card>
          </div>
        </div>
      )}

      {/* Eventos Sazonais */}
      {(activeEvents.length > 0 || scheduledEvents.length > 0) && (
        <div>
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold text-slate-300 flex items-center gap-2">
              <CalendarDays size={14} className="text-emerald-400" /> Eventos Sazonais
            </h2>
            <Button size="sm" variant="ghost" onClick={() => navigate('/events')}>
              Ver todos
            </Button>
          </div>
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
            {activeEvents.length > 0 && (
              <Card className="flex items-center gap-3 border-emerald-800/60">
                <div className="p-2 bg-emerald-900/40 rounded-lg">
                  <CalendarDays size={16} className="text-emerald-400" />
                </div>
                <div>
                  <p className="text-xs text-slate-500">Ativos</p>
                  <p className="text-lg font-bold text-emerald-400">{activeEvents.length}</p>
                </div>
              </Card>
            )}
            {scheduledEvents.length > 0 && (
              <Card className="flex items-center gap-3 border-ark-800/60">
                <div className="p-2 bg-ark-900/40 rounded-lg">
                  <CalendarDays size={16} className="text-ark-400" />
                </div>
                <div>
                  <p className="text-xs text-slate-500">Agendados</p>
                  <p className="text-lg font-bold text-ark-400">{scheduledEvents.length}</p>
                </div>
              </Card>
            )}
          </div>
          {activeEvents.map(ev => (
            <div key={ev.id} className="mt-2 flex items-center gap-2 rounded-lg px-3 py-2 bg-emerald-900/10 border border-emerald-800/50 text-xs text-emerald-300">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse shrink-0" />
              <span className="font-medium">{ev.name}</span>
              <span className="text-emerald-600">— ativo agora</span>
            </div>
          ))}
        </div>
      )}

      {/* RAM */}
      {metrics && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <MemoryStick size={14} className="text-ark-400" /> RAM do sistema
            </CardTitle>
            <span className="text-xs text-slate-400">
              {formatBytes(metrics.usedMemoryBytes)} / {formatBytes(metrics.totalMemoryBytes)}
            </span>
          </CardHeader>
          <div className="w-full bg-surface-700 rounded-full h-2">
            <div
              className="bg-ark-500 h-2 rounded-full transition-all"
              style={{ width: `${Math.min(metrics.memoryPercent, 100)}%` }}
            />
          </div>
          <p className="text-xs text-slate-500 mt-1">{metrics.memoryPercent.toFixed(1)}% utilizado</p>
        </Card>
      )}

      {/* Servidores */}
      <div>
        <h2 className="text-sm font-semibold text-slate-300 mb-3">Servidores</h2>
        {loading && servers.length === 0 ? (
          <div className="text-center py-12 text-slate-500 text-sm">Carregando...</div>
        ) : servers.length === 0 ? (
          <Card className="text-center py-12">
            <HardDrive size={32} className="text-slate-600 mx-auto mb-3" />
            <p className="text-slate-400 text-sm">Nenhum servidor configurado</p>
            <p className="text-slate-500 text-xs mt-1">Adicione seu primeiro servidor na página Servidores.</p>
            <Button className="mt-4" size="sm" onClick={() => navigate('/servers')}>
              Adicionar servidor
            </Button>
          </Card>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
            {servers.map(s => <ServerCard key={s.id} server={s as any} />)}
          </div>
        )}
      </div>
    </div>
  )
}

