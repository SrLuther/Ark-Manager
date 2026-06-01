import { useState, useRef, useEffect } from 'react'
import { Download, FolderOpen, CheckCircle, XCircle, Loader2 } from 'lucide-react'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useInstallStore } from '../../stores/installStore'
import { isSteamcmdInstalled } from '../../utils/tauri'
import { Button, Input, Modal } from '../ui'

type Step = 'setup' | 'installing' | 'done' | 'error'

export interface InstallServerDialogProps {
  open: boolean
  onClose: () => void
  onComplete: () => void
}

export function InstallServerDialog({ open, onClose, onComplete }: InstallServerDialogProps) {
  const { installing, progress, error, installSteamcmd, installServer, clearProgress } =
    useInstallStore()

  const [step, setStep] = useState<Step>('setup')
  const [steamcmdDir, setSteamcmdDir] = useState('')
  const [installDir, setInstallDir] = useState('')
  const [steamcmdReady, setSteamcmdReady] = useState<boolean | null>(null)
  const logRef = useRef<HTMLDivElement>(null)

  // Auto-scroll no log
  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight
    }
  }, [progress])

  // Reset ao abrir
  useEffect(() => {
    if (open) {
      setStep('setup')
      setSteamcmdReady(null)
      clearProgress()
    }
  }, [open, clearProgress])

  // Detectar se SteamCMD já está instalado ao mudar o dir
  useEffect(() => {
    if (!steamcmdDir) { setSteamcmdReady(null); return }
    const t = setTimeout(async () => {
      try {
        const ready = await isSteamcmdInstalled(steamcmdDir)
        setSteamcmdReady(ready)
      } catch {
        setSteamcmdReady(false)
      }
    }, 400)
    return () => clearTimeout(t)
  }, [steamcmdDir])

  const pickDir = async (setter: (v: string) => void) => {
    const selected = await openDialog({ directory: true, multiple: false })
    if (typeof selected === 'string') setter(selected)
  }

  const handleInstall = async () => {
    if (!steamcmdDir || !installDir) return
    setStep('installing')
    try {
      if (!steamcmdReady) {
        await installSteamcmd(steamcmdDir)
      }
      await installServer(steamcmdDir, installDir)
      setStep('done')
    } catch {
      setStep('error')
    }
  }

  const handleClose = () => {
    if (!installing) onClose()
  }

  return (
    <Modal
      open={open}
      onClose={handleClose}
      title="Instalar Servidor ARK"
      size="lg"
    >
      <div className="p-5 flex flex-col gap-5">

        {/* Etapa: configuração */}
        {(step === 'setup') && (
          <>
            <p className="text-xs text-slate-400">
              Informe os diretórios para o SteamCMD e para o servidor ARK. O ARK Manager
              fará o download automático via SteamCMD (App ID 376030).
            </p>

            {/* SteamCMD dir */}
            <div className="flex items-end gap-2">
              <Input
                label="Diretório do SteamCMD"
                value={steamcmdDir}
                onChange={(e) => setSteamcmdDir(e.target.value)}
                placeholder="Ex: C:\steamcmd"
                hint={
                  steamcmdReady === true
                    ? 'SteamCMD detectado — instalação será pulada'
                    : steamcmdReady === false
                    ? 'SteamCMD não encontrado — será instalado automaticamente'
                    : undefined
                }
                className="flex-1"
              />
              <Button
                variant="secondary"
                size="sm"
                className="mb-0.5"
                onClick={() => pickDir(setSteamcmdDir)}
              >
                <FolderOpen size={13} />
              </Button>
            </div>

            {/* Install dir */}
            <div className="flex items-end gap-2">
              <Input
                label="Diretório de instalação do servidor"
                value={installDir}
                onChange={(e) => setInstallDir(e.target.value)}
                placeholder="Ex: C:\ark-server"
                className="flex-1"
              />
              <Button
                variant="secondary"
                size="sm"
                className="mb-0.5"
                onClick={() => pickDir(setInstallDir)}
              >
                <FolderOpen size={13} />
              </Button>
            </div>

            <div className="flex justify-end gap-2 pt-1">
              <Button variant="secondary" onClick={handleClose}>
                Cancelar
              </Button>
              <Button
                disabled={!steamcmdDir || !installDir}
                onClick={handleInstall}
              >
                <Download size={14} />
                Iniciar instalação
              </Button>
            </div>
          </>
        )}

        {/* Etapa: instalando */}
        {step === 'installing' && (
          <>
            <div className="flex items-center gap-2 text-sm text-slate-300">
              <Loader2 size={14} className="animate-spin text-ark-400" />
              Baixando via SteamCMD… isso pode levar vários minutos.
            </div>
            <div
              ref={logRef}
              className="h-56 overflow-y-auto bg-surface-900 rounded-lg p-3 font-mono text-xs text-slate-400 space-y-0.5"
            >
              {progress.map((line, i) => (
                <div key={i}>{line}</div>
              ))}
            </div>
          </>
        )}

        {/* Etapa: concluído */}
        {step === 'done' && (
          <div className="flex flex-col items-center gap-4 py-4">
            <CheckCircle size={40} className="text-emerald-400" />
            <p className="text-sm text-slate-200 text-center">
              Servidor instalado com sucesso!<br />
              Agora cadastre-o na página <strong>Servidores</strong>.
            </p>
            <Button onClick={() => { onComplete(); onClose() }}>
              Concluir
            </Button>
          </div>
        )}

        {/* Etapa: erro */}
        {step === 'error' && (
          <div className="flex flex-col items-center gap-4 py-4">
            <XCircle size={40} className="text-red-400" />
            <p className="text-sm text-red-300 text-center">
              {error ?? 'Falha durante a instalação.'}
            </p>
            <div className="flex gap-2">
              <Button variant="secondary" onClick={() => setStep('setup')}>
                Tentar novamente
              </Button>
              <Button variant="danger" onClick={handleClose}>
                Fechar
              </Button>
            </div>
          </div>
        )}
      </div>
    </Modal>
  )
}
