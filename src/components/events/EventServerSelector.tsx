import { Server } from 'lucide-react'
import { cn, mapLabel, statusLabel } from '../../utils/helpers'
import type { Server as ServerType } from '../../types'

export interface EventServerSelectorProps {
  servers: ServerType[]
  selected: number[]
  onChange: (ids: number[]) => void
  className?: string
}

export function EventServerSelector({
  servers,
  selected,
  onChange,
  className,
}: EventServerSelectorProps) {
  const toggle = (id: number) => {
    if (selected.includes(id)) {
      onChange(selected.filter((s) => s !== id))
    } else {
      onChange([...selected, id])
    }
  }

  const allSelected = servers.length > 0 && selected.length === servers.length
  const toggleAll = () => {
    if (allSelected) {
      onChange([])
    } else {
      onChange(servers.map((s) => s.id))
    }
  }

  return (
    <div className={cn('flex flex-col gap-2', className)}>
      {/* Selecionar todos */}
      {servers.length > 1 && (
        <label className="flex items-center gap-2 px-3 py-2 rounded-lg bg-surface-800 border border-surface-600 cursor-pointer hover:bg-surface-700 transition-colors">
          <input
            type="checkbox"
            checked={allSelected}
            onChange={toggleAll}
            className="accent-ark-500"
          />
          <span className="text-xs text-slate-300 font-medium">
            {allSelected ? 'Desmarcar todos' : 'Selecionar todos'}
          </span>
        </label>
      )}

      {servers.length === 0 && (
        <p className="text-xs text-slate-600 text-center py-4">
          Nenhum servidor cadastrado
        </p>
      )}

      {servers.map((srv) => {
        const checked = selected.includes(srv.id)
        return (
          <label
            key={srv.id}
            className={cn(
              'flex items-center gap-3 px-3 py-2.5 rounded-lg border cursor-pointer transition-colors',
              checked
                ? 'border-ark-700 bg-ark-900/20'
                : 'border-surface-700 bg-surface-800 hover:bg-surface-700'
            )}
          >
            <input
              type="checkbox"
              checked={checked}
              onChange={() => toggle(srv.id)}
              className="accent-ark-500 shrink-0"
            />
            <Server size={13} className={cn('shrink-0', checked ? 'text-ark-400' : 'text-slate-500')} />
            <div className="flex flex-col min-w-0 flex-1">
              <span className={cn('text-xs font-medium truncate', checked ? 'text-slate-100' : 'text-slate-300')}>
                {srv.name}
              </span>
              <span className="text-[10px] text-slate-500">
                {mapLabel(srv.map)} · {statusLabel(srv.status)}
              </span>
            </div>
          </label>
        )
      })}

      {selected.length > 0 && (
        <p className="text-xs text-slate-500 text-right">
          {selected.length} servidor{selected.length > 1 ? 'es' : ''} selecionado{selected.length > 1 ? 's' : ''}
        </p>
      )}
    </div>
  )
}
