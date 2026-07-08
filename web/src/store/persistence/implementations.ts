/**
 * Persistence Layer - Storage Implementations
 *
 * Concrete implementations of DashboardStorage for different backends.
 */

import type {
  DashboardStorage,
  StorageResult,
  DashboardDTO,
  CreateDashboardDTO,
  UpdateDashboardDTO,
} from './types'
import type { Dashboard } from '@/types/dashboard'
import { generateId } from '@/lib/id'
import {
  toDashboardDTO,
  fromDashboardDTO,
  toCreateDashboardDTO,
  toUpdateDashboardDTO,
} from './types'

// ============================================================================
// LocalStorage Storage
// ============================================================================

const LOCAL_STORAGE_KEY = 'heramind_dashboards'
const LOCAL_STORAGE_CACHE_TIMESTAMP_KEY = 'heramind_dashboards_cache_ts'
const CURRENT_DASHBOARD_KEY = 'heramind_current_dashboard_id'
const LOCAL_TO_SERVER_ID_KEY = 'heramind_local_to_server_id'
const CACHE_TTL_MS = 5 * 60 * 1000 // 5 minutes

export class LocalStorageDashboardStorage implements DashboardStorage {
  private storageKey: string

  constructor(storageKey: string = LOCAL_STORAGE_KEY) {
    this.storageKey = storageKey
  }

  async load(): Promise<StorageResult<Dashboard[]>> {
    try {
      const stored = localStorage.getItem(this.storageKey)
      if (!stored) {
        return { data: [], error: null }
      }

      let parsed: unknown
      try {
        parsed = JSON.parse(stored)
      } catch {
        return { data: [], error: null }
      }
      if (!Array.isArray(parsed)) return { data: [], error: null }
      const dashboards = parsed as Dashboard[]
      // Ensure all dashboards have valid components arrays (defensive against corrupted data)
      const normalized = dashboards
        .filter((d) => typeof d === 'object' && d !== null && 'id' in d)
        .map((d) => ({
          ...d,
          components: Array.isArray(d.components) ? d.components : [],
          layout: d.layout || { columns: 12, rows: 'auto' },
        })) as Dashboard[]
      return { data: normalized, error: null }
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Failed to load from localStorage'),

      }
    }
  }

  async save(dashboards: Dashboard[]): Promise<StorageResult<void>> {
    try {
      localStorage.setItem(this.storageKey, JSON.stringify(dashboards))
      return { data: undefined, error: null }
    } catch (error) {
      // Attempt quota recovery: clear old data and retry once
      if (error instanceof DOMException && error.name === 'QuotaExceededError') {
        console.warn('[LocalStorage] Quota exceeded, clearing old dashboard data and retrying...')
        try {
          localStorage.removeItem(this.storageKey)
          localStorage.setItem(this.storageKey, JSON.stringify(dashboards))
          return { data: undefined, error: null }
        } catch (retryError) {
          return {
            data: null,
            error: retryError instanceof Error
              ? retryError
              : new Error('Failed to save to localStorage even after clearing'),
          }
        }
      }
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Failed to save to localStorage'),
      }
    }
  }

  async sync(dashboard: Dashboard): Promise<StorageResult<Dashboard>> {
    try {
      // Load existing, update, and save back
      const result = await this.load()
      const dashboards = result.data || []

      // If dashboard doesn't have an ID, generate one for new dashboards
      const dashboardToSave = dashboard.id
        ? dashboard
        : { ...dashboard, id: generateId(), createdAt: Date.now(), updatedAt: Date.now() }

      const index = dashboards.findIndex(d => d.id === dashboardToSave.id)
      if (index >= 0) {
        dashboards[index] = dashboardToSave
      } else {
        dashboards.push(dashboardToSave)
      }

      await this.save(dashboards)
      return { data: dashboardToSave, error: null }
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Failed to sync to localStorage'),

      }
    }
  }

  async delete(id: string): Promise<StorageResult<void>> {
    try {
      const result = await this.load()
      const dashboards = (result.data || []).filter(d => d.id !== id)
      await this.save(dashboards)
      return { data: undefined, error: null }
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Failed to delete from localStorage'),

      }
    }
  }

  async reorder(dashboardIds: string[]): Promise<StorageResult<void>> {
    try {
      const result = await this.load()
      const all = result.data || []
      // Reindex the supplied order, then append any dashboards not in the
      // list (defensive — keeps localStorage consistent even if the caller
      // passed a partial list).
      const indexById = new Map(dashboardIds.map((id, i) => [id, i]))
      const reordered = [...all].sort((a, b) => {
        const ia = indexById.get(a.id)
        const ib = indexById.get(b.id)
        if (ia !== undefined && ib !== undefined) return ia - ib
        if (ia !== undefined) return -1
        if (ib !== undefined) return 1
        return 0
      }).map((d, i) => ({ ...d, sortOrder: i }))
      await this.save(reordered)
      return { data: undefined, error: null }
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Failed to reorder in localStorage'),
      }
    }
  }

  isAvailable(): boolean {
    try {
      localStorage.setItem('test', 'test')
      localStorage.removeItem('test')
      return true
    } catch {
      return false
    }
  }

  getType(): string {
    return 'local'
  }

  // Current dashboard helpers
  getCurrentDashboardId(): string | null {
    return localStorage.getItem(CURRENT_DASHBOARD_KEY)
  }

  setCurrentDashboardId(id: string | null): void {
    if (id) {
      localStorage.setItem(CURRENT_DASHBOARD_KEY, id)
    } else {
      localStorage.removeItem(CURRENT_DASHBOARD_KEY)
    }
  }

  clear(): void {
    localStorage.removeItem(this.storageKey)
    localStorage.removeItem(CURRENT_DASHBOARD_KEY)
  }
}

// ============================================================================
// API Storage
// ============================================================================

export class ApiDashboardStorage implements DashboardStorage {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any -- dynamic import, resolved at runtime
  private api: any = null
  private currentDashboardId: string | null = null

  constructor() {
    // Import api module dynamically to avoid circular deps
    this.api = null
  }

  private async getApi() {
    if (!this.api) {
      const module = await import('@/lib/api')
      this.api = module.api
    }
    return this.api
  }

  async load(): Promise<StorageResult<Dashboard[]>> {
    try {
      const api = await this.getApi()
      const response = await api.getDashboards()

      // Backend returns { dashboards: Dashboard[], count: number }
      const dashboards = 'dashboards' in response
        ? (response as { dashboards: typeof response.dashboards; count: number }).dashboards.map(fromDashboardDTO)
        : Array.isArray(response)
          ? response.map(fromDashboardDTO)
          : []

      return { data: dashboards, error: null }
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Failed to load from API'),

      }
    }
  }

  async save(dashboards: Dashboard[]): Promise<StorageResult<void>> {
    // API doesn't support bulk save - sync individual dashboards instead
    // Cache to localStorage for instant access
    try {
      localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(dashboards))
    } catch {
      // Ignore cache errors
    }
    return { data: undefined, error: null }
  }

  async sync(dashboard: Dashboard): Promise<StorageResult<Dashboard>> {
    try {
      const api = await this.getApi()

      // Check if this is a local-only dashboard (has local UUID format, not server format)
      // Server IDs are like "dashboard_1234567890" (timestamp-based)
      // Local IDs are UUIDs like "550e8400-e29b-41d4-a716-446655440000"
      const isLocalDashboard = dashboard.id && !dashboard.id.startsWith('dashboard_')

      // For local dashboards, try to create on server
      if (isLocalDashboard) {
        try {
          // Don't include the local ID - let server generate it
          const { id, createdAt, updatedAt, ...dashboardForCreate } = dashboard
          const createDto = toCreateDashboardDTO(dashboardForCreate as any)
          const result = await api.createDashboard(createDto)
          // Backend returns full Dashboard
          return { data: fromDashboardDTO(result), error: null }
        } catch (createError) {
          console.warn('[ApiStorage] Dashboard creation failed:', createError)
          // Return local version - do NOT fall through to avoid querying server with local UUID
          return { data: dashboard, error: null }
        }
      }

      // For server dashboards, try to update
      const existing = await api.getDashboard(dashboard.id).catch(() => null)

      if (existing) {
        // Update existing - use UpdateDashboardRequest format
        const updateDto = toUpdateDashboardDTO(dashboard)
        const result = await api.updateDashboard(dashboard.id, updateDto)
        // Backend returns full Dashboard
        return { data: fromDashboardDTO(result), error: null }
      } else {
        // Dashboard doesn't exist on server - try to create it
        try {
          const createDto = toCreateDashboardDTO(dashboard)
          const result = await api.createDashboard(createDto)
          return { data: fromDashboardDTO(result), error: null }
        } catch (createError) {
          // Create failed - keep local version
          console.warn('[ApiStorage] Dashboard sync failed, using local version:', createError)
          return { data: dashboard, error: null }
        }
      }
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Failed to sync to API'),

      }
    }
  }

  async delete(id: string): Promise<StorageResult<void>> {
    try {
      const api = await this.getApi()
      await api.deleteDashboard(id)

      // Also remove from local cache
      try {
        const stored = localStorage.getItem(LOCAL_STORAGE_KEY)
        if (stored) {
          const dashboards = JSON.parse(stored) as Dashboard[]
          const filtered = dashboards.filter(d => d.id !== id)
          localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(filtered))
        }
      } catch {
        // Ignore cache errors
      }

      return { data: undefined, error: null }
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Failed to delete from API'),

      }
    }
  }

  async reorder(dashboardIds: string[]): Promise<StorageResult<void>> {
    try {
      const api = await this.getApi()
      await api.reorderDashboards(dashboardIds)
      return { data: undefined, error: null }
    } catch (error) {
      return {
        data: null,
        error: error instanceof Error ? error : new Error('Failed to reorder via API'),
      }
    }
  }

  isAvailable(): boolean {
    // API is always considered available if we have network
    // Errors will be caught during operations
    return typeof window !== 'undefined' && navigator.onLine
  }

  getType(): string {
    return 'api'
  }

  getCurrentDashboardId(): string | null {
    return this.currentDashboardId
  }

  setCurrentDashboardId(id: string | null): void {
    this.currentDashboardId = id
    // Also sync to localStorage
    if (id) {
      localStorage.setItem(CURRENT_DASHBOARD_KEY, id)
    } else {
      localStorage.removeItem(CURRENT_DASHBOARD_KEY)
    }
  }

  clear(): void {
    // Clear local cache only - server data remains
    localStorage.removeItem(LOCAL_STORAGE_KEY)
    localStorage.removeItem(CURRENT_DASHBOARD_KEY)
    this.currentDashboardId = null
  }
}

// ============================================================================
// Hybrid Storage (API with localStorage fallback)
// ============================================================================

export class HybridDashboardStorage implements DashboardStorage {
  private apiStorage: ApiDashboardStorage
  private localStorage: LocalStorageDashboardStorage
  private cacheEnabled: boolean
  // Track in-flight sync operations for local dashboards to prevent duplicate creation.
  // Key: local UUID, Value: the Promise resolving to the server dashboard (or null).
  private pendingSync: Map<string, Promise<StorageResult<Dashboard>>> = new Map()
  // Map local UUID -> server ID so subsequent syncs use the server ID.
  private localToServerId: Map<string, string> = new Map()

  constructor(options: { cacheEnabled?: boolean } = {}) {
    this.apiStorage = new ApiDashboardStorage()
    this.localStorage = new LocalStorageDashboardStorage()
    this.cacheEnabled = options.cacheEnabled ?? true
    // Restore persisted ID mapping from localStorage
    this.loadIdMapping()
  }

  /** Persist localToServerId mapping to localStorage */
  private persistIdMapping(): void {
    try {
      const entries = Array.from(this.localToServerId.entries())
      localStorage.setItem(LOCAL_TO_SERVER_ID_KEY, JSON.stringify(entries))
    } catch { /* ignore storage errors */ }
  }

  /** Restore localToServerId mapping from localStorage */
  private loadIdMapping(): void {
    try {
      const stored = localStorage.getItem(LOCAL_TO_SERVER_ID_KEY)
      if (stored) {
        const entries = JSON.parse(stored) as [string, string][]
        for (const [local, server] of entries) {
          this.localToServerId.set(local, server)
        }
      }
    } catch { /* ignore corrupt data */ }
  }

  async load(): Promise<StorageResult<Dashboard[]>> {
    // Try API first
    const apiResult = await this.apiStorage.load()

    if (apiResult.error || !apiResult.data) {
      console.warn('[HybridStorage] API load failed, checking error type:', apiResult.error?.message)

      // Check if the error is because the dashboards table doesn't exist
      // In this case, fall back to localStorage instead of clearing it
      // This allows users to work locally when backend is unavailable
      const errorMessage = apiResult.error?.message || ''
      const isTableNotExist = errorMessage.includes("Table 'dashboards' does not exist") ||
                             errorMessage.includes('does not exist')

      if (isTableNotExist) {
        return this.localStorage.load()
      }

      // For other errors, fall back to localStorage but check cache freshness
      const cacheAge = this.getCacheAge()
      if (cacheAge !== null && cacheAge > CACHE_TTL_MS) {
        console.warn('[HybridStorage] Cache is stale (' + Math.round(cacheAge / 1000) + 's old), returning empty')
        return { data: [], error: null }
      }

      console.warn('[HybridStorage] API load failed, falling back to localStorage')
      return this.localStorage.load()
    }

    // Cache to localStorage if enabled and update timestamp
    if (this.cacheEnabled && apiResult.data) {
      this.localStorage.save(apiResult.data).catch(() => {})
      this.updateCacheTimestamp()
    }

    return apiResult
  }

  async save(dashboards: Dashboard[]): Promise<StorageResult<void>> {
    // Always save to localStorage immediately for responsiveness
    const localResult = await this.localStorage.save(dashboards)

    // Try to sync to API in background
    this.syncToApi(dashboards).catch(() => {
      // API sync failed, but local save succeeded
      console.warn('[HybridStorage] Background API sync failed')
    })

    return localResult
  }

  async sync(dashboard: Dashboard): Promise<StorageResult<Dashboard>> {
    // Check if this is a local dashboard (UUID format, not server format)
    const isLocalDashboard = dashboard.id && !dashboard.id.startsWith('dashboard_')

    if (isLocalDashboard) {
      // Check if we already have a pending sync for this local ID
      const pending = this.pendingSync.get(dashboard.id)
      if (pending) {
        // A sync is already in progress for this dashboard.
        // Wait for it, then update with our latest data using the server ID.
        try {
          const result = await pending
          if (result.data) {
            // Map the local ID to the server ID for future syncs
            this.localToServerId.set(dashboard.id, result.data.id)
            this.persistIdMapping()
            // Use LATEST dashboard data (not stale result.data) to avoid
            // losing rapid edits that arrived while the first sync was in flight
            const updatedDashboard = { ...dashboard, id: result.data.id, updatedAt: Date.now() }
            return this.doServerSync(updatedDashboard)
          }
        } catch (err) {
          console.warn('[HybridStorage] Pending sync failed, retrying:', err)
        }
      }

      // Check if we already resolved this local ID to a server ID
      const serverId = this.localToServerId.get(dashboard.id)
      if (serverId) {
        // Already synced before - just update the server dashboard
        const updatedDashboard = { ...dashboard, id: serverId, updatedAt: Date.now() }
        return this.doServerSync(updatedDashboard)
      }

      // First time syncing this local dashboard - lock it
      const syncPromise = this.apiStorage.sync(dashboard)
      this.pendingSync.set(dashboard.id, syncPromise)

      try {
        const apiResult = await syncPromise
        if (apiResult.data && apiResult.data.id !== dashboard.id) {
          // Server assigned a new ID - map it
          this.localToServerId.set(dashboard.id, apiResult.data.id)
          this.persistIdMapping()
          // Update localStorage with the server version
          await this.localStorage.sync(apiResult.data)
          return apiResult
        }
        return apiResult
      } catch (apiError) {
        console.warn('[HybridStorage] API sync failed for new dashboard, using local only:', apiError)
        // Fall through to local sync
        return this.localStorage.sync(dashboard)
      } finally {
        this.pendingSync.delete(dashboard.id)
      }
    }

    // For server dashboards, sync to both localStorage and API
    return this.doServerSync(dashboard)
  }

  /**
   * Sync a dashboard that already has a server ID to both localStorage and API.
   */
  private async doServerSync(dashboard: Dashboard): Promise<StorageResult<Dashboard>> {
    const localResult = await this.localStorage.sync(dashboard)

    // Sync to API — track failure so callers can detect stale server state
    try {
      const apiResult = await this.apiStorage.sync(localResult.data || dashboard)
      if (apiResult.error) {
        console.warn('[HybridStorage] API sync error for dashboard:', dashboard.id, apiResult.error)
      }
    } catch (err) {
      console.warn('[HybridStorage] API sync failed for dashboard:', dashboard.id, err)
    }

    return localResult
  }

  async delete(id: string): Promise<StorageResult<void>> {
    // Delete from localStorage first
    const localResult = await this.localStorage.delete(id)

    // Try to delete from API in background
    this.apiStorage.delete(id).catch(() => {
      console.warn('[HybridStorage] API delete failed for dashboard:', id)
    })

    return localResult
  }

  async reorder(dashboardIds: string[]): Promise<StorageResult<void>> {
    // Local first for instant UI
    const localResult = await this.localStorage.reorder(dashboardIds)
    // API second for persistence (awaited so caller can roll back on failure)
    const apiResult = await this.apiStorage.reorder(dashboardIds)
    if (apiResult.error) {
      console.warn('[HybridStorage] API reorder failed:', apiResult.error)
      return apiResult
    }
    return localResult
  }

  isAvailable(): boolean {
    return this.localStorage.isAvailable() || this.apiStorage.isAvailable()
  }

  getType(): string {
    return 'hybrid'
  }

  // Helper to sync all dashboards to API
  // Only syncs dashboards that already have a server ID mapping.
  // Local-only dashboards are synced through the dedicated sync() method
  // which handles ID mapping to prevent duplicate creation.
  private async syncToApi(dashboards: Dashboard[]): Promise<void> {
    await Promise.allSettled(
      dashboards
        .filter(d => {
          // Only sync if the dashboard has a server ID (not local-only)
          const serverId = this.localToServerId.get(d.id)
          return serverId || d.id.startsWith('dashboard_')
        })
        .map(d => this.apiStorage.sync(d))
    )
  }

  // Expose current dashboard helpers from localStorage
  getCurrentDashboardId(): string | null {
    return this.localStorage.getCurrentDashboardId()
  }

  setCurrentDashboardId(id: string | null): void {
    this.localStorage.setCurrentDashboardId(id)
  }

  clear(): void {
    this.localStorage.clear()
    this.localToServerId.clear()
    this.pendingSync.clear()
    try { localStorage.removeItem(LOCAL_STORAGE_CACHE_TIMESTAMP_KEY) } catch {}
    try { localStorage.removeItem(LOCAL_TO_SERVER_ID_KEY) } catch {}
  }

  /** Get cache age in milliseconds, or null if no timestamp */
  private getCacheAge(): number | null {
    try {
      const ts = localStorage.getItem(LOCAL_STORAGE_CACHE_TIMESTAMP_KEY)
      if (!ts) return null
      return Date.now() - parseInt(ts, 10)
    } catch {
      return null
    }
  }

  /** Update the cache timestamp to now */
  private updateCacheTimestamp(): void {
    try {
      localStorage.setItem(LOCAL_STORAGE_CACHE_TIMESTAMP_KEY, String(Date.now()))
    } catch {}
  }
}

// ============================================================================
// Factory
// ============================================================================

export interface CreateStorageOptions {
  type?: 'local' | 'api' | 'hybrid'
  cacheEnabled?: boolean
}

export function createDashboardStorage(options: CreateStorageOptions = {}): DashboardStorage {
  const { type = 'hybrid', cacheEnabled = true } = options

  switch (type) {
    case 'local':
      return new LocalStorageDashboardStorage()
    case 'api':
      return new ApiDashboardStorage()
    case 'hybrid':
      return new HybridDashboardStorage({ cacheEnabled })
    default:
      return new HybridDashboardStorage({ cacheEnabled })
  }
}
