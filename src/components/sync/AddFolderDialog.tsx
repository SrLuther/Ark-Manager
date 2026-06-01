import { useState } from 'react'
import { FolderOpen } from 'lucide-react'
import { Button, Input, Modal } from '../ui'
import type { SyncAgent } from '../../types'

export interface AddFolderDialogProps {
  open: boolean
  onClose: () => void
  agents: SyncAgent[]
  onAdd: (name: string, localPath: string, agentId?: number) => Promise<void>
}

export function AddFolderDialog({ open, onClose, agents, onAdd }: AddFolderDialogProps) {
  const [name, setName] = useState('')
  const [localPath, setLocalPath] = useState('')
  const [agentId, setAgentId] = useState<string>('')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async () => {
    if (!name.trim() || !localPath.trim()) return
    setSaving(true)
    setError(null)
    try {
      await onAdd(name.trim(), localPath.trim(), agentId ? Number(agentId) : undefined)
      setName('')
      setLocalPath('')
      setAgentId('')
      onClose()
    } catch (e) {
      setError(String(e))
    } finally {
      setSaving(false)
    }
  }

  const handleClose = () => {
    if (saving) return
    setName('')
    setLocalPath('')
    setAgentId('')
    setError(null)
    onClose()
  }

  const pairedAgents = agents.filter(a => a.paired)

  return (
    <Modal open={open} onClose={handleClose} title="Adicionar pasta sincronizada" size="md">
      <div className="p-5 flex flex-col gap-4">
        <Input
          label="Nome da pasta"
          value={name}
          onChange={e => setName(e.target.value)}
          placeholder="Ex: SaveGames Servidor 1"
        />

        <div className="flex flex-col gap-1.5">
          <label className="text-xs font-medium text-slate-300">Caminho local</label>
          <div className="flex gap-2">
            <input
              type="text"
              value={localPath}
              onChange={e => setLocalPath(e.target.value)}
              placeholder="C:\ARK\Server1\ShooterGame\Saved"
              className="flex-1 bg-surface-900 border border-surface-600 rounded-lg px-3 py-2 text-sm text-slate-200 font-mono placeholder:text-slate-600 focus:outline-none focus:ring-1 focus:ring-ark-500"
            />
            <Button
              variant="secondary"
              size="sm"
              title="Selecionar pasta"
              onClick={() => {}}
            >
              <FolderOpen size={14} />
            </Button>
          </div>
          <p className="text-xs text-slate-500">Caminho absoluto da pasta local a sincronizar</p>
        </div>

        {pairedAgents.length > 0 && (
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-slate-300">Peer remoto (opcional)</label>
            <select
              value={agentId}
              onChange={e => setAgentId(e.target.value)}
              className="bg-surface-900 border border-surface-600 rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:ring-1 focus:ring-ark-500"
            >
              <option value="">Sem sincronização remota</option>
              {pairedAgents.map(a => (
                <option key={a.id} value={String(a.id)}>
                  {a.name} — {a.address}:{a.port}
                </option>
              ))}
            </select>
            <p className="text-xs text-slate-500">Selecione um peer para habilitar sincronização bidirecional</p>
          </div>
        )}

        {error && <p className="text-xs text-red-400">{error}</p>}

        <div className="flex justify-end gap-2 pt-1">
          <Button variant="secondary" size="sm" onClick={handleClose}>Cancelar</Button>
          <Button
            size="sm"
            loading={saving}
            disabled={!name.trim() || !localPath.trim()}
            onClick={handleSubmit}
          >
            Adicionar
          </Button>
        </div>
      </div>
    </Modal>
  )
}
