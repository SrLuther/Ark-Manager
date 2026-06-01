import { useEffect, useState } from 'react'
import { Plus, Trash2, Clock, RefreshCw } from 'lucide-react'
import { listTasks, createTask, deleteTask, validateCronExpression } from '../utils/tauri'
import { useServerStore } from '../stores/serverStore'
import { Button, Input, Card, Modal, Badge } from '../components/ui'
import type { ScheduledTask, CreateTaskRequest, TaskType } from '../types'
import toast from 'react-hot-toast'

const TASK_TYPES: { value: TaskType; label: string }[] = [
  { value: 'Restart',        label: 'Reiniciar servidor' },
  { value: 'CreateBackup',   label: 'Backup' },
  { value: 'Broadcast',      label: 'Broadcast' },
  { value: 'ExecuteCommand', label: 'Comando RCON' },
  { value: 'UpdateServer',   label: 'Atualizar servidor' },
  { value: 'Saveworld',      label: 'Salvar mundo' },
]

const EMPTY: CreateTaskRequest = {
  serverId: 0, taskType: 'Restart', cronExpression: '',
}

export default function Scheduler() {
  const { servers } = useServerStore()
  const [selectedServerId, setSelectedServerId] = useState<number>(0)
  const [tasks, setTasks] = useState<ScheduledTask[]>([])
  const [loading, setLoading] = useState(false)
  const [showAdd, setShowAdd] = useState(false)
  const [form, setForm] = useState<CreateTaskRequest>({ ...EMPTY })
  const [cronValid, setCronValid] = useState<{ valid: boolean; next?: string } | null>(null)
  const [saving, setSaving] = useState(false)

  const load = async (sid: number) => {
    if (!sid) return
    setLoading(true)
    try {
      setTasks(await listTasks(sid))
    } catch (e) {
      toast.error(String(e))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    if (servers.length > 0 && !selectedServerId) {
      setSelectedServerId(servers[0].id)
    }
  }, [servers])

  useEffect(() => { if (selectedServerId) load(selectedServerId) }, [selectedServerId])

  const handleCronBlur = async () => {
    if (!form.cronExpression.trim()) { setCronValid(null); return }
    try {
      const next = await validateCronExpression(form.cronExpression)
      setCronValid({ valid: true, next: next ?? undefined })
    } catch {
      setCronValid({ valid: false })
    }
  }

  const handleCreate = async () => {
    if (!form.cronExpression || !form.taskType) { toast.error('Preencha expressão cron e tipo'); return }
    if (cronValid && !cronValid.valid) { toast.error('Expressão cron inválida'); return }
    setSaving(true)
    try {
      await createTask({ ...form, serverId: selectedServerId })
      await load(selectedServerId)
      setShowAdd(false)
      setForm({ ...EMPTY })
      setCronValid(null)
      toast.success('Tarefa criada')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSaving(false)
    }
  }

  const handleDelete = async (id: number) => {
    if (!confirm('Excluir esta tarefa agendada?')) return
    try {
      await deleteTask(id)
      await load(selectedServerId)
      toast.success('Tarefa removida')
    } catch (e) {
      toast.error(String(e))
    }
  }

  const taskLabel = (type: TaskType) => TASK_TYPES.find(t => t.value === type)?.label ?? type

  return (
    <div className="p-6 space-y-4 h-full overflow-auto">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-slate-100 flex items-center gap-2">
            <Clock size={16} className="text-ark-400" /> Agendador
          </h1>
          <p className="text-xs text-slate-500 mt-0.5">Automação de tarefas por expressão cron</p>
        </div>
        <div className="flex gap-2 items-center">
          {servers.length > 0 && (
            <select
              className="bg-surface-700 border border-surface-600 rounded-lg px-3 py-1.5 text-sm text-slate-200 outline-none focus:ring-1 focus:ring-ark-500"
              value={selectedServerId}
              onChange={e => setSelectedServerId(Number(e.target.value))}
            >
              {servers.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
            </select>
          )}
          <Button size="sm" variant="secondary" onClick={() => load(selectedServerId)} loading={loading}>
            <RefreshCw size={13} />
          </Button>
          <Button size="sm" onClick={() => setShowAdd(true)} disabled={!selectedServerId}>
            <Plus size={13} /> Nova tarefa
          </Button>
        </div>
      </div>

      {!selectedServerId ? (
        <Card className="text-center py-16">
          <p className="text-slate-400 text-sm">Selecione um servidor acima para ver as tarefas agendadas.</p>
        </Card>
      ) : tasks.length === 0 && !loading ? (
        <Card className="text-center py-16">
          <Clock size={36} className="text-slate-600 mx-auto mb-3" />
          <p className="text-slate-400 text-sm">Nenhuma tarefa agendada</p>
          <Button className="mt-4" size="sm" onClick={() => setShowAdd(true)}>
            <Plus size={13} /> Nova tarefa
          </Button>
        </Card>
      ) : (
        <Card noPad>
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left border-b border-surface-700">
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Tipo</th>
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Expressão cron</th>
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Mensagem / Comando</th>
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Próxima execução</th>
                <th className="px-4 py-3 text-xs text-slate-400 font-medium">Status</th>
                <th className="px-4 py-3"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-700/50">
              {tasks.map(t => (
                <tr key={t.id} className="hover:bg-surface-700/20 transition-colors">
                  <td className="px-4 py-3 text-slate-200">{taskLabel(t.taskType)}</td>
                  <td className="px-4 py-3 font-mono text-xs text-slate-400">{t.cronExpression}</td>
                  <td className="px-4 py-3 text-xs text-slate-500 max-w-[160px] truncate">
                    {t.message ?? t.command ?? '—'}
                  </td>
                  <td className="px-4 py-3 text-xs text-slate-400">
                    {t.nextRun ? new Date(t.nextRun).toLocaleString('pt-BR') : '—'}
                  </td>
                  <td className="px-4 py-3">
                    <Badge variant={t.enabled ? 'success' : 'default'}>
                      {t.enabled ? 'ativo' : 'inativo'}
                    </Badge>
                  </td>
                  <td className="px-4 py-3 flex gap-1 justify-end">
                    <Button size="sm" variant="ghost" className="text-red-400 hover:text-red-300" onClick={() => handleDelete(t.id)}>
                      <Trash2 size={13} />
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}

      {/* Modal nova tarefa */}
      <Modal open={showAdd} onClose={() => { setShowAdd(false); setForm({ ...EMPTY }); setCronValid(null) }} title="Nova tarefa agendada" size="md">
        <div className="space-y-3">
          <div>
            <label className="text-xs text-slate-400 mb-1 block">Tipo de tarefa</label>
            <select
              className="w-full bg-surface-700 border border-surface-600 rounded-lg px-3 py-2 text-sm text-slate-200 outline-none focus:ring-1 focus:ring-ark-500"
              value={form.taskType}
              onChange={e => setForm(f => ({ ...f, taskType: e.target.value as TaskType }))}
            >
              {TASK_TYPES.map(t => <option key={t.value} value={t.value}>{t.label}</option>)}
            </select>
          </div>

          <Input
            label="Expressão cron"
            placeholder="Ex: 0 4 * * * (todo dia às 04h)"
            value={form.cronExpression}
            onChange={e => { setForm(f => ({ ...f, cronExpression: e.target.value })); setCronValid(null) }}
            onBlur={handleCronBlur}
            hint={cronValid
              ? (cronValid.valid ? `Próxima: ${cronValid.next ?? '—'}` : 'Expressão inválida')
              : 'min hora dom mês dow'}
          />

          {(form.taskType === 'Broadcast' || form.taskType === 'ExecuteCommand') && (
            <Input
              label={form.taskType === 'Broadcast' ? 'Mensagem' : 'Comando RCON'}
              value={form.message ?? form.command ?? ''}
              onChange={e => {
                if (form.taskType === 'Broadcast') setForm(f => ({ ...f, message: e.target.value }))
                else setForm(f => ({ ...f, command: e.target.value }))
              }}
            />
          )}

          <div className="flex gap-2 pt-2">
            <Button loading={saving} onClick={handleCreate}>Criar tarefa</Button>
            <Button variant="ghost" onClick={() => { setShowAdd(false); setForm({ ...EMPTY }); setCronValid(null) }}>Cancelar</Button>
          </div>
        </div>
      </Modal>
    </div>
  )
}
