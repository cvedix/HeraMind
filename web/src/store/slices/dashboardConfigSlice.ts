/**
 * Dashboard Config Slice
 *
 * Component configuration update + template application.
 * Uses scheduleSync from DashboardCrudSlice via get().
 */

import type { StateCreator } from 'zustand'
import type {
  Dashboard,
  DashboardComponent,
  DashboardTemplate,
} from '@/types/dashboard'
import { generateId } from './dashboardHelpers'
import type { DashboardStore } from './dashboardHelpers'

export interface DashboardConfigSlice {
  updateComponent: (id: string, updates: Partial<DashboardComponent>, persist?: boolean) => void
  applyTemplate: (template: DashboardTemplate) => void
}

export const createDashboardConfigSlice: StateCreator<
  DashboardStore, [], [], DashboardConfigSlice
> = (set, get) => ({
  updateComponent(id, updates, persist = true) {
    const { currentDashboard, dashboards } = get()
    if (!currentDashboard) return
    if (!currentDashboard.components.some((c) => c.id === id)) {
      console.warn('[DashboardConfigSlice] Component not found:', id)
      return
    }
    const updatedDashboard = {
      ...currentDashboard,
      components: currentDashboard.components.map((c) =>
        c.id === id ? { ...c, ...updates } : c,
      ),
      updatedAt: Date.now(),
    }
    const updatedDashboards = dashboards.map((d) =>
      d.id === currentDashboard.id ? updatedDashboard : d,
    )
    set({ dashboards: updatedDashboards, currentDashboard: updatedDashboard })
    if (persist) get().scheduleSync(updatedDashboard)
  },

  applyTemplate(template) {
    if (!template || !Array.isArray(template.components)) {
      console.warn('[DashboardConfigSlice] Invalid template:', template)
      return
    }
    const newDashboard: Dashboard = {
      id: generateId(),
      name: template.name || 'Untitled',
      layout: template.layout || { columns: 12, rows: 'auto' },
      components: template.components.map((c) => ({ ...structuredClone(c), id: generateId() })) as DashboardComponent[],
      createdAt: Date.now(),
      updatedAt: Date.now(),
    }
    set((state) => ({
      dashboards: [...state.dashboards, newDashboard],
      currentDashboardId: newDashboard.id,
      currentDashboard: newDashboard,
    }))
    get().scheduleSync(newDashboard)
  },
})
