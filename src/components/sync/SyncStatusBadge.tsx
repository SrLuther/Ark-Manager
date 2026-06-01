import { cn } from '../../utils/helpers'
import type { SyncStatus } from '../../types'

const STATUS_CONFIG: Record<SyncStatus, { label: string; classes: string }> = {
  synced:   { label: 'Sincronizado',  classes: 'bg-emerald-900/50 text-emerald-400 border border-emerald-800' },
  syncing:  { label: 'Sincronizando', classes: 'bg-ark-900/50 text-ark-400 border border-ark-800' },
  pending:  { label: 'Pendente',      classes: 'bg-yellow-900/50 text-yellow-400 border border-yellow-800' },
  conflict: { label: 'Conflito',      classes: 'bg-orange-900/50 text-orange-400 border border-orange-800' },
  offline:  { label: 'Offline',       classes: 'bg-surface-700 text-slate-400' },
  error:    { label: 'Erro',          classes: 'bg-red-900/50 text-red-400 border border-red-800' },
}

export interface SyncStatusBadgeProps {
  status: SyncStatus
  className?: string
}

export function SyncStatusBadge({ status, className }: SyncStatusBadgeProps) {
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
