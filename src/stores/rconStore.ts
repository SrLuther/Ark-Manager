import { create } from 'zustand'
import * as api from '../utils/tauri'

export type RconEntryType = 'command' | 'response' | 'error' | 'system'

export interface RconEntry {
  type: RconEntryType
  text: string
  ts: string
}

interface RconState {
  connected: boolean
  connecting: boolean
  history: RconEntry[]
  error: string | null
  // Actions
  connect: (serverId: number, host: string, port: number, password: string) => Promise<void>
  disconnect: (serverId: number) => Promise<void>
  sendCommand: (serverId: number, command: string) => Promise<void>
  clearHistory: () => void
}

export const useRconStore = create<RconState>((set, get) => ({
  connected: false,
  connecting: false,
  history: [],
  error: null,

  connect: async (serverId, host, port, password) => {
    set({ connecting: true, error: null })
    try {
      await api.rconConnect(serverId, host, port, password)
      set({
        connected: true,
        connecting: false,
        history: [{ type: 'system', text: `Conectado a ${host}:${port}`, ts: new Date().toISOString() }],
      })
    } catch (err) {
      set({ connecting: false, error: String(err) })
      throw err
    }
  },

  disconnect: async (serverId) => {
    try {
      await api.rconDisconnect(serverId)
    } finally {
      set({
        connected: false,
        history: (state => [
          ...state,
          { type: 'system' as RconEntryType, text: 'Desconectado.', ts: new Date().toISOString() },
        ])(get().history),
      })
    }
  },

  sendCommand: async (serverId, command) => {
    const ts = new Date().toISOString()
    set((s) => ({
      history: [...s.history, { type: 'command', text: command, ts }],
    }))
    try {
      const response = await api.rconSendCommand(serverId, command)
      set((s) => ({
        history: [
          ...s.history,
          { type: 'response', text: (response as string) || '(sem resposta)', ts: new Date().toISOString() },
        ],
      }))
    } catch (err) {
      set((s) => ({
        history: [
          ...s.history,
          { type: 'error', text: String(err), ts: new Date().toISOString() },
        ],
      }))
    }
  },

  clearHistory: () => set({ history: [] }),
}))
