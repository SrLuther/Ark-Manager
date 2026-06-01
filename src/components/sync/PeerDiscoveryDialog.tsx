import { useState } from 'react'
import { Wifi, WifiOff, Link2, Loader2, RefreshCw } from 'lucide-react'
import { cn, formatRelative } from '../../utils/helpers'
import { Button, Input, Modal } from '../ui'
import type { SyncAgent } from '../../types'

export interface PeerDiscoveryDialogProps {
  open: boolean
  onClose: () => void
  /** Agentes descobertos na rede (fornecidos pelo backend após Fase 8) */
  discoveredAgents?: SyncAgent[]
  /** Disparado quando o usuário clica em "Parear" em um agente */
  onPair: (agent: SyncAgent, pairingCode: string) => Promise<void>
  /** Disparado para redicovery manual */
  onRefresh?: () => void
  refreshing?: boolean
}

export function PeerDiscoveryDialog({
  open,
  onClose,
  discoveredAgents = [],
  onPair,
  onRefresh,
  refreshing = false,
}: PeerDiscoveryDialogProps) {
  const [selected, setSelected] = useState<SyncAgent | null>(null)
  const [code, setCode] = useState('')
  const [pairing, setPairing] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handlePair = async () => {
    if (!selected || code.length !== 6) return
    setPairing(true)
    setError(null)
    try {
      await onPair(selected, code)
      setSelected(null)
      setCode('')
      onClose()
    } catch (e) {
      setError(String(e))
    } finally {
      setPairing(false)
    }
  }

  const handleClose = () => {
    if (pairing) return
    setSelected(null)
    setCode('')
    setError(null)
    onClose()
  }

  return (
    <Modal open={open} onClose={handleClose} title="Descoberta de peers" size="lg">
      <div className="p-5 flex flex-col gap-5">

        {/* Header de ação */}
        <div className="flex items-center justify-between">
          <p className="text-xs text-slate-400">
            Agentes ARK Manager detectados na rede local via UDP broadcast.
          </p>
          {onRefresh && (
            <Button
              size="sm"
              variant="secondary"
              loading={refreshing}
              onClick={onRefresh}
            >
              <RefreshCw size={12} />
              Atualizar
            </Button>
          )}
        </div>

        {/* Lista de agentes */}
        <div className="rounded-xl border border-surface-700 divide-y divide-surface-700 min-h-[80px]">
          {discoveredAgents.length === 0 && (
            <div className="flex flex-col items-center gap-2 py-8 text-slate-600">
              {refreshing
                ? <Loader2 size={20} className="animate-spin" />
                : <WifiOff size={20} />
              }
              <span className="text-xs">
                {refreshing ? 'Buscando agentes…' : 'Nenhum agente encontrado na rede'}
              </span>
            </div>
          )}
          {discoveredAgents.map((agent) => (
            <button
              key={agent.id}
              type="button"
              onClick={() => setSelected(agent.id === selected?.id ? null : agent)}
              className={cn(
                'w-full flex items-center justify-between px-4 py-3 gap-3',
                'text-left transition-colors',
                selected?.id === agent.id
                  ? 'bg-ark-900/30'
                  : 'hover:bg-surface-700/50'
              )}
            >
              <div className="flex items-center gap-3 min-w-0">
                {agent.status === 'online'
                  ? <Wifi size={14} className="text-emerald-400 shrink-0" />
                  : <WifiOff size={14} className="text-slate-500 shrink-0" />
                }
                <div className="flex flex-col min-w-0">
                  <span className="text-sm text-slate-200">{agent.name}</span>
                  <span className="text-xs text-slate-500 font-mono">
                    {agent.address}:{agent.port}
                  </span>
                </div>
              </div>
              <div className="flex flex-col items-end shrink-0 text-xs text-slate-500">
                <span className={cn(
                  agent.status === 'online' ? 'text-emerald-400' : 'text-slate-500'
                )}>
                  {agent.status === 'online' ? 'Online' : 'Offline'}
                </span>
                {agent.lastSeenAt && (
                  <span>{formatRelative(agent.lastSeenAt)}</span>
                )}
              </div>
            </button>
          ))}
        </div>

        {/* Painel de pareamento */}
        {selected && (
          <div className="rounded-xl bg-surface-900 border border-ark-800/50 p-4 flex flex-col gap-3">
            <div className="flex items-center gap-2 text-sm text-slate-200">
              <Link2 size={14} className="text-ark-400" />
              Parear com <strong>{selected.name}</strong>
            </div>
            <p className="text-xs text-slate-400">
              Insira o código de 6 dígitos exibido no ARK Manager do peer remoto.
            </p>
            <Input
              label="Código de pareamento"
              value={code}
              onChange={(e) => setCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
              placeholder="000000"
              maxLength={6}
              className="font-mono tracking-widest text-center text-lg"
            />
            {error && (
              <p className="text-xs text-red-400">{error}</p>
            )}
            <div className="flex justify-end gap-2">
              <Button variant="secondary" size="sm" onClick={() => setSelected(null)}>
                Cancelar
              </Button>
              <Button
                size="sm"
                loading={pairing}
                disabled={code.length !== 6}
                onClick={handlePair}
              >
                <Link2 size={12} />
                Parear
              </Button>
            </div>
          </div>
        )}
      </div>
    </Modal>
  )
}
