import { useEffect, useRef, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { listen } from '@tauri-apps/api/event'
import { ArrowLeft, Activity, StopCircle, Trash2 } from 'lucide-react'
import { useServerStore } from '../stores/serverStore'
import { startLogWatcher, stopLogWatcher } from '../utils/tauri'
import { Button, Input, Card } from '../components/ui'
import { cn } from '../utils/helpers'
import type { LogLine, LogLevel } from '../types'
import toast from 'react-hot-toast'

const LEVEL_COLORS: Record<LogLevel, string> = {
  Info:    'text-slate-300',
  Warning: 'text-yellow-400',
  Error:   'text-red-400',
  Debug:   'text-slate-500',
}

export default function LogsConsole() {
  const { serverId } = useParams<{ serverId: string }>()
  const navigate = useNavigate()
  const { servers } = useServerStore()
  const server = servers.find(s => s.id === Number(serverId))

  const [lines, setLines] = useState<LogLine[]>([])
  const [watching, setWatching] = useState(false)
  const [filter, setFilter] = useState<LogLevel | 'all'>('all')
  const [search, setSearch] = useState('')
  const bottomRef = useRef<HTMLDivElement>(null)
  const unlistenRef = useRef<(() => void) | null>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'auto' })
  }, [lines])

  useEffect(() => {
    return () => { unlistenRef.current?.() }
  }, [])

  const handleStart = async () => {
    if (!server) return
    try {
      unlistenRef.current = await listen<LogLine>('log:line', e => {
        setLines(prev => [...prev.slice(-2000), e.payload])
      })
      await startLogWatcher(server.id, server.installDir)
      setWatching(true)
    } catch (e) {
      toast.error(String(e))
    }
  }

  const handleStop = async () => {
    if (!server) return
    unlistenRef.current?.()
    unlistenRef.current = null
    await stopLogWatcher(server.id).catch(() => {})
    setWatching(false)
  }

  const filtered = lines.filter(l => {
    if (filter !== 'all' && l.level !== filter) return false
    if (search && !l.message.toLowerCase().includes(search.toLowerCase())) return false
    return true
  })

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
    <div className="p-6 flex flex-col gap-3 h-full">
      {/* Header */}
      <div className="flex items-center justify-between shrink-0 flex-wrap gap-2">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
            <ArrowLeft size={14} />
          </Button>
          <div>
            <h1 className="text-lg font-semibold text-slate-100">Logs</h1>
            <p className="text-xs text-slate-500">{server.name}</p>
          </div>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          {/* Filtro de nível */}
          {(['all', 'Info', 'Warning', 'Error', 'Debug'] as const).map(l => (
            <Button
              key={l}
              size="sm"
              variant={filter === l ? 'primary' : 'ghost'}
              onClick={() => setFilter(l)}
            >
              {l === 'all' ? 'Todos' : l}
            </Button>
          ))}
          {watching ? (
            <Button size="sm" variant="danger" onClick={handleStop}>
              <StopCircle size={13} /> Parar
            </Button>
          ) : (
            <Button size="sm" onClick={handleStart}>
              <Activity size={13} /> Monitorar
            </Button>
          )}
          <Button size="sm" variant="ghost" onClick={() => setLines([])}>
            <Trash2 size={13} />
          </Button>
        </div>
      </div>

      {/* Busca */}
      <div className="shrink-0">
        <Input
          placeholder="Buscar nos logs..."
          value={search}
          onChange={e => setSearch(e.target.value)}
        />
      </div>

      {/* Log */}
      <Card noPad className="flex-1 overflow-y-auto p-3 font-mono text-xs min-h-0">
        {filtered.length === 0 ? (
          <p className="text-slate-600 italic">
            {watching ? 'Aguardando logs...' : 'Clique em "Monitorar" para iniciar.'}
          </p>
        ) : (
          filtered.map((line, i) => (
            <div key={i} className={cn('leading-relaxed flex gap-2', LEVEL_COLORS[line.level])}>
              <span className="text-slate-600 shrink-0 select-none">{line.timestamp.slice(11, 19)}</span>
              <span className={cn('shrink-0 w-14 uppercase text-[10px] pt-0.5', LEVEL_COLORS[line.level])}>
                [{line.level}]
              </span>
              <span className="break-all">{line.message}</span>
            </div>
          ))
        )}
        <div ref={bottomRef} />
      </Card>

      <div className="shrink-0 flex items-center justify-between text-xs text-slate-600">
        <span>{filtered.length} linha(s)</span>
        {watching && (
          <span className="flex items-center gap-1 text-emerald-500">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
            Monitorando
          </span>
        )}
      </div>
    </div>
  )
}

