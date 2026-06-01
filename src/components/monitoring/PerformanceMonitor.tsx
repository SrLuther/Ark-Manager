import { useEffect, useRef, useState } from 'react'
import { Cpu, MemoryStick } from 'lucide-react'
import { getSystemMetrics } from '../../utils/tauri'
import { formatBytes, cn } from '../../utils/helpers'
import type { SystemMetrics } from '../../types'

const MAX_POINTS = 30

interface SparklineProps {
  data: number[]
  color: string
  height?: number
}

function Sparkline({ data, color, height = 40 }: SparklineProps) {
  if (data.length < 2) return <div style={{ height }} />
  const w = 200
  const h = height
  const max = 100
  const step = w / (MAX_POINTS - 1)

  const points = Array.from({ length: MAX_POINTS }, (_, i) => {
    const val = data[i] ?? 0
    const x = i * step
    const y = h - (val / max) * h
    return `${x},${y}`
  }).join(' ')

  return (
    <svg
      viewBox={`0 0 ${w} ${h}`}
      preserveAspectRatio="none"
      className="w-full"
      style={{ height }}
    >
      <polyline
        points={points}
        fill="none"
        stroke={color}
        strokeWidth="1.5"
        strokeLinejoin="round"
        strokeLinecap="round"
        opacity="0.8"
      />
      {/* Área preenchida */}
      <polyline
        points={`0,${h} ${points} ${w},${h}`}
        fill={color}
        opacity="0.1"
        strokeWidth="0"
      />
    </svg>
  )
}

interface MetricCardProps {
  label: string
  icon: React.ReactNode
  value: number
  unit: string
  history: number[]
  color: string
  detail?: string
}

function MetricCard({ label, icon, value, unit, history, color, detail }: MetricCardProps) {
  return (
    <div className="rounded-xl bg-surface-800 border border-surface-700 p-4 flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-slate-400 text-xs">
          {icon}
          {label}
        </div>
        <div className="flex items-baseline gap-1">
          <span className="text-lg font-semibold text-slate-100">{value.toFixed(1)}</span>
          <span className="text-xs text-slate-500">{unit}</span>
        </div>
      </div>
      {detail && <div className="text-xs text-slate-500">{detail}</div>}
      {/* Barra de uso */}
      <div className="h-1.5 w-full bg-surface-700 rounded-full overflow-hidden">
        <div
          className="h-full rounded-full transition-all duration-500"
          style={{ width: `${Math.min(value, 100)}%`, backgroundColor: color }}
        />
      </div>
      <Sparkline data={history} color={color} height={36} />
    </div>
  )
}

export interface PerformanceMonitorProps {
  /** Intervalo de polling em milissegundos (padrão: 3000) */
  interval?: number
  className?: string
}

export function PerformanceMonitor({ interval = 3000, className }: PerformanceMonitorProps) {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  const cpuHistory   = useRef<number[]>(Array(MAX_POINTS).fill(0))
  const ramHistory   = useRef<number[]>(Array(MAX_POINTS).fill(0))
  const [, forceRender] = useState(0)

  useEffect(() => {
    let active = true

    const poll = async () => {
      try {
        const m = await getSystemMetrics()
        if (!active) return
        setMetrics(m)
        cpuHistory.current = [...cpuHistory.current.slice(1), m.cpuPercent]
        ramHistory.current = [...ramHistory.current.slice(1), m.memoryPercent]
        forceRender((n) => n + 1)
      } catch {
        // silencioso
      }
    }

    poll()
    const id = setInterval(poll, interval)
    return () => { active = false; clearInterval(id) }
  }, [interval])

  if (!metrics) {
    return (
      <div className={cn('grid grid-cols-2 gap-4', className)}>
        {[0, 1].map((i) => (
          <div key={i} className="h-32 rounded-xl bg-surface-800 border border-surface-700 animate-pulse" />
        ))}
      </div>
    )
  }

  const ramUsedGB  = metrics.usedMemoryBytes / 1073741824
  const ramTotalGB = metrics.totalMemoryBytes / 1073741824

  return (
    <div className={cn('grid grid-cols-1 sm:grid-cols-2 gap-4', className)}>
      <MetricCard
        label="CPU"
        icon={<Cpu size={13} />}
        value={metrics.cpuPercent}
        unit="%"
        history={cpuHistory.current}
        color="#38bdf8"
        detail="Uso total do processador"
      />
      <MetricCard
        label="RAM"
        icon={<MemoryStick size={13} />}
        value={metrics.memoryPercent}
        unit="%"
        history={ramHistory.current}
        color="#a78bfa"
        detail={`${formatBytes(metrics.usedMemoryBytes)} / ${formatBytes(metrics.totalMemoryBytes)} — ${ramUsedGB.toFixed(1)} GB / ${ramTotalGB.toFixed(1)} GB`}
      />
    </div>
  )
}
