import { cn } from '../../utils/helpers'
import type { EventRate } from '../../types'

const RATE_FIELDS: { key: keyof EventRate; label: string }[] = [
  { key: 'xpMultiplier',        label: 'XP' },
  { key: 'harvestMultiplier',   label: 'Coleta' },
  { key: 'tameSpeedMultiplier', label: 'Domesticação' },
  { key: 'breedingMultiplier',  label: 'Breeding' },
  { key: 'hatchSpeedMultiplier',label: 'Eclosão de ovos' },
  { key: 'matureSpeedMultiplier',label: 'Maturação' },
]

const DEFAULT_RATES: EventRate = {
  xpMultiplier:         1.0,
  harvestMultiplier:    1.0,
  tameSpeedMultiplier:  1.0,
  breedingMultiplier:   1.0,
  hatchSpeedMultiplier: 1.0,
  matureSpeedMultiplier:1.0,
}

export interface EventRatesFormProps {
  value?: EventRate
  onChange: (rates: EventRate) => void
  className?: string
}

export function EventRatesForm({ value = DEFAULT_RATES, onChange, className }: EventRatesFormProps) {
  const handleChange = (key: keyof EventRate, raw: string) => {
    const num = parseFloat(raw)
    if (isNaN(num) || num < 0) return
    onChange({ ...value, [key]: num })
  }

  return (
    <div className={cn('grid grid-cols-1 sm:grid-cols-2 gap-3', className)}>
      {RATE_FIELDS.map(({ key, label }) => {
        const val = value[key]
        const isModified = Math.abs(val - 1.0) > 0.001

        return (
          <div
            key={key}
            className={cn(
              'flex items-center gap-3 rounded-lg px-3 py-2.5 border',
              isModified
                ? 'border-ark-700 bg-surface-800'
                : 'border-surface-700 bg-surface-800'
            )}
          >
            <span className="text-xs text-slate-300 flex-1">{label}</span>
            <input
              type="number"
              step="0.5"
              min="0.1"
              value={val}
              onChange={(e) => handleChange(key, e.target.value)}
              className={cn(
                'w-20 bg-surface-900 border border-surface-600 rounded px-2 py-1',
                'text-xs text-right font-mono text-slate-200',
                'focus:outline-none focus:ring-1 focus:ring-ark-500'
              )}
            />
            <span className="text-xs text-slate-500 w-2">×</span>
          </div>
        )
      })}
    </div>
  )
}
