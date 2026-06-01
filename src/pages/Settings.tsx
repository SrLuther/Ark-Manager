import { useEffect, useState } from 'react'
import { Settings as SettingsIcon, Save, RefreshCw, Send, CheckCircle, Database, FlaskConical } from 'lucide-react'
import { Button, Input, Card, CardHeader, CardTitle } from '../components/ui'
import { getDiscordConfig, saveDiscordConfig, testDiscordWebhook, getDatabaseUrl, saveDatabaseUrl, testDatabaseConnection } from '../utils/tauri'
import toast from 'react-hot-toast'

interface AppSettings {
  steamcmdDir: string
  backupDir: string
  syncIntervalMinutes: string
}

const STORAGE_KEY = 'ark_manager_settings'

const DISCORD_EVENTS = [
  { key: 'server_start',    label: 'Servidor iniciado' },
  { key: 'server_stop',     label: 'Servidor parado' },
  { key: 'server_crash',    label: 'Servidor travado' },
  { key: 'backup_done',     label: 'Backup concluído' },
  { key: 'sync_conflict',   label: 'Conflito de sincronização' },
  { key: 'player_join',     label: 'Jogador entrou' },
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

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULTS)
  const [saving, setSaving] = useState(false)
  // Database state
  const [dbUrl, setDbUrl] = useState('')
  const [savingDb, setSavingDb] = useState(false)
  const [testingDb, setTestingDb] = useState(false)
  // Discord state
  const [discordWebhook, setDiscordWebhook] = useState('')
  const [discordEvents, setDiscordEvents] = useState<string[]>([])
  const [savingDiscord, setSavingDiscord] = useState(false)
  const [testingWebhook, setTestingWebhook] = useState(false)

  useEffect(() => {
    setSettings(loadSettings())
    getDatabaseUrl().then(url => {
      if (url) setDbUrl(url)
    }).catch(() => {})
    getDiscordConfig().then(cfg => {
      if (cfg) {
        setDiscordWebhook(cfg.webhookUrl)
        setDiscordEvents(cfg.enabledEvents)
      }
    }).catch(() => {})
  }, [])

  const handleSaveDb = async () => {
    if (!dbUrl.trim()) {
      toast.error('Informe a URL de conexão')
      return
    }
    setSavingDb(true)
    try {
      await saveDatabaseUrl(dbUrl.trim())
      toast.success('Configuração salva! Reinicie o aplicativo para conectar.')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSavingDb(false)
    }
  }

  const handleTestDb = async () => {
    if (!dbUrl.trim()) {
      toast.error('Informe a URL de conexão')
      return
    }
    setTestingDb(true)
    try {
      await testDatabaseConnection(dbUrl.trim())
      toast.success('Conexão com MySQL bem-sucedida!')
    } catch (e) {
      toast.error(`Falha: ${String(e)}`)
    } finally {
      setTestingDb(false)
    }
  }

  const toggleDiscordEvent = (key: string) => {
    setDiscordEvents(prev =>
      prev.includes(key) ? prev.filter(e => e !== key) : [...prev, key]
    )
  }

  const handleSaveDiscord = async () => {
    setSavingDiscord(true)
    try {
      await saveDiscordConfig(discordWebhook, discordEvents)
      toast.success('Configuração Discord salva')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setSavingDiscord(false)
    }
  }

  const handleTestWebhook = async () => {
    if (!discordWebhook) return
    setTestingWebhook(true)
    try {
      await testDiscordWebhook(discordWebhook)
      toast.success('Mensagem de teste enviada!')
    } catch (e) {
      toast.error(String(e))
    } finally {
      setTestingWebhook(false)
    }
  }

  const set = <K extends keyof AppSettings>(k: K, v: string) =>
    setSettings(prev => ({ ...prev, [k]: v }))

  const handleSave = async () => {
    setSaving(true)
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
      toast.success('Configurações salvas')
    } finally {
      setSaving(false)
    }
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

      {/* Banco de Dados */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Database size={14} className="text-ark-400" /> Banco de Dados (MySQL)
          </CardTitle>
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="secondary"
              loading={testingDb}
              disabled={!dbUrl}
              onClick={handleTestDb}
              title="Testar conexão"
            >
              <FlaskConical size={12} /> Testar
            </Button>
            <Button size="sm" loading={savingDb} onClick={handleSaveDb}>
              <Save size={12} /> Salvar
            </Button>
          </div>
        </CardHeader>
        <div className="space-y-3">
          <Input
            label="DATABASE_URL"
            value={dbUrl}
            onChange={e => setDbUrl(e.target.value)}
            placeholder="mysql://usuario:senha@localhost:3306/ark_manager"
            hint="URL de conexão MySQL. Reinicie o app após salvar."
          />
          <p className="text-xs text-slate-500">
            Formato: <code className="text-slate-400">mysql://usuario:senha@host:porta/banco</code><br />
            O banco de dados e as tabelas serão criados automaticamente.
          </p>
        </div>
      </Card>

      {/* SteamCMD */}
      <Card>
        <CardHeader><CardTitle>SteamCMD</CardTitle></CardHeader>
        <div className="space-y-3">
          <Input
            label="Pasta do SteamCMD"
            value={settings.steamcmdDir}
            onChange={e => set('steamcmdDir', e.target.value)}
            hint="Onde o steamcmd.exe está instalado"
          />
        </div>
      </Card>

      {/* Backups */}
      <Card>
        <CardHeader><CardTitle>Backups</CardTitle></CardHeader>
        <div className="space-y-3">
          <Input
            label="Pasta padrão de backups"
            value={settings.backupDir}
            onChange={e => set('backupDir', e.target.value)}
            hint="Destino padrão para novos backups"
          />
        </div>
      </Card>

      {/* Integração Discord */}
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
            <Button
              variant="secondary"
              size="sm"
              loading={testingWebhook}
              disabled={!discordWebhook}
              onClick={handleTestWebhook}
              title="Enviar mensagem de teste"
            >
              <Send size={13} /> Testar
            </Button>
          </div>

          {/* Eventos habilitados */}
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-medium text-slate-300">Notificar ao ocorrer</label>
            <div className="grid grid-cols-2 gap-x-4 gap-y-2 mt-1">
              {DISCORD_EVENTS.map(({ key, label }) => (
                <label
                  key={key}
                  className="flex items-center gap-2 cursor-pointer group"
                >
                  <div
                    className={`w-4 h-4 rounded border flex items-center justify-center transition-colors ${
                      discordEvents.includes(key)
                        ? 'bg-ark-600 border-ark-500'
                        : 'bg-surface-800 border-surface-600'
                    }`}
                    onClick={() => toggleDiscordEvent(key)}
                  >
                    {discordEvents.includes(key) && (
                      <CheckCircle size={10} className="text-white" />
                    )}
                  </div>
                  <span
                    className="text-xs text-slate-400 group-hover:text-slate-300"
                    onClick={() => toggleDiscordEvent(key)}
                  >
                    {label}
                  </span>
                </label>
              ))}
            </div>
          </div>
        </div>
      </Card>

      {/* Monitoramento */}
      <Card>
        <CardHeader><CardTitle>Monitoramento</CardTitle></CardHeader>
        <div className="space-y-3">
          <Input
            label="Intervalo de sincronização (minutos)"
            type="number"
            min="1"
            max="60"
            value={settings.syncIntervalMinutes}
            onChange={e => set('syncIntervalMinutes', e.target.value)}
            hint="Com qual frequência atualizar status dos servidores"
          />
        </div>
      </Card>

      <p className="text-xs text-slate-600">
        As configurações são salvas localmente neste computador.
      </p>
    </div>
  )
}

