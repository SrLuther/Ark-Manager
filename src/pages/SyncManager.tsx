import { useEffect, useState } from 'react'
import {
  FolderSync, Plus, Trash2, RefreshCw, Wifi,
  Link2, AlertTriangle,
  Clock, ArrowUpDown,
} from 'lucide-react'
import { useSyncStore } from '../stores/syncStore'
import { useAgentStore } from '../stores/agentStore'
import { Button, Badge } from '../components/ui'
import { SyncFolderCard } from '../components/sync/SyncFolderCard'
import { SyncStatusBadge } from '../components/sync/SyncStatusBadge'
import { PeerDiscoveryDialog } from '../components/sync/PeerDiscoveryDialog'
import { AddFolderDialog } from '../components/sync/AddFolderDialog'
import { PairingCodeDisplay } from '../components/sync/PairingCodeDisplay'
import { cn, formatRelative, formatBytes } from '../utils/helpers'
import type { SyncFolder, SyncAgent } from '../types'
import toast from 'react-hot-toast'

function AgentStatusBadge({ status }: { status: SyncAgent['status'] }) {
  if (status === 'online')
    return <Badge variant="success">Online</Badge>
  if (status === 'pairing')
    return <Badge variant="warning">Pareando</Badge>
  return <Badge variant="default">Offline</Badge>
}

function FolderDetail({ folder }: { folder: SyncFolder }) {
  const { events, conflicts, fetchEvents, fetchConflicts, forceSync, removeFolder } = useSyncStore()
  const { agents } = useAgentStore()
  const [loadingEvents, setLoadingEvents] = useState(false)
  const [tab, setTab] = useState<'events' | 'conflicts'>('events')
  const [syncing, setSyncing] = useState(false)
  const [removing, setRemoving] = useState(false)

  const peer = folder.agentId ? agents.find(a => a.id === folder.agentId) : null
  const folderEvents = events[folder.id] ?? []
  const folderConflicts = conflicts[folder.id] ?? []

  useEffect(() => {
    setLoadingEvents(true)
    Promise.all([
      fetchEvents(folder.id, 50),
      fetchConflicts(folder.id),
    ]).finally(() => setLoadingEvents(false))
  }, [folder.id, fetchEvents, fetchConflicts])

  const handleForceSync = async () => {
    setSyncing(true)
    try {
      await forceSync(folder.id)
      toast.success('Sincronização iniciada')
      await Promise.all([fetchEvents(folder.id, 50), fetchConflicts(folder.id)])
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSyncing(false)
    }
  }

  const handleRemove = async () => {
    if (!confirm(`Remover pasta "${folder.name}" da sincronização?`)) return
    setRemoving(true)
    try {
      await removeFolder(folder.id)
      toast.success('Pasta removida')
    } catch (e) {
      toast.error(String(e))
      setRemoving(false)
    }
  }

  return (
    <div className="flex flex-col gap-4">
      {/* Header */}
      <div className="flex items-start justify-between gap-2">
        <div className="flex flex-col gap-1 min-w-0">
          <div className="flex items-center gap-2">
            <FolderSync size={16} className="text-ark-400 shrink-0" />
            <span className="text-sm font-semibold text-slate-100 truncate">{folder.name}</span>
            <SyncStatusBadge status={folder.status} />
          </div>
          <span className="text-xs text-slate-500 font-mono truncate ml-6">{folder.localPath}</span>
        </div>
        <div className="flex gap-1.5 shrink-0">
          <Button
            size="sm"
            variant="secondary"
            loading={syncing}
            disabled={!folder.agentId}
            onClick={handleForceSync}
            title={!folder.agentId ? 'Vincule um peer para sincronizar' : 'Forçar sincronização agora'}
          >
            <RefreshCw size={12} /> Sincronizar
          </Button>
          <Button
            size="sm"
            variant="ghost"
            loading={removing}
            onClick={handleRemove}
            className="text-red-400 hover:text-red-300"
          >
            <Trash2 size={12} />
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
        <div className="bg-surface-800 rounded-lg p-3 text-xs">
          <p className="text-slate-500">Peer vinculado</p>
          <p className="text-slate-200 font-medium mt-0.5">
            {peer ? peer.name : <span className="text-slate-500 italic">Nenhum</span>}
          </p>
        </div>
        <div className="bg-surface-800 rounded-lg p-3 text-xs">
          <p className="text-slate-500">Última sincronização</p>
          <p className="text-slate-200 font-medium mt-0.5">
            {folder.lastSyncAt ? formatRelative(folder.lastSyncAt) : '—'}
          </p>
        </div>
        <div className="bg-surface-800 rounded-lg p-3 text-xs">
          <p className="text-slate-500">Transferido</p>
          <p className="text-slate-200 font-medium mt-0.5">
            {formatBytes(folder.bytesTransferred)}
          </p>
        </div>
        <div className={cn('bg-surface-800 rounded-lg p-3 text-xs', folder.conflictCount > 0 && 'border border-orange-800/60')}>
          <p className="text-slate-500">Conflitos</p>
          <p className={cn('font-medium mt-0.5', folder.conflictCount > 0 ? 'text-orange-400' : 'text-slate-200')}>
            {folder.conflictCount}
          </p>
        </div>
      </div>

      {/* Tabs eventos / conflitos */}
      <div className="flex border-b border-surface-700">
        {(['events', 'conflicts'] as const).map(t => (
          <button
            key={t}
            type="button"
            onClick={() => setTab(t)}
            className={cn(
              'px-4 py-2 text-xs font-medium transition-colors',
              tab === t
                ? 'text-ark-400 border-b-2 border-ark-500 -mb-px'
                : 'text-slate-500 hover:text-slate-300'
            )}
          >
            {t === 'events' ? 'Eventos' : 'Conflitos'}
            {t === 'conflicts' && folder.conflictCount > 0 && (
              <span className="ml-1.5 bg-orange-900/60 text-orange-400 text-[10px] px-1.5 py-0.5 rounded-full">
                {folder.conflictCount}
              </span>
            )}
          </button>
        ))}
      </div>

      {loadingEvents ? (
        <div className="text-center py-8 text-slate-500 text-xs">Carregando...</div>
      ) : tab === 'events' ? (
        <div className="space-y-0.5 max-h-64 overflow-y-auto scrollbar-thin">
          {folderEvents.length === 0 ? (
            <div className="text-center py-8 text-slate-500 text-xs">Nenhum evento registrado</div>
          ) : folderEvents.map(ev => (
            <div key={ev.id} className="flex items-start gap-2 px-2 py-1.5 rounded hover:bg-surface-800 text-xs">
              <ArrowUpDown size={10} className={cn(
                'shrink-0 mt-0.5',
                ev.direction === 'upload' ? 'text-ark-400' : 'text-emerald-400'
              )} />
              <div className="flex-1 min-w-0">
                <span className="text-slate-300 font-mono truncate block">{ev.path}</span>
                {ev.message && <span className="text-slate-500">{ev.message}</span>}
              </div>
              <div className="shrink-0 text-slate-600 flex items-center gap-1">
                {(ev.bytes ?? 0) > 0 && <span>{formatBytes(ev.bytes ?? 0)}</span>}
                <Clock size={8} />
                <span>{formatRelative(ev.createdAt)}</span>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="space-y-0.5 max-h-64 overflow-y-auto scrollbar-thin">
          {folderConflicts.length === 0 ? (
            <div className="text-center py-8 text-slate-500 text-xs">Nenhum conflito registrado</div>
          ) : folderConflicts.map(c => (
            <div key={c.id} className="flex items-start gap-2 px-2 py-1.5 rounded hover:bg-surface-800 text-xs">
              <AlertTriangle size={10} className="text-orange-400 shrink-0 mt-0.5" />
              <div className="flex-1 min-w-0">
                <span className="text-slate-300 font-mono truncate block">{c.path}</span>
                <span className="text-slate-500">
                  Resolução: <span className={c.resolution === 'local' ? 'text-ark-400' : 'text-emerald-400'}>
                    {c.resolution === 'local' ? 'Local (mais recente)' : 'Remoto (mais recente)'}
                  </span>
                </span>
              </div>
              <span className="shrink-0 text-slate-600">{formatRelative(c.createdAt)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default function SyncManager() {
  const { folders, loading, fetchFolders, addFolder } = useSyncStore()
  const {
    agents, discovered, loading: loadingAgents, discovering,
    fetchAgents, discoverAgents, pairAgent, removeAgent, generatePairingCode,
  } = useAgentStore()

  const [selectedFolder, setSelectedFolder] = useState<number | null>(null)
  const [showPeerDialog, setShowPeerDialog] = useState(false)
  const [showAddFolder, setShowAddFolder] = useState(false)
  const [showPairingCode, setShowPairingCode] = useState(false)

  useEffect(() => {
    fetchFolders()
    fetchAgents()
  }, [fetchFolders, fetchAgents])

  const currentFolder = folders.find(f => f.id === selectedFolder) ?? null

  const handleAddFolder = async (name: string, localPath: string, agentId?: number) => {
    await addFolder(name, localPath, agentId)
    toast.success('Pasta adicionada')
  }

  const handlePairAgent = async (agent: { name: string; address: string; port: number }, code: string) => {
    await pairAgent(agent.address, agent.port, code)
    toast.success(`Peer "${agent.name}" pareado com sucesso`)
    await fetchAgents()
  }

  const handleRemoveAgent = async (id: number) => {
    const agent = agents.find(a => a.id === id)
    if (!confirm(`Remover peer "${agent?.name}"?`)) return
    try {
      await removeAgent(id)
      toast.success('Peer removido')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const pairedAgents = agents.filter(a => a.paired)

  return (
    <div className="flex h-full overflow-hidden">
      {/* Painel esquerdo — Pastas */}
      <div className="flex flex-col w-72 min-w-72 border-r border-surface-700 h-full">
        <div className="flex items-center justify-between px-4 py-3 border-b border-surface-700">
          <span className="text-sm font-semibold text-slate-200">Pastas sincronizadas</span>
          <div className="flex gap-1">
            <Button
              size="sm"
              variant="ghost"
              loading={loading}
              onClick={fetchFolders}
              title="Atualizar"
            >
              <RefreshCw size={13} />
            </Button>
            <Button
              size="sm"
              onClick={() => setShowAddFolder(true)}
              disabled={folders.length >= 5}
              title={folders.length >= 5 ? 'Limite de 5 pastas atingido' : 'Adicionar pasta'}
            >
              <Plus size={13} /> Adicionar
            </Button>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto scrollbar-thin py-2 px-2 space-y-1.5">
          {loading && folders.length === 0 ? (
            <div className="text-center py-8 text-slate-500 text-xs">Carregando...</div>
          ) : folders.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-10 text-slate-600 px-4 text-center">
              <FolderSync size={24} />
              <p className="text-xs">Nenhuma pasta configurada para sincronização</p>
              <Button size="sm" variant="secondary" onClick={() => setShowAddFolder(true)}>
                <Plus size={12} /> Adicionar pasta
              </Button>
            </div>
          ) : folders.map(folder => (
            <button
              key={folder.id}
              type="button"
              onClick={() => setSelectedFolder(folder.id === selectedFolder ? null : folder.id)}
              className={cn(
                'w-full text-left rounded-xl transition-all',
                folder.id === selectedFolder && 'ring-1 ring-ark-500'
              )}
            >
              <SyncFolderCard folder={folder} />
            </button>
          ))}
        </div>

        {/* Limite de pastas */}
        <div className="px-4 py-2 border-t border-surface-700 text-xs text-slate-600 flex justify-between">
          <span>Pastas: {folders.length}/5</span>
          {folders.some(f => f.conflictCount > 0) && (
            <span className="text-orange-400 flex items-center gap-1">
              <AlertTriangle size={10} /> Conflitos detectados
            </span>
          )}
        </div>
      </div>

      {/* Painel central — Detalhe da pasta */}
      <div className="flex-1 flex flex-col h-full overflow-hidden">
        <div className="flex-1 overflow-y-auto scrollbar-thin p-5">
          {currentFolder ? (
            <FolderDetail folder={currentFolder} />
          ) : (
            <div className="flex flex-col items-center justify-center h-full text-slate-600 gap-3">
              <FolderSync size={36} />
              <p className="text-sm">Selecione uma pasta para ver detalhes</p>
              <div className="flex gap-2 mt-1">
                <Button size="sm" variant="secondary" onClick={() => setShowAddFolder(true)}>
                  <Plus size={12} /> Nova pasta
                </Button>
                <Button size="sm" variant="ghost" onClick={() => setShowPeerDialog(true)}>
                  <Wifi size={12} /> Descobrir peers
                </Button>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Painel direito — Peers/Agentes */}
      <div className="flex flex-col w-64 min-w-64 border-l border-surface-700 h-full">
        <div className="flex items-center justify-between px-4 py-3 border-b border-surface-700">
          <span className="text-sm font-semibold text-slate-200">Peers</span>
          <div className="flex gap-1">
            <Button
              size="sm"
              variant="ghost"
              title="Meu código de pareamento"
              onClick={() => setShowPairingCode(true)}
            >
              <Link2 size={13} />
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => setShowPeerDialog(true)}
              loading={loadingAgents}
            >
              <Wifi size={13} /> Descobrir
            </Button>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto scrollbar-thin py-2">
          {/* Pareados */}
          {pairedAgents.length > 0 && (
            <div className="mb-3">
              <div className="px-4 py-1.5">
                <p className="text-[10px] font-medium uppercase tracking-wider text-slate-500">Vinculados</p>
              </div>
              <div className="divide-y divide-surface-700/50">
                {pairedAgents.map(agent => (
                  <div key={agent.id} className="flex items-center justify-between px-4 py-2.5 group">
                    <div className="flex flex-col gap-0.5 min-w-0">
                      <span className="text-xs text-slate-200 font-medium truncate">{agent.name}</span>
                      <span className="text-[10px] text-slate-500 font-mono">{agent.address}:{agent.port}</span>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <AgentStatusBadge status={agent.status} />
                      <button
                        type="button"
                        onClick={() => handleRemoveAgent(agent.id)}
                        className="opacity-0 group-hover:opacity-100 transition-opacity text-red-500 hover:text-red-400 p-0.5"
                        title="Remover peer"
                      >
                        <Trash2 size={11} />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {pairedAgents.length === 0 && (
            <div className="flex flex-col items-center gap-2 py-10 px-4 text-center text-slate-600">
              <Wifi size={20} />
              <p className="text-xs">Nenhum peer vinculado</p>
              <Button size="sm" variant="ghost" onClick={() => setShowPeerDialog(true)}>
                Descobrir peers na rede
              </Button>
            </div>
          )}

          {/* Descobertos não pareados */}
          {discovered.filter(d => !pairedAgents.some(p => p.address === d.address && p.port === d.port)).length > 0 && (
            <div className="border-t border-surface-700 pt-2">
              <div className="px-4 py-1.5">
                <p className="text-[10px] font-medium uppercase tracking-wider text-slate-600">Descobertos</p>
              </div>
              {discovered
                .filter(d => !pairedAgents.some(p => p.address === d.address && p.port === d.port))
                .map(d => (
                  <div key={`${d.address}:${d.port}`} className="flex items-center justify-between px-4 py-2">
                    <div className="flex flex-col gap-0.5 min-w-0">
                      <span className="text-xs text-slate-400 truncate">{d.name}</span>
                      <span className="text-[10px] text-slate-600 font-mono">{d.address}:{d.port}</span>
                    </div>
                    <Button size="sm" variant="ghost" onClick={() => setShowPeerDialog(true)}>
                      Parear
                    </Button>
                  </div>
                ))
              }
            </div>
          )}
        </div>

        {/* Rodapé — refresh */}
        <div className="px-4 py-2 border-t border-surface-700">
          <Button
            size="sm"
            variant="ghost"
            className="w-full"
            loading={discovering}
            onClick={async () => {
              await discoverAgents()
              toast.success(`${discovered.length} peer(s) encontrado(s)`)
            }}
          >
            <RefreshCw size={12} /> Varrer rede
          </Button>
        </div>
      </div>

      {/* Dialogs */}
      <AddFolderDialog
        open={showAddFolder}
        onClose={() => setShowAddFolder(false)}
        agents={agents}
        onAdd={handleAddFolder}
      />

      <PeerDiscoveryDialog
        open={showPeerDialog}
        onClose={() => setShowPeerDialog(false)}
        discoveredAgents={agents.filter(a => !a.paired).map(a => ({
          ...a,
          lastSeenAt: a.lastSeenAt,
        }))}
        onPair={handlePairAgent}
        onRefresh={discoverAgents}
        refreshing={discovering}
      />

      <PairingCodeDisplay
        open={showPairingCode}
        onClose={() => setShowPairingCode(false)}
        onGenerate={generatePairingCode}
      />
    </div>
  )
}
