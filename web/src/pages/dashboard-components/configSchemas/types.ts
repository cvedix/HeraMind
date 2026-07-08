/**
 * Shared types for configSchema modules.
 */

import type { TFunction } from 'i18next'
import type { ComponentConfigSchema } from '@/components/dashboard/config/ComponentConfigBuilder'
import type { DashboardComponent, Dashboard } from '@/types/dashboard'

export interface SchemaContext {
  setConfigTitle: (value: string) => void
  selectedComponent: DashboardComponent | null
  updateComponent: (id: string, updates: Partial<DashboardComponent>, notify?: boolean) => void
  setComponentConfig: React.Dispatch<React.SetStateAction<Record<string, any>>>
  t: TFunction
  agents: { id: string; name: string; status: string }[]
  currentDashboard: Dashboard | null

  // Extra setters used inside the function body
  setCenterPickerOpen: (open: boolean) => void
  setMapEditorBindings: (bindings: any[]) => void
  setMapEditorOpen: (open: boolean) => void
  setLayerEditorBindings: (bindings: any[]) => void
  setLayerEditorOpen: (open: boolean) => void
  agentsLoading: boolean
  visionModels: { id: string; name: string; backendId: string; backendName: string }[]
  visionModelsLoading: boolean
}

export interface Updaters {
  updateConfig: (key: string) => (value: any) => void
  updateNestedConfig: (parent: string, key: string) => (value: any) => void
  updateDataSource: (ds: any) => void
  updateDataMapping: (newMapping: any) => void
}

export type SchemaFactory = (
  config: any,
  ctx: SchemaContext,
  helpers: Updaters,
) => ComponentConfigSchema | null
