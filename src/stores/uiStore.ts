import { create } from 'zustand'

export type ModalName =
  | 'addServer'
  | 'editServer'
  | 'deleteServer'
  | 'addCluster'
  | 'addMod'
  | 'addTask'
  | 'editTask'
  | 'restoreBackup'
  | 'confirmAction'
  | null

interface UiState {
  sidebarCollapsed: boolean
  activeModal: ModalName
  confirmPayload: unknown
  // Actions
  toggleSidebar: () => void
  openModal: (name: ModalName, payload?: unknown) => void
  closeModal: () => void
}

export const useUiStore = create<UiState>((set) => ({
  sidebarCollapsed: false,
  activeModal: null,
  confirmPayload: null,

  toggleSidebar: () =>
    set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),

  openModal: (name, payload = null) =>
    set({ activeModal: name, confirmPayload: payload }),

  closeModal: () =>
    set({ activeModal: null, confirmPayload: null }),
}))
