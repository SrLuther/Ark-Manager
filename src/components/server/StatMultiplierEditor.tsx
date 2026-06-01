import { cn } from '../../utils/helpers'

const STAT_LABELS: Record<number, string> = {
  0:  'Vida (Health)',
  1:  'Estamina (Stamina)',
  2:  'Torpor',
  3:  'Oxigênio (Oxygen)',
  4:  'Comida (Food)',
  5:  'Água (Water)',
  6:  'Temperatura',
  7:  'Peso (Weight)',
  8:  'Dano corpo-a-corpo',
  9:  'Velocidade de movimento',
  10: 'Resistência (Fortitude)',
  11: 'Habilidade de crafting',
}

const DEFAULT_VALUES = Array(12).fill(1.0) as number[]

export interface StatMultiplierEditorProps {
  value?: number[]
  onChange: (values: number[]) => void
  /** Índices desabilitados (ex: torpor = 2) */
  disabledIndices?: number[]
  className?: string
}

export function StatMultiplierEditor({
  value = DEFAULT_VALUES,
  onChange,
  disabledIndices = [2],
  className,
}: StatMultiplierEditorProps) {
  const vals = value.length === 12 ? value : DEFAULT_VALUES

  const handleChange = (index: number, raw: string) => {
    const num = parseFloat(raw)
    if (isNaN(num) || num < 0) return
    const next = [...vals]
    next[index] = num
    onChange(next)
  }

  const reset = (index: number) => {
    const next = [...vals]
    next[index] = 1.0
    onChange(next)
  }

  return (
    <div className={cn('grid grid-cols-1 sm:grid-cols-2 gap-3', className)}>
      {Array.from({ length: 12 }, (_, i) => {
        const disabled = disabledIndices.includes(i)
        const isModified = Math.abs(vals[i] - 1.0) > 0.001

        return (
          <div
            key={i}
            className={cn(
              'flex items-center gap-3 rounded-lg px-3 py-2 border',
              disabled
                ? 'border-surface-700 bg-surface-900/50 opacity-50 cursor-not-allowed'
                : isModified
                ? 'border-ark-700 bg-surface-800'
                : 'border-surface-700 bg-surface-800'
            )}
          >
            {/* Índice */}
            <span className="text-xs font-mono text-slate-500 w-4 shrink-0">{i}</span>

            {/* Label */}
            <span className="text-xs text-slate-300 flex-1 truncate">{STAT_LABELS[i]}</span>

            {/* Input */}
            <input
              type="number"
              step="0.1"
              min="0"
              disabled={disabled}
              value={vals[i]}
              onChange={(e) => handleChange(i, e.target.value)}
              className={cn(
                'w-20 bg-surface-900 border border-surface-600 rounded px-2 py-1',
                'text-xs text-right font-mono text-slate-200',
                'focus:outline-none focus:ring-1 focus:ring-ark-500',
                'disabled:opacity-50 disabled:cursor-not-allowed'
              )}
            />

            {/* Reset */}
            {isModified && !disabled && (
              <button
                type="button"
                onClick={() => reset(i)}
                className="text-xs text-slate-500 hover:text-slate-300 transition-colors shrink-0"
                title="Restaurar padrão (1.0)"
              >
                ↺
              </button>
            )}
          </div>
        )
      })}
    </div>
  )
}
