import { useEffect, useRef, useState } from 'react'
import {
  Settings as SettingsIcon, Save, RefreshCw, Send, CheckCircle, XCircle,
  Loader2, Database, FlaskConical, FolderOpen, Download, Wrench, PackagePlus,
} from 'lucide-react'
import { Button, Input, Card, CardHeader, CardTitle } from '../components/ui'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import {
  getDiscordConfig, saveDiscordConfig, testDiscordWebhook,
  getDatabaseUrl, saveDatabaseUrl, testDatabaseConnection, setupDatabase,
  isSteamcmdInstalled, installSteamcmd,
} from '../utils/tauri'
import toast from 'react-hot-toast'

interface AppSettings {
  steamcmdDir: string
  backupDir: string
  syncIntervalMinutes: string
}

const STORAGE_KEY = 'ark_manager_settings'

const DISCORD_EVENTS = [
  { key: 'server_start',  label: 'Servidor iniciado' },
  { key: 'server_stop',   label: 'Servidor parado' },
  { key: 'server_crash',  label: 'Servidor travado' },
  { key: 'backup_done',   label: 'Backup concluído' },
  { key: 'sync_conflict', label: 'Conflito de sincronização' },
  { key: 'player_join',   label: 'Jogador entrou' },
]

const DEFAULTS: AppSettings = {
  steamcmdDir: 'C:\\steamcmd',
  backupDir: 'C:\\ARK\\Backups',
  syncIntervalMinutes: '5',
}

function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? { ...DEFAULTS, ...JSON.parse(raw) } : { ...DEFAULTS }
  } catch {
    return { ...DEFAULTS }
  }
}

function buildUrl(host: string, port: string, user: string, pass: string, db: string) {
  return `mysql://${user}:${pass}@${host}:${port}/${db}`
}

function parseUrl(url: string) {
  try {
    const m = url.match(/^mysql:\/\/([^:]+):([^@]*)@([^:]+):(\d+)\/(.+)$/)
    if (!m) return null
    return { user: m[1], pass: m[2], host: m[3], port: m[4], db: m[5].split('?')[0] }
  } catch {
    return null
  }
}

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULTS)
  const [saving, setSaving] = useState(false)

  // Database
  const [dbHost, setDbHost] = useState('localhost')
  const [dbPort, setDbPort] = useState('3306')
  const [dbUser, setDbUser] = useState('root')
  const [dbPass, setDbPass] = useState('')
  const [dbName, setDbName] = useState('ark_manager')
  const [savingDb, setSavingDb]     = useState(false)
  const [testingDb, setTestingDb]   = useState(false)
  const [creatingDb, setCreatingDb] = useState(false)

  // SteamCMD
  const [steamStatus, setSteamStatus] = useState<'unknown' | 'installed' | 'missing'>('unknown')
  const [installingCMD, setInstallingCMD] = useState(false)
  const [cmdLog, setCmdLog] = useState<string[]>([])
  const cmdLogRef = useRef<HTMLDivElement>(null)

  // Discord
  const [discordWebhook, setDiscordWebhook] = useState('')
  const [discordEvents, setDiscordEvents]   = useState<string[]>([])
  const [savingDiscord, setSavingDiscord]   = useState(false)
  const [testingWebhook, setTestingWebhook] = useState(false)

  useEffect(() => {
    setSettings(loadSettings())
    getDatabaseUrl().then(url => {
      if (!url) return
      const p = parseUrl(url)
      if (p) { setDbHost(p.host); setDbPort(p.port); setDbUser(p.user); setDbPass(p.pass); setDbName(p.db) }
    }).catch(() => {})
    getDiscordConfig().then(cfg => {
      if (cfg) { setDiscordWebhook(cfg.webhookUrl); setDiscordEvents(cfg.enabledEvents) }
    }).catch(() => {})
  }, [])

  useEffect(() => {
    if (cmdLogRef.current) cmdLogRef.current.scrollTop = cmdLogRef.current.scrollHeight
  }, [cmdLog])

  useEffect(() => {
    if (!settings.steamcmdDir) { setSteamStatus('unknown'); return }
    setSteamStatus('unknown')
    const t = setTimeout(async () => {
      try {
        const ok = await isSteamcmdInstalled(settings.steamcmdDir)
        setSteamStatus(ok ? 'installed' : 'missing')
      } catch { setSteamStatus('missing') }
    }, 500)
    return () => clearTimeout(t)
  }, [settings.steamcmdDir])

  const currentUrl = () => buildUrl(dbHost, dbPort, dbUser, dbPass, dbName)

  const handleSaveDb = async () => {
    setSavingDb(true)
    try {
      await saveDatabaseUrl(currentUrl())
      toast.success('Configuração salva! Reinicie o aplicativo para conectar.')
    } catch (e) { toast.error(String(e)) }
    finally { setSavingDb(false) }
  }

  const handleTestDb = async () => {
    setTestingDb(true)
    try {
      await testDatabaseConnection(currentUrl())
      toast.success('Conexão com MySQL bem-sucedida!')
    } catch (e) { toast.error(String(e)) }
    finally { setTestingDb(false) }
  }

  const handleCreateDb = async () => {
    setCreatingDb(true)
    try {
      const msg = await setupDatabase(currentUrl())
      await saveDatabaseUrl(currentUrl())
      toast.success(msg)
    } catch (e) { toast.error(String(e)) }
    finally { setCreatingDb(false) }
  }

  const handleImportDb = async () => {
    setTestingDb(true)
    try {
      await testDatabaseConnection(currentUrl())
      await saveDatabaseUrl(currentUrl())
      toast.success('Banco importado! As tabelas necessárias serão criadas automaticamente ao reiniciar.')
    } catch (e) { toast.error(`Banco não acessível: ${String(e)}`) }
    finally { setTestingDb(false) }
  }

  const handleInstallSteamCMD = async (force = false) => {
    if (!force && steamStatus === 'installed') {
      if (!confirm('SteamCMD já está instalado. Reinstalar mesmo assim?')) return
    }
    setInstallingCMD(true)
    setCmdLog([])
    const unlisten = await listen<string>('install:output', e => {
      setCmdLog(prev => [...prev, e.payload])
    })
    try {
      await installSteamcmd(settings.steamcmdDir)
      setSteamStatus('installed')
      toast.success('SteamCMD instalado com sucesso!')
    } catch (e) {
      toast.error(String(e))
    } finally {
      unlisten()
      setInstallingCMD(false)
    }
  }

  const toggleDiscordEvent = (key: string) =>
    setDiscordEvents(prev => prev.includes(key) ? prev.filter(e => e !== key) : [...prev, key])

  const handleSaveDiscord = async () => {
    setSavingDiscord(true)
    try {
      await saveDiscordConfig(discordWebhook, discordEvents)
      toast.success('Configuração Discord salva')
    } catch (e) { toast.error(String(e)) }
    finally { setSavingDiscord(false) }
  }

  const handleTestWebhook = async () => {
    if (!discordWebhook) return
    setTestingWebhook(true)
    try {
      await testDiscordWebhook(discordWebhook)
      toast.success('Mensagem de teste enviada!')
    } catch (e) { toast.error(String(e)) }
    finally { setTestingWebhook(false) }
  }

  const set = <K extends keyof AppSettings>(k: K, v: string) =>
    setSettings(prev => ({ ...prev, [k]: v }))

  const pickDir = async (field: keyof AppSettings) => {
    const selected = await openDialog({ directory: true, multiple: false })
    if (typeof selected === 'string') set(field, selected)
  }

  const handleSave = async () => {
    setSaving(true)
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
      await Promise.allSettled([
        saveDatabaseUrl(currentUrl()),
        saveDiscordConfig(discordWebhook, discordEvents),
      ])
      toast.success('Configurações salvas')
    } finally { setSaving(false) }
  }

  const handleReset = () => {
    if (!confirm('Restaurar configurações padrão?')) return
    setSettings({ ...DEFAULTS })
    localStorage.removeItem(STORAGE_KEY)
    toast.success('Configurações restauradas')
  }

  return (
    <div className="p-6 space-y-4 h-full overflow-auto max-w-2xl">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold text-slate-100 flex items-center gap-2">
          <SettingsIcon size={16} className="text-ark-400" /> Configurações
        </h1>
        <div className="flex gap-2">
          <Button size="sm" variant="ghost" onClick={handleReset}>
            <RefreshCw size={13} /> Padrões
          </Button>
          <Button size="sm" loading={saving} onClick={handleSave}>
            <Save size={13} /> Salvar
          </Button>
        </div>
      </div>

      {/* ── Banco de Dados ─────────────────────────────── */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Database size={14} className="text-ark-400" /> Banco de Dados (MySQL / MariaDB)
          </CardTitle>
        </CardHeader>
        <div className="space-y-3">
          <div className="grid grid-cols-3 gap-2">
            <div className="col-span-2">
              <Input label="Host" value={dbHost} onChange={e => setDbHost(e.target.value)} placeholder="localhost" />
            </div>
            <Input label="Porta" value={dbPort} onChange={e => setDbPort(e.target.value)} placeholder="3306" />
          </div>
          <div className="grid grid-cols-2 gap-2">
            <Input label="Usuário" value={dbUser} onChange={e => setDbUser(e.target.value)} placeholder="root" />
            <Input label="Senha" type="password" value={dbPass} onChange={e => setDbPass(e.target.value)} placeholder="••••••••" />
          </div>
          <Input label="Nome do banco" value={dbName} onChange={e => setDbName(e.target.value)} placeholder="ark_manager" />

          <p className="text-xs text-slate-500 font-mono break-all bg-surface-800 rounded px-2 py-1.5">
            {currentUrl()}
          </p>

          <div className="flex flex-wrap gap-2 pt-1">
            <Button size="sm" loading={creatingDb} onClick={handleCreateDb}
              title="Cria o banco se não existir e configura todas as tabelas">
              <PackagePlus size={13} /> Criar banco
            </Button>
            <Button size="sm" variant="secondary" loading={testingDb && !creatingDb} onClick={handleImportDb}
              title="Conecta a um banco existente e salva a configuração">
              <Database size={13} /> Importar existente
            </Button>
            <Button size="sm" variant="ghost" loading={testingDb} onClick={handleTestDb}>
              <FlaskConical size={12} /> Testar conexão
            </Button>
            <Button size="sm" variant="ghost" loading={savingDb} onClick={handleSaveDb}>
              <Save size={12} /> Salvar URL
            </Button>
          </div>

          <p className="text-xs text-slate-500">
            <strong className="text-slate-400">Criar banco</strong> — cria o schema e todas as tabelas do zero.<br />
            <strong className="text-slate-400">Importar existente</strong> — usa um banco já existente (tabelas faltantes criadas automaticamente).<br />
            Reinicie o app após salvar.
          </p>
        </div>
      </Card>

      {/* ── SteamCMD ──────────────────────────────────── */}
      <Card>
        <CardHeader>
          <CardTitle>SteamCMD</CardTitle>
          {steamStatus === 'installed' && (
            <span className="flex items-center gap-1 text-xs text-emerald-400">
              <CheckCircle size={12} /> Instalado
            </span>
          )}
          {steamStatus === 'missing' && (
            <span className="flex items-center gap-1 text-xs text-amber-400">
              <XCircle size={12} /> Não encontrado
            </span>
          )}
        </CardHeader>
        <div className="space-y-3">
          <div className="flex items-end gap-2">
            <div className="flex-1">
              <Input
                label="Pasta do SteamCMD"
                value={settings.steamcmdDir}
                onChange={e => set('steamcmdDir', e.target.value)}
                hint="Onde o steamcmd.exe será instalado"
              />
            </div>
            <button type="button" onClick={() => pickDir('steamcmdDir')}
              className="mb-0.5 p-2 rounded-lg bg-surface-700 hover:bg-surface-600 border border-surface-600 text-slate-300 hover:text-white transition-colors"
              title="Escolher pasta">
              <FolderOpen size={15} />
            </button>
          </div>

          <div className="flex gap-2">
            <Button size="sm" loading={installingCMD} disabled={installingCMD}
              onClick={() => handleInstallSteamCMD(false)}>
              {installingCMD
                ? <Loader2 size={13} className="animate-spin" />
                : <Download size={13} />}
              {steamStatus === 'installed' ? 'Reinstalar' : 'Instalar'}
            </Button>
            {steamStatus === 'installed' && (
              <Button size="sm" variant="ghost" loading={installingCMD} disabled={installingCMD}
                onClick={() => handleInstallSteamCMD(true)}>
                <Wrench size={13} /> Reparar
              </Button>
            )}
          </div>

          {cmdLog.length > 0 && (
            <div ref={cmdLogRef}
              className="h-32 overflow-y-auto bg-surface-900 rounded-lg p-2 font-mono text-xs text-slate-400 space-y-0.5">
              {cmdLog.map((line, i) => <div key={i}>{line}</div>)}
            </div>
          )}
        </div>
      </Card>

      {/* ── Backups ──────────────────────────────────── */}
      <Card>
        <CardHeader><CardTitle>Backups</CardTitle></CardHeader>
        <div className="space-y-3">
          <div className="flex items-end gap-2">
            <div className="flex-1">
              <Input
                label="Pasta padrão de backups"
                value={settings.backupDir}
                onChange={e => set('backupDir', e.target.value)}
                hint="Destino padrão para novos backups"
              />
            </div>
            <button type="button" onClick={() => pickDir('backupDir')}
              className="mb-0.5 p-2 rounded-lg bg-surface-700 hover:bg-surface-600 border border-surface-600 text-slate-300 hover:text-white transition-colors"
              title="Escolher pasta">
              <FolderOpen size={15} />
            </button>
          </div>
        </div>
      </Card>

      {/* ── Discord ───────────────────────────────────── */}
      <Card>
        <CardHeader>
          <CardTitle>Discord</CardTitle>
          <Button size="sm" loading={savingDiscord} onClick={handleSaveDiscord}>
            <Save size={12} /> Salvar
          </Button>
        </CardHeader>
        <div className="space-y-4">
          <div className="flex gap-2 items-end">
            <div className="flex-1">
              <Input
                label="Webhook URL"
                value={discordWebhook}
                onChange={e => setDiscordWebhook(e.target.value)}
                hint="URL do Incoming Webhook do Discord (opcional)"
                placeholder="https://discord.com/api/webhooks/..."
              />
            </div>
            <Button variant="secondary" size="sm" loading={testingWebhook}
              disabled={!discordWebhook} onClick={handleTestWebhook}>
              <Send size={13} /> Testar
            </Button>
          </div>

          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-slate-300">Notificar ao ocorrer</label>
            <div className="grid grid-cols-2 gap-x-4 gap-y-2 mt-1">
              {DISCORD_EVENTS.map(({ key, label }) => (
                <label key={key} className="flex items-center gap-2 cursor-pointer group">
                  <div
                    className={`w-4 h-4 rounded border flex items-center justify-center transition-colors ${
                      discordEvents.includes(key) ? 'bg-ark-600 border-ark-500' : 'bg-surface-800 border-surface-600'
                    }`}
                    onClick={() => toggleDiscordEvent(key)}
                  >
                    {discordEvents.includes(key) && <CheckCircle size={10} className="text-white" />}
                  </div>
                  <span className="text-xs text-slate-400 group-hover:text-slate-300"
                    onClick={() => toggleDiscordEvent(key)}>
                    {label}
                  </span>
                </label>
              ))}
            </div>
          </div>
        </div>
      </Card>

      {/* ── Monitoramento ─────────────────────────────── */}
      <Card>
        <CardHeader><CardTitle>Monitoramento</CardTitle></CardHeader>
        <div className="space-y-3">
          <Input
            label="Intervalo de sincronização (minutos)"
            type="number" min="1" max="60"
            value={settings.syncIntervalMinutes}
            onChange={e => set('syncIntervalMinutes', e.target.value)}
            hint="Com qual frequência atualizar status dos servidores"
          />
        </div>
      </Card>

      <p className="text-xs text-slate-600">As configurações são salvas localmente neste computador.</p>
    </div>
  )
}
