import { create } from 'zustand'
import type { Server } from '../types'
import * as api from '../utils/tauri'

interface ServerState {
  servers: Server[]
  selectedServerId: number | null
  loading: boolean
  error: string | null
  // Actions
  fetchServers: () => Promise<void>
  selectServer: (id: number | null) => void
  startServer: (id: number) => Promise<void>
  stopServer: (id: number) => Promise<void>
  restartServer: (id: number) => Promise<void>
  refreshStatus: (id: number) => Promise<void>
  removeServer: (id: number) => Promise<void>
}

export const useServerStore = create<ServerState>((set, get) => ({
  servers: [],
  selectedServerId: null,
  loading: false,
  error: null,

  fetchServers: async () => {
    set({ loading: true, error: null })
    try {
      const servers = await api.listServers()
      set({ servers: servers as Server[], loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  selectServer: (id) => set({ selectedServerId: id }),

  startServer: async (id) => {
    try {
      await api.startServer(id)
      await get().refreshStatus(id)
    } catch (err) {
      set({ error: String(err) })
      throw err
    }
  },

  stopServer: async (id) => {
    try {
      await api.stopServer(id)
      await get().refreshStatus(id)
    } catch (err) {
      set({ error: String(err) })
      throw err
    }
  },

  restartServer: async (id) => {
    try {
      await api.restartServer(id)
      await get().refreshStatus(id)
    } catch (err) {
      set({ error: String(err) })
      throw err
    }
  },

  refreshStatus: async (id) => {
    try {
      const updated = await api.serverStatus(id)
      set((state) => ({
        servers: state.servers.map((s) =>
          s.id === id ? { ...s, ...(updated as Server) } : s
        ),
      }))
    } catch {
      // Silencioso — não propaga erro de status individual
    }
  },

  removeServer: async (id) => {
    try {
      await api.deleteServer(id)
      set((state) => ({
        servers: state.servers.filter((s) => s.id !== id),
        selectedServerId:
          state.selectedServerId === id ? null : state.selectedServerId,
      }))
    } catch (err) {
      set({ error: String(err) })
      throw err
    }
  },
}))
