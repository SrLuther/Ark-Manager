import { create } from 'zustand'
import type { SyncAgent, DiscoveredAgent } from '../types'
import * as api from '../utils/tauri'

interface AgentState {
  agents: SyncAgent[]
  discovered: DiscoveredAgent[]
  pairingCode: string | null
  loading: boolean
  discovering: boolean
  error: string | null
  // Actions
  fetchAgents: () => Promise<void>
  discoverAgents: () => Promise<void>
  pairAgent: (address: string, port: number, code: string) => Promise<SyncAgent>
  removeAgent: (id: number) => Promise<void>
  generatePairingCode: () => Promise<string>
  checkStatus: (address: string, port: number) => Promise<boolean>
}

export const useAgentStore = create<AgentState>((set) => ({
  agents: [],
  discovered: [],
  pairingCode: null,
  loading: false,
  discovering: false,
  error: null,

  fetchAgents: async () => {
    set({ loading: true, error: null })
    try {
      const agents = await api.listAgents()
      set({ agents, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  discoverAgents: async () => {
    set({ discovering: true })
    try {
      const discovered = await api.discoverAgents()
      set({ discovered, discovering: false })
    } catch {
      set({ discovering: false })
    }
  },

  pairAgent: async (address, port, code) => {
    const agent = await api.pairAgent(address, port, code)
    set(s => ({
      agents: s.agents.some(a => a.id === agent.id)
        ? s.agents.map(a => a.id === agent.id ? agent : a)
        : [...s.agents, agent],
    }))
    return agent
  },

  removeAgent: async (id) => {
    await api.removeAgent(id)
    set(s => ({ agents: s.agents.filter(a => a.id !== id) }))
  },

  generatePairingCode: async () => {
    const code = await api.generatePairingCode()
    set({ pairingCode: code })
    return code
  },

  checkStatus: async (address, port) => {
    return api.getAgentStatus(address, port)
  },
}))
