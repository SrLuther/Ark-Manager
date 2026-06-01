import { useState } from 'react'
import { ArrowRight, ArrowLeft, Link2, Server } from 'lucide-react'
import { cn, mapLabel, statusLabel } from '../../utils/helpers'
import { Badge, Button } from '../ui'
import type { BadgeVariant } from '../ui/Badge'
import type { Cluster, Server as ServerType } from '../../types'
import toast from 'react-hot-toast'

function statusVariant(s: string): BadgeVariant {
  if (s === 'running')   return 'success'
  if (s === 'error')     return 'error'
  if (s === 'starting' || s === 'stopping') return 'warning'
  if (s === 'updating'  || s === 'installing') return 'purple'
  return 'default'
}

export interface ClusterBuilderProps {
  cluster: Cluster
  allServers: ServerType[]
  onAssign: (serverId: number) => Promise<void>
  onUnassign: (serverId: number) => Promise<void>
}

export function ClusterBuilder({ cluster, allServers, onAssign, onUnassign }: ClusterBuilderProps) {
  const [busy, setBusy] = useState<number | null>(null)

  const inCluster  = allServers.filter((s) => s.clusterId === cluster.id)
  const available  = allServers.filter((s) => s.clusterId === null)

  const handleAssign = async (serverId: number) => {
    setBusy(serverId)
    try {
      await onAssign(serverId)
      toast.success('Servidor vinculado ao cluster')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(null)
    }
  }

  const handleUnassign = async (serverId: number) => {
    setBusy(serverId)
    try {
      await onUnassign(serverId)
      toast.success('Servidor removido do cluster')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBusy(null)
    }
  }

  return (
    <div className="grid grid-cols-[1fr_40px_1fr] gap-3 items-start">
      {/* Coluna: disponíveis */}
      <div className="flex flex-col gap-2">
        <p className="text-xs font-medium text-slate-400 uppercase tracking-wide">
          Disponíveis ({available.length})
        </p>
        <div className="rounded-xl bg-surface-800 border border-surface-700 divide-y divide-surface-700 min-h-[80px]">
          {available.length === 0 && (
            <p className="text-xs text-slate-600 p-4 text-center">
              Nenhum servidor disponível
            </p>
          )}
          {available.map((srv) => (
            <div key={srv.id} className="flex items-center justify-between px-3 py-2.5 gap-2">
              <div className="flex items-center gap-2 min-w-0">
                <Server size={13} className="text-slate-500 shrink-0" />
                <div className="flex flex-col min-w-0">
                  <span className="text-xs text-slate-200 truncate">{srv.name}</span>
                  <span className="text-[10px] text-slate-500">{mapLabel(srv.map)}</span>
                </div>
              </div>
              <div className="flex items-center gap-2 shrink-0">
                <Badge variant={statusVariant(srv.status)}>{statusLabel(srv.status)}</Badge>
                <Button
                  size="sm"
                  variant="ghost"
                  loading={busy === srv.id}
                  onClick={() => handleAssign(srv.id)}
                  title="Vincular ao cluster"
                >
                  <ArrowRight size={12} />
                </Button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Separador central */}
      <div className="flex items-center justify-center pt-8">
        <Link2 size={18} className="text-surface-600" />
      </div>

      {/* Coluna: no cluster */}
      <div className="flex flex-col gap-2">
        <p className="text-xs font-medium text-slate-400 uppercase tracking-wide">
          No cluster <span className="text-ark-400">«{cluster.name}»</span> ({inCluster.length})
        </p>
        <div className="rounded-xl bg-surface-800 border border-ark-900/50 divide-y divide-surface-700 min-h-[80px]">
          {inCluster.length === 0 && (
            <p className="text-xs text-slate-600 p-4 text-center">
              Nenhum servidor vinculado
            </p>
          )}
          {inCluster.map((srv) => (
            <div key={srv.id} className="flex items-center justify-between px-3 py-2.5 gap-2">
              <div className="flex items-center gap-2 min-w-0">
                <Button
                  size="sm"
                  variant="ghost"
                  loading={busy === srv.id}
                  onClick={() => handleUnassign(srv.id)}
                  title="Remover do cluster"
                  className="shrink-0"
                >
                  <ArrowLeft size={12} />
                </Button>
                <div className="flex flex-col min-w-0">
                  <span className="text-xs text-slate-200 truncate">{srv.name}</span>
                  <span className="text-[10px] text-slate-500">{mapLabel(srv.map)}</span>
                </div>
              </div>
              <Badge variant={statusVariant(srv.status)}>{statusLabel(srv.status)}</Badge>
            </div>
          ))}
        </div>
      </div>

      {/* Info do cluster */}
      <div className={cn('col-span-3 rounded-lg bg-surface-900 border border-surface-700 px-3 py-2 text-xs text-slate-500')}>
        <span className="text-slate-400 font-medium">Cluster ID: </span>
        <span className="font-mono">{cluster.clusterId}</span>
        {cluster.clusterDir && (
          <>
            <span className="mx-2 text-surface-600">·</span>
            <span className="text-slate-400">Dir: </span>
            <span className="font-mono">{cluster.clusterDir}</span>
          </>
        )}
      </div>
    </div>
  )
}
