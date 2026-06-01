import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { listen } from '@tauri-apps/api/event'
import { ArrowLeft, Plus, RotateCcw, Trash2, Archive, RefreshCw } from 'lucide-react'
import { useServerStore } from '../stores/serverStore'
import { listBackups, createBackup, restoreBackup, pruneBackups } from '../utils/tauri'
import { Button, Card, Badge } from '../components/ui'
import { formatBytes, formatDate, formatRelative } from '../utils/helpers'
import type { Backup } from '../types'
import toast from 'react-hot-toast'

export default function Backups() {
  const { serverId } = useParams<{ serverId: string }>()
  const navigate = useNavigate()
  const { servers } = useServerStore()
  const server = servers.find(s => s.id === Number(serverId))

  const [backups, setBackups] = useState<Backup[]>([])
  const [loading, setLoading] = useState(false)
  const [backing, setBacking] = useState(false)
  const [restoring, setRestoring] = useState<number | null>(null)

  const load = async () => {
    if (!server) return
    setLoading(true)
    try {
      setBackups((await listBackups(server.id)) as Backup[])
    } catch (e) {
      toast.error(String(e))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (!server) return
    load()
    const unsubs = Promise.all([
      listen('backup:started',   () => toast.loading('Backup iniciado...', { id: 'backup' })),
      listen('backup:completed', () => { toast.success('Backup concluído', { id: 'backup' }); load() }),
      listen('backup:failed',    (e: any) => toast.error('Backup falhou: ' + e.payload, { id: 'backup' })),
    ])
    return () => { unsubs.then(fns => fns.forEach(f => f())) }
  }, [server?.id])

  const handleCreate = async () => {
    if (!server) return
    setBacking(true)
    try {
      await createBackup(server.id, server.installDir, '')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setBacking(false)
    }
  }

  const handleRestore = async (backup: Backup) => {
    if (!server) return
    if (!confirm(`Restaurar backup de ${formatDate(backup.createdAt)}?\nO servidor será parado se estiver rodando.`)) return
    setRestoring(backup.id)
    try {
      await restoreBackup(backup.backupPath ?? '', server.installDir)
      toast.success('Backup restaurado com sucesso')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setRestoring(null)
    }
  }

  const handlePrune = async () => {
    if (!server || !confirm('Excluir backups antigos além do limite configurado?')) return
    try {
      await pruneBackups('', server.id, 10)
      await load()
      toast.success('Backups antigos removidos')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const statusVariant = (status: string) => {
    if (status === 'completed') return 'success'
    if (status === 'failed')    return 'error'
    return 'warning'
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
            <h1 className="text-lg font-semibold text-slate-100">Backups</h1>
            <p className="text-xs text-slate-500">{server.name}</p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={load} loading={loading}>
            <RefreshCw size={13} />
          </Button>
          <Button size="sm" variant="ghost" onClick={handlePrune}>
            <Trash2 size={13} /> Limpar antigos
          </Button>
          <Button size="sm" loading={backing} onClick={handleCreate}>
            <Plus size={13} /> Criar backup
          </Button>
        </div>
      </div>

      {backups.length === 0 && !loading ? (
        <Card className="text-center py-16">
          <Archive size={36} className="text-slate-600 mx-auto mb-3" />
          <p className="text-slate-400 text-sm">Nenhum backup encontrado</p>
          <Button className="mt-4" size="sm" onClick={handleCreate} loading={backing}>
            <Plus size={13} /> Criar backup
          </Button>
        </Card>
      ) : (
        <Card noPad>
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left border-b border-surface-700">
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Data</th>
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Tamanho</th>
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Tipo</th>
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Status</th>
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Caminho</th>
                <th className="px-4 py-3"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-700/50">
              {backups.map(b => (
                <tr key={b.id} className="hover:bg-surface-700/20 transition-colors">
                  <td className="px-4 py-3">
                    <div>
                      <p className="text-slate-200">{formatDate(b.createdAt)}</p>
                      <p className="text-xs text-slate-500">{formatRelative(b.createdAt)}</p>
                    </div>
                  </td>
                  <td className="px-4 py-3 text-slate-400">{b.sizeBytes ? formatBytes(b.sizeBytes) : '—'}</td>
                  <td className="px-4 py-3 text-slate-400 capitalize">{b.backupType}</td>
                  <td className="px-4 py-3">
                    <Badge variant={statusVariant(b.status) as any}>{b.status}</Badge>
                  </td>
                  <td className="px-4 py-3 max-w-xs">
                    <p className="text-xs text-slate-500 truncate font-mono" title={b.backupPath ?? ''}>{b.backupPath}</p>
                  </td>
                  <td className="px-4 py-3">
                    {b.status === 'completed' && (
                      <Button
                        size="sm" variant="secondary"
                        loading={restoring === b.id}
                        onClick={() => handleRestore(b)}
                      >
                        <RotateCcw size={12} /> Restaurar
                      </Button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}
    </div>
  )
}

