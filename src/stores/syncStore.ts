import { create } from 'zustand'
import type { SyncFolder, SyncEvent, SyncConflict } from '../types'
import * as api from '../utils/tauri'

interface SyncState {
  folders: SyncFolder[]
  events: Record<number, SyncEvent[]>
  conflicts: Record<number, SyncConflict[]>
  loading: boolean
  error: string | null
  // Actions
  fetchFolders: () => Promise<void>
  addFolder: (name: string, localPath: string, agentId?: number) => Promise<SyncFolder>
  removeFolder: (id: number) => Promise<void>
  forceSync: (folderId: number) => Promise<void>
  fetchEvents: (folderId: number, limit?: number) => Promise<void>
  fetchConflicts: (folderId: number) => Promise<void>
}

export const useSyncStore = create<SyncState>((set) => ({
  folders: [],
  events: {},
  conflicts: {},
  loading: false,
  error: null,

  fetchFolders: async () => {
    set({ loading: true, error: null })
    try {
      const folders = await api.listSyncFolders()
      set({ folders, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  addFolder: async (name, localPath, agentId) => {
    const folder = await api.addSyncFolder(name, localPath, agentId)
    set(s => ({ folders: [...s.folders, folder] }))
    return folder
  },

  removeFolder: async (id) => {
    await api.removeSyncFolder(id)
    set(s => ({ folders: s.folders.filter(f => f.id !== id) }))
  },

  forceSync: async (folderId) => {
    // Marca como syncing otimisticamente
    set(s => ({
      folders: s.folders.map(f =>
        f.id === folderId ? { ...f, status: 'syncing' as const } : f
      ),
    }))
    try {
      await api.forceSync(folderId)
      // Recarrega o folder atualizado
      const folders = await api.listSyncFolders()
      set({ folders })
    } catch (err) {
      set(s => ({
        folders: s.folders.map(f =>
          f.id === folderId ? { ...f, status: 'error' as const } : f
        ),
        error: String(err),
      }))
      throw err
    }
  },

  fetchEvents: async (folderId, limit) => {
    const events = await api.getSyncEvents(folderId, limit)
    set(s => ({ events: { ...s.events, [folderId]: events } }))
  },

  fetchConflicts: async (folderId) => {
    const conflicts = await api.getSyncConflicts(folderId)
    set(s => ({ conflicts: { ...s.conflicts, [folderId]: conflicts } }))
  },
}))
