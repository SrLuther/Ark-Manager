import { create } from 'zustand'
import type { SeasonalEvent, CreateEventRequest } from '../types'
import * as api from '../utils/tauri'

interface EventState {
  events: SeasonalEvent[]
  loading: boolean
  error: string | null
  // Actions
  fetchEvents: () => Promise<void>
  createEvent: (req: CreateEventRequest) => Promise<SeasonalEvent>
  cancelEvent: (id: number) => Promise<void>
  forceStart: (id: number) => Promise<void>
  forceEnd: (id: number) => Promise<void>
  refreshEvent: (id: number) => Promise<void>
}

export const useSeasonalEventStore = create<EventState>((set) => ({
  events: [],
  loading: false,
  error: null,

  fetchEvents: async () => {
    set({ loading: true, error: null })
    try {
      const events = await api.listSeasonalEvents()
      set({ events, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  createEvent: async (req) => {
    const event = await api.createSeasonalEvent(req)
    set(s => ({ events: [event, ...s.events] }))
    return event
  },

  cancelEvent: async (id) => {
    await api.cancelSeasonalEvent(id)
    set(s => ({
      events: s.events.map(e => e.id === id ? { ...e, status: 'cancelled' as const } : e),
    }))
  },

  forceStart: async (id) => {
    await api.forceStartEvent(id)
    set(s => ({
      events: s.events.map(e => e.id === id ? { ...e, status: 'active' as const } : e),
    }))
  },

  forceEnd: async (id) => {
    await api.forceEndEvent(id)
    set(s => ({
      events: s.events.map(e => e.id === id ? { ...e, status: 'ended' as const } : e),
    }))
  },

  refreshEvent: async (id) => {
    const updated = await api.getSeasonalEvent(id)
    set(s => ({
      events: s.events.map(e => e.id === id ? updated : e),
    }))
  },
}))
