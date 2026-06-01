import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { Save, ArrowLeft, RefreshCw } from 'lucide-react'
import { useServerStore } from '../stores/serverStore'
import { readGameUserSettings, saveServerConfig } from '../utils/tauri'
import { mapLabel } from '../utils/helpers'
import { Button, Input, Card, CardHeader, CardTitle } from '../components/ui'
import type { ServerConfig } from '../types'
import toast from 'react-hot-toast'

const DEFAULT_CONFIG: ServerConfig = {
  sessionName: '', serverPassword: '', adminPassword: '', maxPlayers: 70,
  rconPort: 32330, rconPassword: '', rconEnabled: true,
  gamePort: 7777, queryPort: 27015,
  xpMultiplier: 1, harvestMultiplier: 1, tameSpeedMultiplier: 1,
  serverPve: false, allowThirdPersonPlayer: true,
  alwaysNotifyPlayerJoined: true, alwaysNotifyPlayerLeft: true,
  serverHardcore: false, enableCrossArk: false,
  clusterId: '', clusterDirOverride: '', activeMods: '',
  mapName: 'TheIsland', serverAutoForceRespawnWildDinosCooldown: 86400,
  enableRcon: true,
}

function Toggle({ label, checked, onChange }: { label: string; checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="flex items-center justify-between py-2 cursor-pointer">
      <span className="text-sm text-slate-300">{label}</span>
      <button
        type="button"
        onClick={() => onChange(!checked)}
        className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${checked ? 'bg-ark-600' : 'bg-surface-600'}`}
      >
        <span className={`inline-block h-3.5 w-3.5 rounded-full bg-white shadow transition-transform ${checked ? 'translate-x-4.5' : 'translate-x-0.5'}`} />
      </button>
    </label>
  )
}

export default function ConfigEditor() {
  const { serverId } = useParams<{ serverId: string }>()
  const navigate = useNavigate()
  const { servers } = useServerStore()
  const server = servers.find(s => s.id === Number(serverId))

  const [config, setConfig] = useState<ServerConfig>(DEFAULT_CONFIG)
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)

  const set = <K extends keyof ServerConfig>(k: K, v: ServerConfig[K]) =>
    setConfig(prev => ({ ...prev, [k]: v }))

  const loadConfig = async () => {
    if (!server) return
    setLoading(true)
    try {
      const raw = await readGameUserSettings(server.installDir) as Record<string, Record<string, string>>
      const ss = raw['SessionSettings'] ?? {}
      const srv = raw['ServerSettings'] ?? {}
      setConfig({
        ...DEFAULT_CONFIG,
        sessionName:              ss['SessionName'] ?? server.name,
        maxPlayers:               parseInt(ss['MaxPlayers'] ?? '70'),
        rconPort:                 parseInt(ss['RCONPort'] ?? '32330'),
        gamePort:                 parseInt(ss['Port'] ?? '7777'),
        queryPort:                parseInt(ss['QueryPort'] ?? '27015'),
        adminPassword:            srv['ServerAdminPassword'] ?? server.adminPassword,
        serverPassword:           srv['ServerPassword'] ?? '',
        rconPassword:             srv['RCONServerGameLogEnabled'] ?? server.rconPassword,
        rconEnabled:              (srv['RCONEnabled'] ?? 'True') === 'True',
        serverPve:                (srv['ServerPVE'] ?? 'False') === 'True',
        allowThirdPersonPlayer:   (srv['AllowThirdPersonPlayer'] ?? 'True') === 'True',
        alwaysNotifyPlayerJoined: (srv['AlwaysNotifyPlayerJoined'] ?? 'True') === 'True',
        alwaysNotifyPlayerLeft:   (srv['AlwaysNotifyPlayerLeft'] ?? 'True') === 'True',
        serverHardcore:           (srv['ServerHardcore'] ?? 'False') === 'True',
        enableCrossArk:           srv['PreventDownloadSurvivors'] === 'False',
        clusterId:                srv['clusterid'] ?? '',
        clusterDirOverride:       srv['ClusterDirOverride'] ?? '',
        activeMods:               ss['ActiveMods'] ?? '',
        mapName:                  server.map,
        xpMultiplier:             parseFloat(srv['XPMultiplier'] ?? '1'),
        harvestMultiplier:        parseFloat(srv['HarvestAmountMultiplier'] ?? '1'),
        tameSpeedMultiplier:      parseFloat(srv['TamingSpeedMultiplier'] ?? '1'),
        serverAutoForceRespawnWildDinosCooldown: parseInt(srv['ServerAutoForceRespawnWildDinosCooldown'] ?? '86400'),
        enableRcon:               (srv['RCONEnabled'] ?? 'True') === 'True',
      })
    } catch (e) {
      toast.error('Falha ao ler configuração: ' + String(e))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { if (server) loadConfig() }, [server?.id])

  const handleSave = async () => {
    if (!server) return
    setSaving(true)
    try {
      await saveServerConfig(server.installDir, config)
      toast.success('Configuração salva com sucesso')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSaving(false)
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
          <Button variant="ghost" size="sm" onClick={() => navigate('/servers')}>
            <ArrowLeft size={14} />
          </Button>
          <div>
            <h1 className="text-lg font-semibold text-slate-100">Configuração</h1>
            <p className="text-xs text-slate-500">{server.name} · {mapLabel(server.map as any)}</p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="secondary" onClick={loadConfig} loading={loading}>
            <RefreshCw size={13} /> Recarregar
          </Button>
          <Button size="sm" loading={saving} onClick={handleSave}>
            <Save size={13} /> Salvar
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Sessão */}
        <Card>
          <CardHeader><CardTitle>Sessão</CardTitle></CardHeader>
          <div className="space-y-3">
            <Input label="Nome da sessão" value={config.sessionName} onChange={e => set('sessionName', e.target.value)} />
            <Input label="Máx. jogadores" type="number" value={config.maxPlayers} onChange={e => set('maxPlayers', +e.target.value)} />
            <Input label="Senha do servidor" type="password" value={config.serverPassword} onChange={e => set('serverPassword', e.target.value)} />
            <Input label="Senha do admin" type="password" value={config.adminPassword} onChange={e => set('adminPassword', e.target.value)} />
          </div>
        </Card>

        {/* Portas */}
        <Card>
          <CardHeader><CardTitle>Portas</CardTitle></CardHeader>
          <div className="space-y-3">
            <Input label="Porta do jogo" type="number" value={config.gamePort} onChange={e => set('gamePort', +e.target.value)} />
            <Input label="Porta Query" type="number" value={config.queryPort} onChange={e => set('queryPort', +e.target.value)} />
            <Input label="Porta RCON" type="number" value={config.rconPort} onChange={e => set('rconPort', +e.target.value)} />
            <Input label="Senha RCON" type="password" value={config.rconPassword} onChange={e => set('rconPassword', e.target.value)} />
          </div>
        </Card>

        {/* Multiplicadores */}
        <Card>
          <CardHeader><CardTitle>Multiplicadores</CardTitle></CardHeader>
          <div className="space-y-3">
            <Input label="XP" type="number" step="0.1" value={config.xpMultiplier} onChange={e => set('xpMultiplier', +e.target.value)} />
            <Input label="Coleta" type="number" step="0.1" value={config.harvestMultiplier} onChange={e => set('harvestMultiplier', +e.target.value)} />
            <Input label="Domesticação" type="number" step="0.1" value={config.tameSpeedMultiplier} onChange={e => set('tameSpeedMultiplier', +e.target.value)} />
          </div>
        </Card>

        {/* Opções do servidor */}
        <Card>
          <CardHeader><CardTitle>Opções</CardTitle></CardHeader>
          <div className="divide-y divide-surface-700">
            <Toggle label="Modo PvE" checked={config.serverPve} onChange={v => set('serverPve', v)} />
            <Toggle label="Terceira pessoa" checked={config.allowThirdPersonPlayer} onChange={v => set('allowThirdPersonPlayer', v)} />
            <Toggle label="Notificar entrada de jogadores" checked={config.alwaysNotifyPlayerJoined} onChange={v => set('alwaysNotifyPlayerJoined', v)} />
            <Toggle label="Notificar saída de jogadores" checked={config.alwaysNotifyPlayerLeft} onChange={v => set('alwaysNotifyPlayerLeft', v)} />
            <Toggle label="Modo Hardcore" checked={config.serverHardcore} onChange={v => set('serverHardcore', v)} />
            <Toggle label="RCON habilitado" checked={config.rconEnabled} onChange={v => set('rconEnabled', v)} />
          </div>
        </Card>

        {/* Cluster */}
        <Card>
          <CardHeader><CardTitle>Cluster Cross-ARK</CardTitle></CardHeader>
          <div className="space-y-3">
            <Toggle label="Cross-ARK habilitado" checked={config.enableCrossArk} onChange={v => set('enableCrossArk', v)} />
            <Input label="Cluster ID" value={config.clusterId} onChange={e => set('clusterId', e.target.value)} disabled={!config.enableCrossArk} />
            <Input label="Pasta do cluster" value={config.clusterDirOverride} onChange={e => set('clusterDirOverride', e.target.value)} disabled={!config.enableCrossArk} />
          </div>
        </Card>

        {/* Mods */}
        <Card>
          <CardHeader><CardTitle>Mods ativos</CardTitle></CardHeader>
          <div className="space-y-2">
            <Input
              label="IDs separados por vírgula"
              value={config.activeMods}
              onChange={e => set('activeMods', e.target.value)}
              hint="Ex: 731604991,889745138"
            />
            <p className="text-xs text-slate-500">Para gerenciamento detalhado, use a página Mods.</p>
          </div>
        </Card>
      </div>
    </div>
  )
}

