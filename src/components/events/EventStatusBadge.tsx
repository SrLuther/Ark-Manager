import { cn } from '../../utils/helpers'
import type { EventStatus } from '../../types'

const STATUS_CONFIG: Record<EventStatus, { label: string; classes: string }> = {
  scheduled: { label: 'Agendado',  classes: 'bg-ark-900/50 text-ark-400 border border-ark-800' },
  active:    { label: 'Ativo',     classes: 'bg-emerald-900/50 text-emerald-400 border border-emerald-800' },
  ended:     { label: 'Encerrado', classes: 'bg-surface-700 text-slate-400' },
  cancelled: { label: 'Cancelado', classes: 'bg-surface-700 text-slate-500' },
  error:     { label: 'Erro',      classes: 'bg-red-900/50 text-red-400 border border-red-800' },
}

export interface EventStatusBadgeProps {
  status: EventStatus
  className?: string
}

export function EventStatusBadge({ status, className }: EventStatusBadgeProps) {
  const cfg = STATUS_CONFIG[status]
  return (
    <span
      className={cn(
        'inline-flex items-center px-2 py-0.5 rounded text-xs font-medium',
        cfg.classes,
        className
      )}
    >
      {cfg.label}
    </span>
  )
}
