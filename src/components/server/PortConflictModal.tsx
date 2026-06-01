import { useState, useEffect } from 'react'
import { AlertTriangle, RefreshCw } from 'lucide-react'
import { suggestAvailablePort } from '../../utils/tauri'
import { Button, Modal, Input } from '../ui'

export interface PortConflictModalProps {
  open: boolean
  onClose: () => void
  conflicts: number[]               // portas conflitantes detectadas
  gamePort: number
  queryPort: number
  rconPort: number
  onResolve: (gamePort: number, queryPort: number, rconPort: number) => void
}

export function PortConflictModal({
  open,
  onClose,
  conflicts,
  gamePort: initialGame,
  queryPort: initialQuery,
  rconPort: initialRcon,
  onResolve,
}: PortConflictModalProps) {
  const [game, setGame]   = useState(initialGame)
  const [query, setQuery] = useState(initialQuery)
  const [rcon, setRcon]   = useState(initialRcon)
  const [suggesting, setSuggesting] = useState(false)

  useEffect(() => {
    setGame(initialGame)
    setQuery(initialQuery)
    setRcon(initialRcon)
  }, [initialGame, initialQuery, initialRcon, open])

  const suggestAll = async () => {
    setSuggesting(true)
    try {
      const [g, q, r] = await Promise.all([
        suggestAvailablePort(game),
        suggestAvailablePort(query),
        suggestAvailablePort(rcon),
      ])
      setGame(g)
      setQuery(q)
      setRcon(r)
    } catch {
      // silencioso — usuário pode ajustar manualmente
    } finally {
      setSuggesting(false)
    }
  }

  const isConflict = (port: number) => conflicts.includes(port)

  return (
    <Modal open={open} onClose={onClose} title="Conflito de portas detectado" size="md">
      <div className="p-5 flex flex-col gap-5">
        {/* Aviso */}
        <div className="flex items-start gap-3 rounded-lg bg-yellow-900/30 border border-yellow-800 p-3">
          <AlertTriangle size={16} className="text-yellow-400 shrink-0 mt-0.5" />
          <p className="text-xs text-yellow-300">
            As seguintes portas já estão em uso nesta máquina:{' '}
            <span className="font-mono font-semibold">{conflicts.join(', ')}</span>.
            Ajuste os valores ou use a sugestão automática.
          </p>
        </div>

        {/* Campos */}
        <div className="grid grid-cols-3 gap-3">
          <Input
            label="Porta de jogo"
            type="number"
            value={String(game)}
            onChange={(e) => setGame(Number(e.target.value))}
            className={isConflict(game) ? 'border-yellow-600 focus:ring-yellow-500' : ''}
            hint={isConflict(game) ? 'Em conflito' : undefined}
          />
          <Input
            label="Porta query"
            type="number"
            value={String(query)}
            onChange={(e) => setQuery(Number(e.target.value))}
            className={isConflict(query) ? 'border-yellow-600 focus:ring-yellow-500' : ''}
            hint={isConflict(query) ? 'Em conflito' : undefined}
          />
          <Input
            label="Porta RCON"
            type="number"
            value={String(rcon)}
            onChange={(e) => setRcon(Number(e.target.value))}
            className={isConflict(rcon) ? 'border-yellow-600 focus:ring-yellow-500' : ''}
            hint={isConflict(rcon) ? 'Em conflito' : undefined}
          />
        </div>

        {/* Ações */}
        <div className="flex items-center justify-between pt-1">
          <Button
            variant="secondary"
            size="sm"
            loading={suggesting}
            onClick={suggestAll}
          >
            <RefreshCw size={12} />
            Sugerir portas disponíveis
          </Button>
          <div className="flex gap-2">
            <Button variant="secondary" onClick={onClose}>
              Cancelar
            </Button>
            <Button onClick={() => { onResolve(game, query, rcon); onClose() }}>
              Aplicar
            </Button>
          </div>
        </div>
      </div>
    </Modal>
  )
}
