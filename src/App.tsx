import { BrowserRouter, Routes, Route, Navigate, useNavigate } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import { useEffect, useState, Component, type ReactNode, type ErrorInfo } from 'react'
import { listen } from '@tauri-apps/api/event'
import { AlertTriangle, X, Bug } from 'lucide-react'
import Layout from './components/layout/Layout'
import Dashboard from './pages/Dashboard'
import ServerManager from './pages/ServerManager'
import ConfigEditor from './pages/ConfigEditor'
import RconConsole from './pages/RconConsole'
import LogsConsole from './pages/LogsConsole'
import ModManager from './pages/ModManager'
import ClusterManager from './pages/ClusterManager'
import Backups from './pages/Backups'
import Scheduler from './pages/Scheduler'
import Settings from './pages/Settings'
import Monitoring from './pages/Monitoring'
import SyncManager from './pages/SyncManager'
import SeasonalEvents from './pages/SeasonalEvents'

// ─── Error Boundary ───────────────────────────────────────────────────────────
class ErrorBoundary extends Component<
  { children: ReactNode },
  { error: Error | null }
> {
  constructor(props: { children: ReactNode }) {
    super(props)
    this.state = { error: null }
  }
  static getDerivedStateFromError(error: Error) {
    return { error }
  }
  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('[ARK Manager] Crash:', error, info)
  }
  render() {
    const { error } = this.state
    if (!error) return this.props.children
    return (
      <div className="flex flex-col items-center justify-center h-screen bg-surface-950 p-8 gap-6">
        <div className="flex items-center gap-3 text-red-400">
          <Bug size={28} />
          <span className="text-lg font-semibold">Erro ao inicializar o Ark Manager</span>
        </div>
        <div className="w-full max-w-2xl bg-surface-900 border border-red-900/60 rounded-lg p-4 overflow-auto max-h-72">
          <p className="text-sm font-mono text-red-300 break-all whitespace-pre-wrap">
            {error.name}: {error.message}
          </p>
          {error.stack && (
            <p className="text-xs font-mono text-slate-500 mt-3 whitespace-pre-wrap">{error.stack}</p>
          )}
        </div>
        <button
          className="px-4 py-2 rounded-lg bg-ark-700 hover:bg-ark-600 text-white text-sm"
          onClick={() => window.location.reload()}
        >
          Recarregar
        </button>
      </div>
    )
  }
}

function DbErrorBanner() {
  const [show, setShow] = useState(false)
  const navigate = useNavigate()

  useEffect(() => {
    const unlisten = listen<string>('db:error', () => {
      setShow(true)
    })
    return () => { unlisten.then(fn => fn()) }
  }, [])

  if (!show) return null

  return (
    <div className="fixed top-0 left-0 right-0 z-50 flex items-center gap-3 px-4 py-2.5 bg-amber-900/95 border-b border-amber-700 text-amber-100 text-sm">
      <AlertTriangle size={15} className="shrink-0 text-amber-400" />
      <span className="flex-1">
        Banco de dados não configurado ou inacessível.{' '}
        <button
          className="underline hover:no-underline font-medium"
          onClick={() => { navigate('/settings'); setShow(false) }}
        >
          Abrir Configurações
        </button>{' '}
        para informar a URL de conexão MySQL.
      </span>
      <button onClick={() => setShow(false)} className="shrink-0 hover:text-white">
        <X size={14} />
      </button>
    </div>
  )
}

export default function AppWithErrorBoundary() {
  return (
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  )
}

function App() {
  return (
    <BrowserRouter>
      <DbErrorBanner />
      <Toaster
        position="top-right"
        toastOptions={{
          style: {
            background: '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
          },
        }}
      />
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<Dashboard />} />
          <Route path="servers" element={<ServerManager />} />
          <Route path="config/:serverId" element={<ConfigEditor />} />
          <Route path="rcon/:serverId" element={<RconConsole />} />
          <Route path="logs/:serverId" element={<LogsConsole />} />
          <Route path="mods/:serverId" element={<ModManager />} />
          <Route path="cluster" element={<ClusterManager />} />
          <Route path="backups" element={<Backups />} />
          <Route path="scheduler" element={<Scheduler />} />
          <Route path="monitoring" element={<Monitoring />} />
          <Route path="sync" element={<SyncManager />} />
          <Route path="events" element={<SeasonalEvents />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}
