import { useState, useEffect } from 'react'
import { Plus, RefreshCw, CalendarDays, Zap, Play, StopCircle } from 'lucide-react'
import toast from 'react-hot-toast'
import { useSeasonalEventStore } from '../stores/seasonalEventStore'
import { useServerStore } from '../stores/serverStore'
import { EventCard } from '../components/events/EventCard'
import { EventRatesForm } from '../components/events/EventRatesForm'
import { EventServerSelector } from '../components/events/EventServerSelector'
import { Button, Modal } from '../components/ui'
import { cn } from '../utils/helpers'
import type { CreateEventRequest, EventRate, Server } from '../types'

// ─────────────────────────────────────────────
// Dialog de criação de evento
// ─────────────────────────────────────────────

const DEFAULT_RATES: EventRate = {
  xpMultiplier: 2.0,
  harvestMultiplier: 2.0,
  tameSpeedMultiplier: 2.0,
  breedingMultiplier: 2.0,
  hatchSpeedMultiplier: 2.0,
  matureSpeedMultiplier: 2.0,
}

interface CreateEventDialogProps {
  open: boolean
  onClose: () => void
  onSave: (req: CreateEventRequest) => Promise<void>
  serverList: Server[]
}

function CreateEventDialog({ open, onClose, onSave, serverList }: CreateEventDialogProps) {
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [startAt, setStartAt] = useState('')
  const [endAt, setEndAt] = useState('')
  const [rates, setRates] = useState<EventRate>(DEFAULT_RATES)
  const [serverIds, setServerIds] = useState<number[]>([])
  const [saving, setSaving] = useState(false)

  // Reseta ao fechar
  useEffect(() => {
    if (!open) {
      setName(''); setDescription(''); setStartAt(''); setEndAt('')
      setRates(DEFAULT_RATES); setServerIds([]); setSaving(false)
    }
  }, [open])

  const valid = name.trim().length > 0 && startAt && endAt && endAt > startAt && serverIds.length > 0

  const handleSave = async () => {
    if (!valid) return
    setSaving(true)
    try {
      await onSave({
        name: name.trim(),
        description: description.trim() || undefined,
        startAt,
        endAt,
        rates,
        serverIds,
      })
      onClose()
    } catch (err) {
      toast.error(String(err))
    } finally {
      setSaving(false)
    }
  }

  return (
    <Modal open={open} onClose={onClose} title="Novo Evento Sazonal" size="lg">
      <div className="flex flex-col gap-4">
        {/* Nome */}
        <div>
          <label className="block text-xs text-slate-400 mb-1">Nome do evento *</label>
          <input
            type="text"
            value={name}
            onChange={e => setName(e.target.value)}
            placeholder="Ex: Evento de Natal"
            className="w-full bg-surface-800 border border-surface-600 rounded-lg px-3 py-2
                       text-sm text-slate-100 placeholder-slate-600
                       focus:outline-none focus:ring-2 focus:ring-ark-500"
          />
        </div>

        {/* Descrição */}
        <div>
          <label className="block text-xs text-slate-400 mb-1">Descrição</label>
          <textarea
            value={description}
            onChange={e => setDescription(e.target.value)}
            placeholder="Descrição exibida aos jogadores..."
            rows={2}
            className="w-full bg-surface-800 border border-surface-600 rounded-lg px-3 py-2
                       text-sm text-slate-100 placeholder-slate-600 resize-none
                       focus:outline-none focus:ring-2 focus:ring-ark-500"
          />
        </div>

        {/* Datas */}
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="block text-xs text-slate-400 mb-1">Início *</label>
            <input
              type="datetime-local"
              value={startAt}
              onChange={e => setStartAt(e.target.value)}
              className="w-full bg-surface-800 border border-surface-600 rounded-lg px-3 py-2
                         text-sm text-slate-100 focus:outline-none focus:ring-2 focus:ring-ark-500"
            />
          </div>
          <div>
            <label className="block text-xs text-slate-400 mb-1">Fim *</label>
            <input
              type="datetime-local"
              value={endAt}
              onChange={e => setEndAt(e.target.value)}
              min={startAt}
              className={cn(
                'w-full bg-surface-800 border rounded-lg px-3 py-2',
                'text-sm text-slate-100 focus:outline-none focus:ring-2 focus:ring-ark-500',
                endAt && endAt <= startAt
                  ? 'border-red-700 focus:ring-red-500'
                  : 'border-surface-600'
              )}
            />
            {endAt && endAt <= startAt && (
              <p className="text-xs text-red-400 mt-1">Fim deve ser posterior ao início</p>
            )}
          </div>
        </div>

        {/* Taxas */}
        <div>
          <label className="block text-xs text-slate-400 mb-2">Taxas do evento</label>
          <EventRatesForm value={rates} onChange={setRates} />
        </div>

        {/* Servidores */}
        <div>
          <label className="block text-xs text-slate-400 mb-2">
            Servidores participantes *
          </label>
          <EventServerSelector
            servers={serverList}
            selected={serverIds}
            onChange={setServerIds}
          />
          {serverIds.length === 0 && (
            <p className="text-xs text-amber-400 mt-1">
              Selecione ao menos um servidor
            </p>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 pt-2 border-t border-surface-700">
          <Button variant="ghost" onClick={onClose} disabled={saving}>
            Cancelar
          </Button>
          <Button
            onClick={handleSave}
            disabled={!valid || saving}
            className="gap-2"
          >
            {saving ? (
              <>
                <RefreshCw size={14} className="animate-spin" />
                Criando...
              </>
            ) : (
              <>
                <Plus size={14} />
                Criar evento
              </>
            )}
          </Button>
        </div>
      </div>
    </Modal>
  )
}

// ─────────────────────────────────────────────
// Diálogo de confirmação de cancelamento
// ─────────────────────────────────────────────

interface ConfirmCancelDialogProps {
  open: boolean
  eventName: string
  onConfirm: () => void
  onClose: () => void
}

function ConfirmCancelDialog({ open, eventName, onConfirm, onClose }: ConfirmCancelDialogProps) {
  return (
    <Modal open={open} onClose={onClose} title="Cancelar evento" size="sm">
      <p className="text-sm text-slate-300 mb-4">
        Tem certeza que deseja cancelar <span className="font-semibold text-slate-100">"{eventName}"</span>?
        Se o evento estiver ativo, os servidores serão reiniciados e os INIs originais serão restaurados.
      </p>
      <div className="flex justify-end gap-2">
        <Button variant="ghost" onClick={onClose}>Voltar</Button>
        <Button
          onClick={onConfirm}
          className="bg-red-700 hover:bg-red-600 text-white"
        >
          Cancelar evento
        </Button>
      </div>
    </Modal>
  )
}

// ─────────────────────────────────────────────
// Página principal
// ─────────────────────────────────────────────

export default function SeasonalEvents() {
  const { events, loading, error, fetchEvents, createEvent, cancelEvent, forceStart, forceEnd } =
    useSeasonalEventStore()
  const { servers: allServers, fetchServers } = useServerStore()

  const [showCreate, setShowCreate] = useState(false)
  const [cancelTarget, setCancelTarget] = useState<{ id: number; name: string } | null>(null)
  const [actionLoading, setActionLoading] = useState<number | null>(null)

  useEffect(() => {
    fetchEvents()
    fetchServers()
  }, [fetchEvents, fetchServers])

  // Refresh automático a cada 30s para manter status atualizado
  useEffect(() => {
    const id = setInterval(fetchEvents, 30_000)
    return () => clearInterval(id)
  }, [fetchEvents])

  const handleCreate = async (req: CreateEventRequest) => {
    await createEvent(req)
    toast.success('Evento criado com sucesso!')
  }

  const handleCancel = async () => {
    if (!cancelTarget) return
    setActionLoading(cancelTarget.id)
    try {
      await cancelEvent(cancelTarget.id)
      toast.success('Evento cancelado')
    } catch (err) {
      toast.error(String(err))
    } finally {
      setActionLoading(null)
      setCancelTarget(null)
    }
  }

  const handleForceStart = async (id: number) => {
    setActionLoading(id)
    try {
      await forceStart(id)
      toast.success('Evento iniciado manualmente')
    } catch (err) {
      toast.error(String(err))
    } finally {
      setActionLoading(null)
    }
  }

  const handleForceEnd = async (id: number) => {
    setActionLoading(id)
    try {
      await forceEnd(id)
      toast.success('Evento encerrado manualmente')
    } catch (err) {
      toast.error(String(err))
    } finally {
      setActionLoading(null)
    }
  }

  const activeEvents = events.filter(e => e.status === 'active')
  const scheduledEvents = events.filter(e => e.status === 'scheduled')
  const pastEvents = events.filter(e => e.status === 'ended' || e.status === 'cancelled' || e.status === 'error')

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-surface-700">
        <div>
          <h1 className="text-lg font-semibold text-slate-100 flex items-center gap-2">
            <CalendarDays size={20} className="text-ark-400" />
            Eventos Sazonais
          </h1>
          <p className="text-xs text-slate-500 mt-0.5">
            Gerencie eventos temporários com taxas especiais nos servidores ARK
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={fetchEvents}
            disabled={loading}
            className="gap-1.5"
          >
            <RefreshCw size={13} className={cn(loading && 'animate-spin')} />
            Atualizar
          </Button>
          <Button size="sm" onClick={() => setShowCreate(true)} className="gap-1.5">
            <Plus size={14} />
            Novo evento
          </Button>
        </div>
      </div>

      {/* Conteúdo */}
      <div className="flex-1 overflow-y-auto p-6">
        {error && (
          <div className="mb-4 rounded-lg bg-red-900/20 border border-red-800 px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {loading && events.length === 0 && (
          <div className="flex flex-col items-center justify-center py-20 text-slate-600">
            <RefreshCw size={28} className="animate-spin mb-3" />
            <span className="text-sm">Carregando eventos...</span>
          </div>
        )}

        {!loading && events.length === 0 && (
          <div className="flex flex-col items-center justify-center py-20 text-slate-600">
            <CalendarDays size={40} className="mb-4 opacity-30" />
            <p className="text-sm font-medium">Nenhum evento cadastrado</p>
            <p className="text-xs mt-1 mb-4 text-slate-700">
              Crie um evento para aplicar taxas especiais automaticamente
            </p>
            <Button size="sm" onClick={() => setShowCreate(true)} className="gap-1.5">
              <Plus size={14} /> Criar primeiro evento
            </Button>
          </div>
        )}

        {/* Ativos */}
        {activeEvents.length > 0 && (
          <section className="mb-6">
            <h2 className="text-xs font-semibold text-emerald-400 uppercase tracking-wider mb-3 flex items-center gap-2">
              <Zap size={12} />
              Ativos ({activeEvents.length})
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
              {activeEvents.map(ev => (
                <div key={ev.id} className="relative">
                  <EventCard
                    event={ev}
                    onCancel={() => setCancelTarget({ id: ev.id, name: ev.name })}
                  />
                  {/* Botão force-end */}
                  <div className="mt-1.5 flex justify-end">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleForceEnd(ev.id)}
                      disabled={actionLoading === ev.id}
                      className="text-xs text-slate-500 hover:text-red-400 gap-1"
                    >
                      <StopCircle size={11} />
                      {actionLoading === ev.id ? 'Encerrando...' : 'Encerrar agora'}
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* Agendados */}
        {scheduledEvents.length > 0 && (
          <section className="mb-6">
            <h2 className="text-xs font-semibold text-ark-400 uppercase tracking-wider mb-3 flex items-center gap-2">
              <CalendarDays size={12} />
              Agendados ({scheduledEvents.length})
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
              {scheduledEvents.map(ev => (
                <div key={ev.id} className="relative">
                  <EventCard
                    event={ev}
                    onCancel={() => setCancelTarget({ id: ev.id, name: ev.name })}
                  />
                  {/* Botão force-start */}
                  <div className="mt-1.5 flex justify-end">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleForceStart(ev.id)}
                      disabled={actionLoading === ev.id}
                      className="text-xs text-slate-500 hover:text-emerald-400 gap-1"
                    >
                      <Play size={11} />
                      {actionLoading === ev.id ? 'Iniciando...' : 'Iniciar agora'}
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* Histórico */}
        {pastEvents.length > 0 && (
          <section>
            <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-3">
              Histórico ({pastEvents.length})
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
              {pastEvents.map(ev => (
                <EventCard key={ev.id} event={ev} />
              ))}
            </div>
          </section>
        )}
      </div>

      {/* Dialogs */}
      <CreateEventDialog
        open={showCreate}
        onClose={() => setShowCreate(false)}
        onSave={handleCreate}
        serverList={allServers}
      />

      <ConfirmCancelDialog
        open={!!cancelTarget}
        eventName={cancelTarget?.name ?? ''}
        onConfirm={handleCancel}
        onClose={() => setCancelTarget(null)}
      />
    </div>
  )
}
