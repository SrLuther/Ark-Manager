import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  Plus, Trash2, Settings2,
  Server, FolderOpen, RefreshCw,
} from 'lucide-react'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useServerStore } from '../stores/serverStore'
import { createServer, updateServer, deleteServer, detectPortConflicts } from '../utils/tauri'
import { statusLabel, mapLabel } from '../utils/helpers'
import { Button, Badge, Card, Modal, Input } from '../components/ui'
import type { ArkMap, CreateServerRequest, UpdateServerRequest, Server as ServerType } from '../types'
import toast from 'react-hot-toast'

const ARK_MAPS: ArkMap[] = [
  'TheIsland','ScorchedEarth','Aberration','Extinction',
  'Genesis','Genesis2','CrystalIsles','Ragnarok','Valguero','LostIsland','Fjordur',
]

const EMPTY_FORM: CreateServerRequest = {
  name: '', map: 'TheIsland', installDir: '', steamcmdDir: '',
  gamePort: 7777, queryPort: 27015, rconPort: 32330,
  rconPassword: '', adminPassword: '', serverPassword: '', maxPlayers: 70,
}

function statusBadgeVariant(s: string) {
  if (s === 'running') return 'success' as const
  if (s === 'error') return 'error' as const
  if (s === 'starting' || s === 'stopping') return 'warning' as const
  return 'default' as const
}

interface ServerFormProps {
  initial?: Partial<CreateServerRequest>
  onSave: (data: CreateServerRequest) => Promise<void>
  onCancel: () => void
  title: string
  loading: boolean
}

function ServerForm({ initial, onSave, onCancel, title, loading }: ServerFormProps) {
  const [form, setForm] = useState<CreateServerRequest>({ ...EMPTY_FORM, ...initial })
  const [portConflicts, setPortConflicts] = useState<number[]>([])

  const set = (k: keyof CreateServerRequest, v: unknown) =>
    setForm(f => ({ ...f, [k]: v }))

  const pickDir = async (field: 'installDir' | 'steamcmdDir') => {
    const selected = await openDialog({ directory: true, multiple: false })
    if (typeof selected === 'string') set(field, selected)
  }

  const checkPorts = async () => {
    try {
      const conflicts = await detectPortConflicts(form.gamePort, form.queryPort, form.rconPort)
      setPortConflicts(conflicts as number[])
    } catch {}
  }

  return (
    <div className="space-y-3 max-h-[70vh] overflow-y-auto pr-1">
      <h3 className="text-sm font-semibold text-slate-100">{title}</h3>
      <Input label="Nome do servidor" value={form.name} onChange={e => set('name', e.target.value)} />
      <div className="flex flex-col gap-1.5">
        <label className="text-xs font-medium text-slate-300">Mapa</label>
        <select
          className="w-full rounded-lg bg-surface-800 border border-surface-600 px-3 py-2 text-sm text-slate-100 focus:outline-none focus:ring-2 focus:ring-ark-500"
          value={form.map}
          onChange={e => set('map', e.target.value as ArkMap)}
        >
          {ARK_MAPS.map(m => <option key={m} value={m}>{mapLabel(m)}</option>)}
        </select>
      </div>
      <div className="flex items-end gap-2">
        <div className="flex-1">
          <Input label="Diretório de instalação" value={form.installDir} onChange={e => set('installDir', e.target.value)} hint="Ex: C:\ARK\Server1" />
        </div>
        <button
          type="button"
          onClick={() => pickDir('installDir')}
          className="mb-0.5 p-2 rounded-lg bg-surface-700 hover:bg-surface-600 border border-surface-600 text-slate-300 hover:text-white transition-colors"
          title="Escolher pasta"
        >
          <FolderOpen size={15} />
        </button>
      </div>
      <div className="flex items-end gap-2">
        <div className="flex-1">
          <Input label="Diretório SteamCMD" value={form.steamcmdDir} onChange={e => set('steamcmdDir', e.target.value)} hint="Ex: C:\SteamCMD" />
        </div>
        <button
          type="button"
          onClick={() => pickDir('steamcmdDir')}
          className="mb-0.5 p-2 rounded-lg bg-surface-700 hover:bg-surface-600 border border-surface-600 text-slate-300 hover:text-white transition-colors"
          title="Escolher pasta"
        >
          <FolderOpen size={15} />
        </button>
      </div>
      <div className="grid grid-cols-3 gap-2">
        <Input label="Porta do jogo" type="number" value={form.gamePort} onChange={e => set('gamePort', +e.target.value)} />
        <Input label="Porta Query" type="number" value={form.queryPort} onChange={e => set('queryPort', +e.target.value)} />
        <Input label="Porta RCON" type="number" value={form.rconPort} onChange={e => set('rconPort', +e.target.value)} />
      </div>
      {portConflicts.length > 0 && (
        <p className="text-xs text-red-400">Portas em conflito: {portConflicts.join(', ')}</p>
      )}
      <Button size="sm" variant="ghost" type="button" onClick={checkPorts}>Verificar portas</Button>
      <Input label="Senha do admin" type="password" value={form.adminPassword} onChange={e => set('adminPassword', e.target.value)} />
      <Input label="Senha RCON" type="password" value={form.rconPassword} onChange={e => set('rconPassword', e.target.value)} />
      <Input label="Senha do servidor (opcional)" type="password" value={form.serverPassword ?? ''} onChange={e => set('serverPassword', e.target.value)} />
      <Input label="Máx. jogadores" type="number" value={form.maxPlayers} onChange={e => set('maxPlayers', +e.target.value)} />
      <div className="flex gap-2 pt-2">
        <Button variant="primary" loading={loading} onClick={() => onSave(form)}>Salvar</Button>
        <Button variant="ghost" onClick={onCancel}>Cancelar</Button>
      </div>
    </div>
  )
}

export default function ServerManager() {
  const { servers, fetchServers, loading } = useServerStore()
  const navigate = useNavigate()
  const [showAdd, setShowAdd] = useState(false)
  const [editServer, setEditServer] = useState<ServerType | null>(null)
  const [saving, setSaving] = useState(false)
  const [deleting, setDeleting] = useState<number | null>(null)

  useEffect(() => { fetchServers() }, [fetchServers])

  const handleCreate = async (data: CreateServerRequest) => {
    setSaving(true)
    try {
      await createServer(data)
      await fetchServers()
      setShowAdd(false)
      toast.success('Servidor criado')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSaving(false)
    }
  }

  const handleUpdate = async (data: CreateServerRequest) => {
    if (!editServer) return
    setSaving(true)
    try {
      await updateServer(editServer.id, data as UpdateServerRequest)
      await fetchServers()
      setEditServer(null)
      toast.success('Servidor atualizado')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSaving(false)
    }
  }

  const handleDelete = async (id: number, name: string) => {
    if (!confirm(`Excluir servidor "${name}"? Esta ação é irreversível.`)) return
    setDeleting(id)
    try {
      await deleteServer(id)
      await fetchServers()
      toast.success('Servidor excluído')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setDeleting(null)
    }
  }

  return (
    <div className="p-6 space-y-4 h-full overflow-auto">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-slate-100">Servidores</h1>
          <p className="text-xs text-slate-500 mt-0.5">{servers.length} servidor(es) configurado(s)</p>
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={() => fetchServers()} loading={loading}>
            <RefreshCw size={13} />
          </Button>
          <Button size="sm" onClick={() => setShowAdd(true)}>
            <Plus size={13} /> Adicionar servidor
          </Button>
        </div>
      </div>

      {servers.length === 0 && !loading ? (
        <Card className="text-center py-16">
          <Server size={36} className="text-slate-600 mx-auto mb-3" />
          <p className="text-slate-400 text-sm">Nenhum servidor configurado</p>
          <Button className="mt-4" size="sm" onClick={() => setShowAdd(true)}>
            <Plus size={13} /> Adicionar servidor
          </Button>
        </Card>
      ) : (
        <div className="space-y-2">
          {servers.map(server => (
            <Card key={server.id} className="flex items-center gap-4">
              <Server size={18} className="text-ark-400 shrink-0" />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-slate-100">{server.name}</span>
                  <Badge variant={statusBadgeVariant(server.status)}>
                    {statusLabel(server.status)}
                  </Badge>
                </div>
                <p className="text-xs text-slate-500 mt-0.5 truncate">
                  {mapLabel(server.map as any)} · Porta {server.gamePort} · Máx. {server.maxPlayers}
                </p>
                <p className="text-xs text-slate-600 truncate">{server.installDir}</p>
              </div>
              <div className="flex items-center gap-1.5 shrink-0 flex-wrap justify-end">
                <Button size="sm" variant="ghost" onClick={() => navigate(`/config/${server.id}`)}>
                  <Settings2 size={13} /> Config
                </Button>
                <Button size="sm" variant="ghost" onClick={() => navigate(`/rcon/${server.id}`)}>
                  RCON
                </Button>
                <Button size="sm" variant="ghost" onClick={() => navigate(`/logs/${server.id}`)}>
                  Logs
                </Button>
                <Button size="sm" variant="secondary" onClick={() => setEditServer(server as any)}>
                  <FolderOpen size={13} /> Editar
                </Button>
                <Button
                  size="sm" variant="danger"
                  loading={deleting === server.id}
                  onClick={() => handleDelete(server.id, server.name)}
                >
                  <Trash2 size={13} />
                </Button>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Modal Adicionar */}
      <Modal open={showAdd} onClose={() => setShowAdd(false)} size="lg">
        <ServerForm
          title="Novo servidor"
          loading={saving}
          onSave={handleCreate}
          onCancel={() => setShowAdd(false)}
        />
      </Modal>

      {/* Modal Editar */}
      <Modal open={!!editServer} onClose={() => setEditServer(null)} size="lg">
        {editServer && (
          <ServerForm
            title={`Editar: ${editServer.name}`}
            initial={editServer as any}
            loading={saving}
            onSave={handleUpdate}
            onCancel={() => setEditServer(null)}
          />
        )}
      </Modal>
    </div>
  )
}

