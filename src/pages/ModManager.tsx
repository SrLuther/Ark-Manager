import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Plus, Trash2, GripVertical, Puzzle, RefreshCw } from 'lucide-react'
import { useServerStore } from '../stores/serverStore'
import { listMods, addMod, removeMod, reorderMods } from '../utils/tauri'
import { Button, Input, Card, CardHeader, CardTitle } from '../components/ui'
import type { ModEntry } from '../types'
import toast from 'react-hot-toast'

export default function ModManager() {
  const { serverId } = useParams<{ serverId: string }>()
  const navigate = useNavigate()
  const { servers } = useServerStore()
  const server = servers.find(s => s.id === Number(serverId))

  const [mods, setMods] = useState<ModEntry[]>([])
  const [newModId, setNewModId] = useState('')
  const [loading, setLoading] = useState(false)
  const [adding, setAdding] = useState(false)
  const [dragging, setDragging] = useState<number | null>(null)

  const loadMods = async () => {
    if (!server) return
    setLoading(true)
    try {
      setMods((await listMods(server.id)) as ModEntry[])
    } catch (e) {
      toast.error(String(e))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { if (server) loadMods() }, [server?.id])

  const handleAdd = async () => {
    if (!server || !newModId.trim()) return
    setAdding(true)
    try {
      await addMod(server.id, newModId.trim())
      await loadMods()
      setNewModId('')
      toast.success('Mod adicionado')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setAdding(false)
    }
  }

  const handleRemove = async (modId: string) => {
    if (!server) return
    try {
      await removeMod(server.id, modId)
      await loadMods()
      toast.success('Mod removido')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const handleDrop = async (targetIndex: number) => {
    if (!server || dragging === null) return
    const newMods = [...mods]
    const [moved] = newMods.splice(dragging, 1)
    newMods.splice(targetIndex, 0, moved)
    setMods(newMods)
    setDragging(null)
    try {
      await reorderMods(server.id, newMods.map(m => m.modId))
    } catch (e) {
      toast.error(String(e))
      await loadMods()
    }
  }

  if (!server) {
    return (
      <div className="p-6">
        <Button variant="ghost" size="sm" onClick={() => navigate('/servers')}>
          <ArrowLeft size={14} /> Voltar
        </Button>
        <p className="text-slate-400 mt-4 text-sm">Servidor não encontrado.</p>
      </div>
    )
  }

  return (
    <div className="p-6 space-y-4 h-full overflow-auto">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
            <ArrowLeft size={14} />
          </Button>
          <div>
            <h1 className="text-lg font-semibold text-slate-100">Mods</h1>
            <p className="text-xs text-slate-500">{server.name}</p>
          </div>
        </div>
        <Button size="sm" variant="secondary" onClick={loadMods} loading={loading}>
          <RefreshCw size={13} />
        </Button>
      </div>

      {/* Adicionar mod */}
      <Card>
        <CardHeader><CardTitle className="flex items-center gap-2"><Plus size={14} /> Adicionar mod</CardTitle></CardHeader>
        <div className="flex gap-2">
          <Input
            className="flex-1"
            placeholder="ID do mod Steam Workshop (ex: 731604991)"
            value={newModId}
            onChange={e => setNewModId(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleAdd()}
          />
          <Button loading={adding} onClick={handleAdd} disabled={!newModId.trim()}>
            <Plus size={14} /> Adicionar
          </Button>
        </div>
        <p className="text-xs text-slate-500 mt-2">
          Encontre IDs em{' '}
          <span className="text-ark-400">steamcommunity.com/workshop</span>
        </p>
      </Card>

      {/* Lista de mods */}
      <Card noPad>
        <div className="px-4 py-3 border-b border-surface-700 flex items-center justify-between">
          <p className="text-sm font-semibold text-slate-100 flex items-center gap-2">
            <Puzzle size={14} className="text-ark-400" /> Mods instalados
          </p>
          <span className="text-xs text-slate-500">{mods.length} mod(s)</span>
        </div>
        {mods.length === 0 ? (
          <div className="text-center py-10">
            <Puzzle size={28} className="text-slate-600 mx-auto mb-2" />
            <p className="text-slate-500 text-sm">Nenhum mod instalado</p>
          </div>
        ) : (
          <div>
            {mods.map((mod, i) => (
              <div
                key={mod.modId}
                draggable
                onDragStart={() => setDragging(i)}
                onDragOver={e => e.preventDefault()}
                onDrop={() => handleDrop(i)}
                className={`flex items-center gap-3 px-4 py-3 border-b border-surface-700/50 last:border-0 hover:bg-surface-700/30 transition-colors ${dragging === i ? 'opacity-50' : ''}`}
              >
                <GripVertical size={14} className="text-slate-600 cursor-grab shrink-0" />
                <span className="text-xs text-slate-500 w-6 shrink-0">{i + 1}</span>
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-slate-200 font-mono">{mod.modId}</p>
                  {mod.name && <p className="text-xs text-slate-500 truncate">{mod.name}</p>}
                </div>
                <Button
                  size="sm" variant="ghost"
                  className="text-red-400 hover:text-red-300 hover:bg-red-900/20"
                  onClick={() => handleRemove(mod.modId)}
                >
                  <Trash2 size={13} />
                </Button>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  )
}

