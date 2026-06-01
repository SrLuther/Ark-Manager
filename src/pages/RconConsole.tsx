import { useEffect, useRef, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Send, Trash2, Wifi, WifiOff, Terminal } from 'lucide-react'
import { useServerStore } from '../stores/serverStore'
import { useRconStore } from '../stores/rconStore'
import { Button, Input, Card } from '../components/ui'
import { cn } from '../utils/helpers'
import toast from 'react-hot-toast'

const QUICK_COMMANDS = [
  { label: 'Salvar mundo', cmd: 'saveworld' },
  { label: 'Listar jogadores', cmd: 'listplayers' },
  { label: 'Destruir dinos selvagens', cmd: 'destroywilddinos' },
  { label: 'Cheats Dino Reset', cmd: 'cheat destroywilddinos' },
]

export default function RconConsole() {
  const { serverId } = useParams<{ serverId: string }>()
  const navigate = useNavigate()
  const { servers } = useServerStore()
  const server = servers.find(s => s.id === Number(serverId))
  const { connected, connecting, history, connect, disconnect, sendCommand, clearHistory } = useRconStore()

  const [input, setInput] = useState('')
  const [broadcast, setBroadcast] = useState('')
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [history])

  const handleConnect = async () => {
    if (!server) return
    try {
      await connect(server.id, '127.0.0.1', server.rconPort, server.rconPassword)
    } catch (e) {
      toast.error('Falha ao conectar RCON: ' + String(e))
    }
  }

  const handleDisconnect = async () => {
    if (!server) return
    await disconnect(server.id)
  }

  const handleSend = async () => {
    if (!server || !input.trim()) return
    await sendCommand(server.id, input.trim())
    setInput('')
  }

  const handleBroadcast = async () => {
    if (!server || !broadcast.trim()) return
    await sendCommand(server.id, `broadcast ${broadcast.trim()}`)
    setBroadcast('')
  }

  const lineColor = (type: string) => {
    if (type === 'command')  return 'text-ark-300'
    if (type === 'response') return 'text-slate-200'
    if (type === 'error')    return 'text-red-400'
    return 'text-slate-500'
  }

  const linePrefix = (type: string) => {
    if (type === 'command')  return '> '
    if (type === 'response') return '  '
    if (type === 'error')    return '! '
    return '# '
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
    <div className="p-6 flex flex-col gap-4 h-full">
      {/* Header */}
      <div className="flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
            <ArrowLeft size={14} />
          </Button>
          <div>
            <h1 className="text-lg font-semibold text-slate-100 flex items-center gap-2">
              <Terminal size={16} className="text-ark-400" /> RCON
            </h1>
            <p className="text-xs text-slate-500">{server.name} · 127.0.0.1:{server.rconPort}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {connected && (
            <span className="flex items-center gap-1 text-xs text-emerald-400">
              <Wifi size={12} /> Conectado
            </span>
          )}
          {!connected && !connecting && (
            <span className="flex items-center gap-1 text-xs text-slate-500">
              <WifiOff size={12} /> Desconectado
            </span>
          )}
          <Button
            size="sm"
            variant={connected ? 'danger' : 'primary'}
            loading={connecting}
            onClick={connected ? handleDisconnect : handleConnect}
          >
            {connected ? 'Desconectar' : 'Conectar'}
          </Button>
          <Button size="sm" variant="ghost" onClick={clearHistory} title="Limpar">
            <Trash2 size={13} />
          </Button>
        </div>
      </div>

      <div className="flex gap-4 flex-1 min-h-0">
        {/* Terminal */}
        <div className="flex flex-col flex-1 min-w-0 gap-2">
          <Card noPad className="flex-1 overflow-y-auto p-3 font-mono text-xs min-h-0">
            {history.length === 0 ? (
              <p className="text-slate-600 italic">Nenhuma saída ainda. Conecte e envie um comando.</p>
            ) : (
              history.map((entry, i) => (
                <div key={i} className={cn('leading-relaxed whitespace-pre-wrap', lineColor(entry.type))}>
                  <span className="text-slate-600 select-none mr-1">
                    {new Date(entry.ts).toLocaleTimeString('pt-BR', { hour12: false })}
                  </span>
                  {linePrefix(entry.type)}{entry.text}
                </div>
              ))
            )}
            <div ref={bottomRef} />
          </Card>

          {/* Input */}
          <div className="flex gap-2 shrink-0">
            <Input
              className="flex-1 font-mono"
              placeholder="Digite um comando RCON..."
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleSend()}
              disabled={!connected}
            />
            <Button onClick={handleSend} disabled={!connected || !input.trim()}>
              <Send size={14} /> Enviar
            </Button>
          </div>
        </div>

        {/* Painel lateral */}
        <div className="w-52 shrink-0 space-y-3">
          <Card>
            <p className="text-xs font-semibold text-slate-400 mb-2">Comandos rápidos</p>
            <div className="space-y-1.5">
              {QUICK_COMMANDS.map(({ label, cmd }) => (
                <Button
                  key={cmd}
                  size="sm"
                  variant="secondary"
                  className="w-full justify-start text-xs"
                  disabled={!connected}
                  onClick={() => sendCommand(server.id, cmd)}
                >
                  {label}
                </Button>
              ))}
            </div>
          </Card>

          <Card>
            <p className="text-xs font-semibold text-slate-400 mb-2">Broadcast</p>
            <Input
              placeholder="Mensagem..."
              value={broadcast}
              onChange={e => setBroadcast(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && handleBroadcast()}
              disabled={!connected}
            />
            <Button
              size="sm"
              className="w-full mt-2"
              disabled={!connected || !broadcast.trim()}
              onClick={handleBroadcast}
            >
              Enviar broadcast
            </Button>
          </Card>
        </div>
      </div>
    </div>
  )
}

