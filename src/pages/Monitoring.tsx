import { useEffect, useState } from 'react'
import { Activity, Cpu, MemoryStick, Server, RefreshCw } from 'lucide-react'
import { useServerStore } from '../stores/serverStore'
import { getSystemMetrics, getProcessMetrics, findServerProcess } from '../utils/tauri'
import { formatBytes } from '../utils/helpers'
import { Card, Badge } from '../components/ui'
import type { SystemMetrics, Server as ServerType } from '../types'

interface ServerProc {
  server: ServerType
  pid: number | null
  cpu: number
  memBytes: number
}

function MetricBar({ label, value, max, unit }: { label: string; value: number; max: number; unit?: string }) {
  const pct = Math.min(100, (value / max) * 100)
  const color = pct > 85 ? 'bg-red-500' : pct > 65 ? 'bg-yellow-500' : 'bg-ark-500'
  return (
    <div>
      <div className="flex justify-between text-xs text-slate-400 mb-1">
        <span>{label}</span>
        <span>{unit ? `${value.toFixed(1)}${unit}` : formatBytes(value)} / {unit ? `${max.toFixed(1)}${unit}` : formatBytes(max)}</span>
      </div>
      <div className="h-2 bg-surface-700 rounded-full overflow-hidden">
        <div className={`h-full rounded-full transition-all duration-500 ${color}`} style={{ width: `${pct}%` }} />
      </div>
    </div>
  )
}

export default function Monitoring() {
  const { servers } = useServerStore()
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  const [procs, setProcs] = useState<ServerProc[]>([])
  const [refreshing, setRefreshing] = useState(false)

  const refresh = async () => {
    setRefreshing(true)
    try {
      const m = await getSystemMetrics()
      setMetrics(m)

      const procData = await Promise.all(
        servers.map(async (s): Promise<ServerProc> => {
          try {
            const pid = await findServerProcess('ShooterGameServer')
            if (!pid) return { server: s, pid: null, cpu: 0, memBytes: 0 }
            const pm = await getProcessMetrics(pid)
            return { server: s, pid: pm.running ? pid : null, cpu: pm.cpuPercent, memBytes: pm.memoryBytes }
          } catch {
            return { server: s, pid: null, cpu: 0, memBytes: 0 }
          }
        })
      )
      setProcs(procData)
    } catch {
      // silently fail metrics
    } finally {
      setRefreshing(false)
    }
  }

  useEffect(() => {
    refresh()
    const id = setInterval(refresh, 5000)
    return () => clearInterval(id)
  }, [servers.length])

  const runningCount = servers.filter(s => s.status === 'running').length

  return (
    <div className="p-6 space-y-4 h-full overflow-auto">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-slate-100 flex items-center gap-2">
            <Activity size={16} className="text-ark-400" /> Monitoramento
          </h1>
          <p className="text-xs text-slate-500 mt-0.5">Recursos do sistema e processos em tempo real</p>
        </div>
        <button
          onClick={refresh}
          className={`p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-surface-700 transition-colors ${refreshing ? 'animate-spin' : ''}`}
        >
          <RefreshCw size={14} />
        </button>
      </div>

      {/* Métricas do sistema */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <div className="flex items-center gap-2 mb-3">
            <Cpu size={14} className="text-ark-400" />
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wide">CPU</span>
          </div>
          {metrics ? (
            <>
              <p className="text-2xl font-bold text-slate-100 mb-2">{metrics.cpuPercent.toFixed(1)}%</p>
              <div className="h-2 bg-surface-700 rounded-full overflow-hidden">
                <div
                  className={`h-full rounded-full transition-all duration-500 ${metrics.cpuPercent > 85 ? 'bg-red-500' : metrics.cpuPercent > 65 ? 'bg-yellow-500' : 'bg-ark-500'}`}
                  style={{ width: `${Math.min(100, metrics.cpuPercent)}%` }}
                />
              </div>
            </>
          ) : (
            <div className="h-8 bg-surface-700 rounded animate-pulse" />
          )}
        </Card>

        <Card>
          <div className="flex items-center gap-2 mb-3">
            <MemoryStick size={14} className="text-purple-400" />
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wide">RAM</span>
          </div>
          {metrics ? (
            <MetricBar
              label={`${metrics.memoryPercent.toFixed(1)}%`}
              value={metrics.usedMemoryBytes}
              max={metrics.totalMemoryBytes}
            />
          ) : (
            <div className="h-8 bg-surface-700 rounded animate-pulse" />
          )}
        </Card>

        <Card>
          <div className="flex items-center gap-2 mb-3">
            <Server size={14} className="text-emerald-400" />
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wide">Servidores</span>
          </div>
          <p className="text-2xl font-bold text-slate-100">{runningCount}<span className="text-sm text-slate-500 font-normal"> / {servers.length}</span></p>
          <p className="text-xs text-slate-500 mt-1">rodando agora</p>
        </Card>
      </div>

      {/* Processos por servidor */}
      {servers.length > 0 && (
        <Card noPad>
          <div className="px-4 py-3 border-b border-surface-700">
            <p className="text-sm font-semibold text-slate-100">Processos</p>
          </div>
          <div className="divide-y divide-surface-700/50">
            {procs.map(({ server, pid, cpu, memBytes }) => (
              <div key={server.id} className="px-4 py-3 flex items-center gap-4">
                <div className="w-8 h-8 rounded-lg bg-surface-700 flex items-center justify-center shrink-0">
                  <Server size={14} className="text-slate-400" />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-slate-200 truncate">{server.name}</p>
                  <p className="text-xs text-slate-500">PID {pid ?? '—'} · {server.map}</p>
                </div>
                <Badge variant={server.status === 'running' ? 'success' : 'default'}>
                  {server.status}
                </Badge>
                {pid && (
                  <>
                    <div className="text-right w-20">
                      <p className="text-xs text-slate-400">CPU</p>
                      <p className="text-sm text-slate-200">{cpu.toFixed(1)}%</p>
                    </div>
                    <div className="text-right w-24">
                      <p className="text-xs text-slate-400">RAM</p>
                      <p className="text-sm text-slate-200">{formatBytes(memBytes)}</p>
                    </div>
                  </>
                )}
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  )
}
