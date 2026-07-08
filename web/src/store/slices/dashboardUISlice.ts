/**
 * Dashboard UI Slice
 *
 * Editing mode, component selection, panel visibility.
 * No persistence needed — pure UI state.
 */

import type { StateCreator } from 'zustand'
import type { DashboardStore } from './dashboardHelpers'

export interface DashboardUISlice {
  editMode: boolean
  selectedComponent: string | null
  componentLibraryOpen: boolean
  configPanelOpen: boolean
  configComponentId: string | null
  templateDialogOpen: boolean

  setEditMode: (edit: boolean) => void
  setSelectedComponent: (id: string | null) => void
  setComponentLibraryOpen: (open: boolean) => void
  setConfigPanelOpen: (open: boolean, componentId?: string) => void
  setTemplateDialogOpen: (open: boolean) => void
}

export const createDashboardUISlice: StateCreator<
  DashboardStore, [], [], DashboardUISlice
> = (set, get) => ({
  editMode: false,
  selectedComponent: null,
  componentLibraryOpen: false,
  configPanelOpen: false,
  configComponentId: null,
  templateDialogOpen: false,

  setEditMode: (edit) => set({ editMode: edit, selectedComponent: null }),

  setSelectedComponent: (id) => {
    const { currentDashboard } = get()
    if (id && currentDashboard?.components) {
      if (!currentDashboard.components.some((c) => c.id === id)) {
        set({ selectedComponent: null })
        return
      }
    }
    set({ selectedComponent: id })
  },

  setComponentLibraryOpen: (open) => set({ componentLibraryOpen: open }),

  setConfigPanelOpen: (open, componentId) => {
    const { currentDashboard } = get()
    if (open && componentId && currentDashboard?.components) {
      if (!currentDashboard.components.some((c) => c.id === componentId)) {
        set({ configPanelOpen: false, configComponentId: null })
        return
      }
    }
    set({ configPanelOpen: open, configComponentId: componentId || null })
  },

  setTemplateDialogOpen: (open) => set({ templateDialogOpen: open }),
})
