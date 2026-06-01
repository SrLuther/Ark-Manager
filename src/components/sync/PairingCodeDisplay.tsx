import { useState, useEffect } from 'react'
import { Copy, RefreshCw, QrCode } from 'lucide-react'
import { Button, Modal } from '../ui'
import toast from 'react-hot-toast'

export interface PairingCodeDisplayProps {
  open: boolean
  onClose: () => void
  onGenerate: () => Promise<string>
}

export function PairingCodeDisplay({ open, onClose, onGenerate }: PairingCodeDisplayProps) {
  const [code, setCode] = useState<string | null>(null)
  const [generating, setGenerating] = useState(false)

  const generate = async () => {
    setGenerating(true)
    try {
      const c = await onGenerate()
      setCode(c)
    } catch (e) {
      toast.error(String(e))
    } finally {
      setGenerating(false)
    }
  }

  useEffect(() => {
    if (open && !code) {
      generate()
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

  const handleCopy = () => {
    if (!code) return
    navigator.clipboard.writeText(code)
    toast.success('Código copiado')
  }

  const handleClose = () => {
    setCode(null)
    onClose()
  }

  return (
    <Modal open={open} onClose={handleClose} title="Código de pareamento" size="sm">
      <div className="p-5 flex flex-col items-center gap-4">
        <div className="p-2 bg-surface-800 rounded-xl">
          <QrCode size={32} className="text-ark-400" />
        </div>
        <p className="text-xs text-slate-400 text-center">
          Mostre este código ao operador do peer remoto para confirmar o pareamento.
          O código expira em 5 minutos.
        </p>

        {code ? (
          <div className="flex items-center gap-3">
            <span className="font-mono text-4xl font-bold tracking-[0.3em] text-ark-300 select-all">
              {code}
            </span>
            <Button variant="ghost" size="sm" onClick={handleCopy} title="Copiar">
              <Copy size={14} />
            </Button>
          </div>
        ) : (
          <div className="h-12 flex items-center justify-center">
            <span className="text-slate-500 text-sm">Gerando...</span>
          </div>
        )}

        <Button variant="secondary" size="sm" loading={generating} onClick={generate}>
          <RefreshCw size={12} /> Novo código
        </Button>
      </div>
    </Modal>
  )
}
