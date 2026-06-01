import { useEffect, useState } from 'react'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { Download, X, RefreshCw, ArrowUpCircle } from 'lucide-react'

type UpdateState =
  | { phase: 'idle' }
  | { phase: 'available'; version: string; notes: string | null | undefined }
  | { phase: 'downloading'; percent: number }
  | { phase: 'ready' }
  | { phase: 'error'; message: string }

export default function UpdateNotifier() {
  const [state, setState] = useState<UpdateState>({ phase: 'idle' })
  const [dismissed, setDismissed] = useState(false)

  // Verifica atualização ao iniciar e a cada 4 horas
  useEffect(() => {
    const run = async () => {
      try {
        const update = await check()
        if (update?.available) {
          setState({
            phase: 'available',
            version: update.version,
            notes: update.body,
          })
        }
      } catch {
        // silencioso — sem conexão ou sem release publicado
      }
    }

    run()
    const interval = setInterval(run, 4 * 60 * 60 * 1000)
    return () => clearInterval(interval)
  }, [])

  const handleInstall = async () => {
    if (state.phase !== 'available') return
    setState({ phase: 'downloading', percent: 0 })

    try {
      const update = await check()
      if (!update?.available) return

      let downloaded = 0
      let total = 0
      await update.downloadAndInstall(event => {
        if (event.event === 'Started') {
          total = event.data.contentLength ?? 0
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength
          if (total > 0) {
            setState(prev =>
              prev.phase === 'downloading'
                ? { phase: 'downloading', percent: Math.round((downloaded / total) * 100) }
                : prev
            )
          }
        } else if (event.event === 'Finished') {
          setState({ phase: 'ready' })
        }
      })
    } catch (e) {
      setState({ phase: 'error', message: String(e) })
    }
  }

  const handleRelaunch = async () => {
    await relaunch()
  }

  if (dismissed || state.phase === 'idle') return null

  return (
    <div className="fixed bottom-4 right-4 z-50 w-80 bg-surface-800 border border-ark-700 rounded-xl shadow-2xl overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 bg-ark-900/60 border-b border-ark-800">
        <div className="flex items-center gap-2 text-ark-300">
          <ArrowUpCircle size={15} />
          <span className="text-sm font-semibold">Atualização disponível</span>
        </div>
        {state.phase !== 'downloading' && state.phase !== 'ready' && (
          <button
            onClick={() => setDismissed(true)}
            className="text-slate-500 hover:text-slate-300 transition-colors"
          >
            <X size={14} />
          </button>
        )}
      </div>

      <div className="px-4 py-3 space-y-3">
        {/* Versão disponível */}
        {state.phase === 'available' && (
          <>
            <p className="text-sm text-slate-300">
              A versão <span className="font-semibold text-ark-400">{state.version}</span> está disponível.
            </p>
            {state.notes && (
              <p className="text-xs text-slate-500 line-clamp-3 bg-surface-900 rounded-lg px-2 py-1.5">
                {state.notes}
              </p>
            )}
            <div className="flex gap-2 pt-1">
              <button
                onClick={() => setDismissed(true)}
                className="flex-1 px-3 py-1.5 rounded-lg text-xs text-slate-400 hover:text-slate-200
                           bg-surface-700 hover:bg-surface-600 transition-colors"
              >
                Depois
              </button>
              <button
                onClick={handleInstall}
                className="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-lg
                           text-xs font-medium text-white bg-ark-600 hover:bg-ark-500 transition-colors"
              >
                <Download size={12} />
                Instalar
              </button>
            </div>
          </>
        )}

        {/* Baixando */}
        {state.phase === 'downloading' && (
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm text-slate-300">
              <RefreshCw size={13} className="animate-spin text-ark-400" />
              <span>Baixando... {state.percent}%</span>
            </div>
            <div className="w-full h-1.5 bg-surface-700 rounded-full overflow-hidden">
              <div
                className="h-full bg-ark-500 transition-all duration-300 rounded-full"
                style={{ width: `${state.percent}%` }}
              />
            </div>
          </div>
        )}

        {/* Pronto para reiniciar */}
        {state.phase === 'ready' && (
          <div className="space-y-3">
            <p className="text-sm text-slate-300">
              Atualização baixada. Reinicie para aplicar.
            </p>
            <button
              onClick={handleRelaunch}
              className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg
                         text-sm font-medium text-white bg-ark-600 hover:bg-ark-500 transition-colors"
            >
              <RefreshCw size={13} />
              Reiniciar agora
            </button>
          </div>
        )}

        {/* Erro */}
        {state.phase === 'error' && (
          <div className="space-y-2">
            <p className="text-xs text-red-400">{state.message}</p>
            <button
              onClick={() => setState({ phase: 'idle' })}
              className="text-xs text-slate-500 hover:text-slate-300"
            >
              Fechar
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
