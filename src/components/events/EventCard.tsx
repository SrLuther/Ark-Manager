import { useState, useEffect } from 'react'
import { Calendar, Clock, Edit2, XCircle, Users } from 'lucide-react'
import { formatDate, cn } from '../../utils/helpers'
import { EventStatusBadge } from './EventStatusBadge'
import { Button } from '../ui'
import type { SeasonalEvent } from '../../types'

function useCountdown(targetIso: string | null) {
  const [diff, setDiff] = useState<number>(0)

  useEffect(() => {
    if (!targetIso) return
    const update = () => {
      setDiff(new Date(targetIso).getTime() - Date.now())
    }
    update()
    const id = setInterval(update, 1000)
    return () => clearInterval(id)
  }, [targetIso])

  if (diff <= 0) return null

  const totalSec = Math.floor(diff / 1000)
  const d = Math.floor(totalSec / 86400)
  const h = Math.floor((totalSec % 86400) / 3600)
  const m = Math.floor((totalSec % 3600) / 60)
  const s = totalSec % 60

  const parts: string[] = []
  if (d > 0) parts.push(`${d}d`)
  if (h > 0 || d > 0) parts.push(`${String(h).padStart(2, '0')}h`)
  parts.push(`${String(m).padStart(2, '0')}m`)
  parts.push(`${String(s).padStart(2, '0')}s`)
  return parts.join(' ')
}

export interface EventCardProps {
  event: SeasonalEvent
  onEdit?: () => void
  onCancel?: () => void
}

export function EventCard({ event, onEdit, onCancel }: EventCardProps) {
  const active    = event.status === 'active'
  const scheduled = event.status === 'scheduled'
  const ended     = event.status === 'ended' || event.status === 'cancelled'

  const countdown = useCountdown(
    active ? event.endAt : scheduled ? event.startAt : null
  )

  const rateItems = [
    { label: 'XP',     value: event.rates.xpMultiplier },
    { label: 'Coleta', value: event.rates.harvestMultiplier },
    { label: 'Tame',   value: event.rates.tameSpeedMultiplier },
    { label: 'Breed',  value: event.rates.breedingMultiplier },
  ]

  return (
    <div
      className={cn(
        'rounded-xl border p-4 flex flex-col gap-4',
        active
          ? 'bg-emerald-900/10 border-emerald-800'
          : ended
          ? 'bg-surface-900 border-surface-700 opacity-60'
          : 'bg-surface-800 border-surface-700'
      )}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-2">
        <div>
          <h3 className="text-sm font-semibold text-slate-100">{event.name}</h3>
          {event.description && (
            <p className="text-xs text-slate-500 mt-0.5">{event.description}</p>
          )}
        </div>
        <EventStatusBadge status={event.status} />
      </div>

      {/* Countdown */}
      {countdown && (
        <div className={cn(
          'flex items-center gap-2 rounded-lg px-3 py-2',
          active ? 'bg-emerald-900/30 text-emerald-300' : 'bg-ark-900/20 text-ark-300'
        )}>
          <Clock size={13} className="shrink-0" />
          <span className="text-xs">
            {active ? 'Encerra em' : 'Inicia em'}:
          </span>
          <span className="font-mono text-sm font-semibold">{countdown}</span>
        </div>
      )}

      {/* Datas */}
      <div className="grid grid-cols-2 gap-2 text-xs text-slate-400">
        <div className="flex items-center gap-1.5">
          <Calendar size={11} className="shrink-0" />
          <span>Início: {formatDate(event.startAt)}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <Calendar size={11} className="shrink-0" />
          <span>Fim: {formatDate(event.endAt)}</span>
        </div>
      </div>

      {/* Taxas resumidas */}
      <div className="grid grid-cols-4 gap-2">
        {rateItems.map(({ label, value }) => (
          <div
            key={label}
            className={cn(
              'flex flex-col items-center rounded-lg py-2 px-1',
              'bg-surface-900 border border-surface-700'
            )}
          >
            <span className="text-[10px] text-slate-500">{label}</span>
            <span className={cn(
              'text-sm font-semibold font-mono',
              value > 1 ? 'text-ark-400' : 'text-slate-300'
            )}>
              {value}×
            </span>
          </div>
        ))}
      </div>

      {/* Servers + actions */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5 text-xs text-slate-500">
          <Users size={11} />
          {event.serverIds.length} servidor{event.serverIds.length !== 1 ? 'es' : ''}
        </div>
        {!ended && (
          <div className="flex gap-1.5">
            {onEdit && (
              <Button size="sm" variant="ghost" onClick={onEdit}>
                <Edit2 size={11} /> Editar
              </Button>
            )}
            {onCancel && (
              <Button size="sm" variant="ghost" onClick={onCancel}
                className="text-red-400 hover:text-red-300 hover:bg-red-950/30">
                <XCircle size={11} /> Cancelar
              </Button>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
