import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  Server, Play, Square, RotateCcw, Terminal,
  ScrollText, Settings, Users, MapPin, Network,
} from 'lucide-react'
import { useServerStore } from '../../stores/serverStore'
import { statusLabel, mapLabel } from '../../utils/helpers'
import { Badge, Button, Card } from '../ui'
import type { BadgeVariant } from '../ui/Badge'
import type { Server as ServerType } from '../../types'
import toast from 'react-hot-toast'

function statusVariant(s: string): BadgeVariant {
  if (s === 'running')   return 'success'
  if (s === 'error')     return 'error'
  if (s === 'starting' || s === 'stopping') return 'warning'
  if (s === 'updating'  || s === 'installing') return 'purple'
  return 'default'
}

export interface ServerCardProps {
  server: ServerType
}

export function ServerCard({ server }: ServerCardProps) {
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
  const transitioning = server.status === 'starting' || server.status === 'stopping'

  return (
    <Card className="flex flex-col gap-4 hover:border-surface-500 transition-colors">
      {/* Header */}
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-2 min-w-0">
          <Server size={16} className="text-ark-400 shrink-0" />
          <span className="text-sm font-semibold text-slate-100 truncate">{server.name}</span>
        </div>
        <Badge variant={statusVariant(server.status)}>
          {statusLabel(server.status)}
        </Badge>
      </div>

      {/* Info grid */}
      <div className="grid grid-cols-2 gap-x-4 gap-y-1.5 text-xs">
        <div className="flex items-center gap-1.5 text-slate-400">
          <MapPin size={11} className="shrink-0" />
          <span>{mapLabel(server.map)}</span>
        </div>
        <div className="flex items-center gap-1.5 text-slate-400">
          <Users size={11} className="shrink-0" />
          <span>Máx. {server.maxPlayers}</span>
        </div>
        <div className="flex items-center gap-1.5 text-slate-400">
          <Network size={11} className="shrink-0" />
          <span>:{server.gamePort}</span>
        </div>
        <div className="flex items-center gap-1.5 text-slate-400">
          <Terminal size={11} className="shrink-0" />
          <span>RCON :{server.rconPort}</span>
        </div>
      </div>

      {/* Cluster badge */}
      {server.clusterId && (
        <div className="text-xs text-slate-500">
          Cluster <span className="text-slate-400">#{server.clusterId}</span>
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center gap-1.5 mt-auto pt-1 flex-wrap">
        {stopped && (
          <Button
            size="sm"
            variant="primary"
            loading={busy}
            onClick={() => act(() => startServer(server.id), 'Servidor iniciado')}
          >
            <Play size={11} /> Iniciar
          </Button>
        )}
        {running && (
          <Button
            size="sm"
            variant="danger"
            loading={busy}
            onClick={() => act(() => stopServer(server.id), 'Servidor parado')}
          >
            <Square size={11} /> Parar
          </Button>
        )}
        {(running || transitioning) && (
          <Button
            size="sm"
            variant="secondary"
            loading={busy}
            onClick={() => act(() => restartServer(server.id), 'Servidor reiniciado')}
          >
            <RotateCcw size={11} /> Reiniciar
          </Button>
        )}
        <Button
          size="sm"
          variant="ghost"
          onClick={() => navigate(`/rcon/${server.id}`)}
        >
          <Terminal size={11} /> RCON
        </Button>
        <Button
          size="sm"
          variant="ghost"
          onClick={() => navigate(`/logs/${server.id}`)}
        >
          <ScrollText size={11} /> Logs
        </Button>
        <Button
          size="sm"
          variant="ghost"
          onClick={() => navigate(`/config/${server.id}`)}
        >
          <Settings size={11} /> Config
        </Button>
      </div>
    </Card>
  )
}
