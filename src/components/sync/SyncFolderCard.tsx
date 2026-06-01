import { FolderSync, AlertTriangle, RefreshCw, Clock } from 'lucide-react'
import { formatRelative, formatBytes, cn } from '../../utils/helpers'
import { SyncStatusBadge } from './SyncStatusBadge'
import type { SyncFolder } from '../../types'

export interface SyncFolderCardProps {
  folder: SyncFolder
  className?: string
}

export function SyncFolderCard({ folder, className }: SyncFolderCardProps) {
  const syncing = folder.status === 'syncing'

  return (
    <div
      className={cn(
        'rounded-xl bg-surface-800 border border-surface-700 p-4 flex flex-col gap-3',
        folder.status === 'conflict' && 'border-orange-800',
        folder.status === 'error'    && 'border-red-900',
        className
      )}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-2 min-w-0">
          <FolderSync
            size={16}
            className={cn(
              'shrink-0',
              syncing ? 'text-ark-400 animate-pulse' : 'text-slate-400'
            )}
          />
          <div className="flex flex-col min-w-0">
            <span className="text-sm font-medium text-slate-100 truncate">{folder.name}</span>
            <span className="text-xs text-slate-500 font-mono truncate">{folder.localPath}</span>
          </div>
        </div>
        <SyncStatusBadge status={folder.status} />
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-slate-400">
        <div className="flex items-center gap-1.5">
          <Clock size={10} className="shrink-0" />
          {folder.lastSyncAt
            ? formatRelative(folder.lastSyncAt)
            : 'Nunca sincronizado'}
        </div>
        {folder.agentId && (
          <div className="flex items-center gap-1.5">
            <RefreshCw size={10} className="shrink-0" />
            <span className="truncate">Peer ID: {folder.agentId}</span>
          </div>
        )}
        {folder.bytesTransferred > 0 && (
          <div className="text-slate-500">
            {formatBytes(folder.bytesTransferred)} transferidos
          </div>
        )}
        {folder.conflictCount > 0 && (
          <div className="flex items-center gap-1 text-orange-400">
            <AlertTriangle size={10} className="shrink-0" />
            {folder.conflictCount} conflito{folder.conflictCount > 1 ? 's' : ''}
          </div>
        )}
      </div>

      {/* Barra de progresso (apenas quando sincronizando) */}
      {syncing && (
        <div className="h-1 w-full bg-surface-700 rounded-full overflow-hidden">
          <div className="h-full bg-ark-500 rounded-full animate-[pulse_1.5s_ease-in-out_infinite] w-1/2" />
        </div>
      )}
    </div>
  )
}
