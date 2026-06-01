import { create } from 'zustand'
import { listen } from '@tauri-apps/api/event'
import * as api from '../utils/tauri'

interface InstallState {
  installing: boolean
  progress: string[]
  error: string | null
  // Actions
  installSteamcmd: (steamcmdDir: string) => Promise<void>
  installServer: (steamcmdDir: string, installDir: string) => Promise<void>
  updateServer: (steamcmdDir: string, installDir: string) => Promise<void>
  clearProgress: () => void
}

export const useInstallStore = create<InstallState>((set) => ({
  installing: false,
  progress: [],
  error: null,

  installSteamcmd: async (steamcmdDir) => {
    set({ installing: true, progress: [], error: null })
    const unlisten = await listen<string>('install:output', (e) => {
      set((s) => ({ progress: [...s.progress, e.payload] }))
    })
    try {
      await api.installSteamcmd(steamcmdDir)
      set({ installing: false })
    } catch (err) {
      set({ installing: false, error: String(err) })
      throw err
    } finally {
      unlisten()
    }
  },

  installServer: async (steamcmdDir, installDir) => {
    set({ installing: true, progress: [], error: null })
    const unlisten = await listen<string>('install:output', (e) => {
      set((s) => ({ progress: [...s.progress, e.payload] }))
    })
    try {
      await api.installArkServer(steamcmdDir, installDir)
      set({ installing: false })
    } catch (err) {
      set({ installing: false, error: String(err) })
      throw err
    } finally {
      unlisten()
    }
  },

  updateServer: async (steamcmdDir, installDir) => {
    set({ installing: true, progress: [], error: null })
    const unlisten = await listen<string>('install:output', (e) => {
      set((s) => ({ progress: [...s.progress, e.payload] }))
    })
    try {
      await api.updateArkServer(steamcmdDir, installDir)
      set({ installing: false })
    } catch (err) {
      set({ installing: false, error: String(err) })
      throw err
    } finally {
      unlisten()
    }
  },

  clearProgress: () => set({ progress: [], error: null }),
}))
