import { useEffect, useState } from 'react'
import { Plus, Trash2, Link, Unlink, Network, RefreshCw } from 'lucide-react'
import {
  listClusters, createCluster, deleteCluster,
  assignServerToCluster, unassignServerFromCluster,
} from '../utils/tauri'
import { useServerStore } from '../stores/serverStore'
import { Button, Input, Card, CardHeader, CardTitle, Modal, Badge } from '../components/ui'
import type { Cluster, CreateClusterRequest } from '../types'
import toast from 'react-hot-toast'

const EMPTY: CreateClusterRequest = { name: '', clusterId: '', clusterDir: '', description: '' }

export default function ClusterManager() {
  const { servers, fetchServers } = useServerStore()
  const [clusters, setClusters] = useState<Cluster[]>([])
  const [loading, setLoading] = useState(false)
  const [showAdd, setShowAdd] = useState(false)
  const [form, setForm] = useState<CreateClusterRequest>(EMPTY)
  const [saving, setSaving] = useState(false)
  const [assignModal, setAssignModal] = useState<{ clusterId: number; clusterName: string } | null>(null)

  const load = async () => {
    setLoading(true)
    try {
      setClusters((await listClusters()) as Cluster[])
      await fetchServers()
    } catch (e) {
      toast.error(String(e))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  const handleCreate = async () => {
    if (!form.name || !form.clusterId || !form.clusterDir) {
      toast.error('Preencha nome, ID e pasta do cluster')
      return
    }
    setSaving(true)
    try {
      await createCluster(form)
      await load()
      setShowAdd(false)
      setForm(EMPTY)
      toast.success('Cluster criado')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSaving(false)
    }
  }

  const handleDelete = async (id: number, name: string) => {
    if (!confirm(`Excluir cluster "${name}"?`)) return
    try {
      await deleteCluster(id)
      await load()
      toast.success('Cluster excluído')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const handleAssign = async (serverId: number, clusterId: number) => {
    try {
      await assignServerToCluster(serverId, clusterId)
      await load()
      setAssignModal(null)
      toast.success('Servidor vinculado')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const handleUnassign = async (serverId: number) => {
    try {
      await unassignServerFromCluster(serverId)
      await load()
      toast.success('Servidor desvinculado')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const unassignedServers = servers.filter(s => !s.clusterId)

  return (
    <div className="p-6 space-y-4 h-full overflow-auto">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-slate-100 flex items-center gap-2">
            <Network size={16} className="text-ark-400" /> Cluster Cross-ARK
          </h1>
          <p className="text-xs text-slate-500 mt-0.5">Gerencie clusters para compartilhamento de personagens e itens</p>
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={load} loading={loading}>
            <RefreshCw size={13} />
          </Button>
          <Button size="sm" onClick={() => setShowAdd(true)}>
            <Plus size={13} /> Criar cluster
          </Button>
        </div>
      </div>

      {clusters.length === 0 && !loading ? (
        <Card className="text-center py-16">
          <Network size={36} className="text-slate-600 mx-auto mb-3" />
          <p className="text-slate-400 text-sm">Nenhum cluster configurado</p>
          <Button className="mt-4" size="sm" onClick={() => setShowAdd(true)}>
            <Plus size={13} /> Criar cluster
          </Button>
        </Card>
      ) : (
        <div className="space-y-3">
          {clusters.map(cluster => {
            const clusterServers = servers.filter(s => s.clusterId === cluster.id)
            return (
              <Card key={cluster.id}>
                <CardHeader>
                  <div>
                    <CardTitle>{cluster.name}</CardTitle>
                    <p className="text-xs text-slate-500 mt-0.5 font-mono">ID: {cluster.clusterId}</p>
                    <p className="text-xs text-slate-600 truncate max-w-xs">{cluster.clusterDir}</p>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      size="sm" variant="secondary"
                      onClick={() => setAssignModal({ clusterId: cluster.id, clusterName: cluster.name })}
                    >
                      <Link size={12} /> Vincular servidor
                    </Button>
                    <Button size="sm" variant="ghost" className="text-red-400 hover:text-red-300"
                      onClick={() => handleDelete(cluster.id, cluster.name)}>
                      <Trash2 size={13} />
                    </Button>
                  </div>
                </CardHeader>
                {clusterServers.length > 0 ? (
                  <div className="space-y-2 mt-2">
                    {clusterServers.map(s => (
                      <div key={s.id} className="flex items-center gap-2 bg-surface-700/30 rounded-lg px-3 py-2">
                        <span className="text-sm text-slate-200 flex-1">{s.name}</span>
                        <Badge variant="info">{s.map}</Badge>
                        <Button size="sm" variant="ghost" onClick={() => handleUnassign(s.id)}>
                          <Unlink size={12} /> Desvincular
                        </Button>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-xs text-slate-600 italic mt-2">Nenhum servidor vinculado</p>
                )}
              </Card>
            )
          })}
        </div>
      )}

      {/* Modal criar cluster */}
      <Modal open={showAdd} onClose={() => { setShowAdd(false); setForm(EMPTY) }} title="Criar cluster" size="md">
        <div className="space-y-3">
          <Input label="Nome do cluster" value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} />
          <Input label="Cluster ID" value={form.clusterId} onChange={e => setForm(f => ({ ...f, clusterId: e.target.value }))} hint="Identificador único (sem espaços)" />
          <Input label="Pasta compartilhada" value={form.clusterDir} onChange={e => setForm(f => ({ ...f, clusterDir: e.target.value }))} hint="Ex: C:\ARK\Cluster" />
          <Input label="Descrição (opcional)" value={form.description ?? ''} onChange={e => setForm(f => ({ ...f, description: e.target.value }))} />
          <div className="flex gap-2 pt-2">
            <Button loading={saving} onClick={handleCreate}>Criar</Button>
            <Button variant="ghost" onClick={() => { setShowAdd(false); setForm(EMPTY) }}>Cancelar</Button>
          </div>
        </div>
      </Modal>

      {/* Modal vincular servidor */}
      <Modal
        open={!!assignModal}
        onClose={() => setAssignModal(null)}
        title={`Vincular servidor ao cluster: ${assignModal?.clusterName}`}
        size="sm"
      >
        <div className="space-y-2">
          {unassignedServers.length === 0 ? (
            <p className="text-slate-400 text-sm">Todos os servidores já estão vinculados a um cluster.</p>
          ) : (
            unassignedServers.map(s => (
              <div key={s.id} className="flex items-center justify-between p-2 rounded-lg hover:bg-surface-700/50">
                <span className="text-sm text-slate-200">{s.name}</span>
                <Button size="sm" onClick={() => handleAssign(s.id, assignModal!.clusterId)}>
                  Vincular
                </Button>
              </div>
            ))
          )}
        </div>
      </Modal>
    </div>
  )
}

