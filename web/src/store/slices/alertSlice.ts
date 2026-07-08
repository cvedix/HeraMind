/**
 * Alert Slice
 *
 * Handles alert state and operations.
 * Note: Now uses the unified messages API internally for backward compatibility.
 */

import type { StateCreator } from 'zustand'
import type { AlertState } from '../types'
import { api } from '@/lib/api'
import { logError } from '@/lib/errors'

export interface AlertSlice extends AlertState {
  // Actions
  fetchAlerts: () => Promise<void>
  acknowledgeAlert: (id: string) => Promise<boolean>
  createAlert: (alert: { title: string; message: string; severity?: string; source?: string }) => Promise<boolean>
}

export const createAlertSlice: StateCreator<
  AlertSlice,
  [],
  [],
  AlertSlice
> = (set, get) => ({
  // Initial state
  alerts: [],
  alertsLoading: false,

  // Actions
  fetchAlerts: async () => {
    set({ alertsLoading: true })
    try {
      const response = await api.getMessages()
      const messagesArray = response.messages || []

      // Convert messages to alert format for backward compatibility
      const alertsArray = messagesArray.map((msg: any) => ({
        id: msg.id,
        title: msg.title,
        message: msg.message,
        severity: msg.severity,
        status: msg.status,
        source: msg.source,
        acknowledged: msg.status !== 'active',
        timestamp: msg.timestamp || msg.created_at,
      }))

      // Sort by timestamp descending (newest first)
      alertsArray.sort((a: any, b: any) => {
        const aTime = new Date(a.timestamp).getTime()
        const bTime = new Date(b.timestamp).getTime()
        if (isNaN(aTime)) return 1
        if (isNaN(bTime)) return -1
        return bTime - aTime
      })

      set({ alerts: alertsArray as any })
    } catch (error) {
      logError(error, { operation: 'Fetch alerts' })
      set({ alerts: [] })
    } finally {
      set({ alertsLoading: false })
    }
  },

  acknowledgeAlert: async (id: string) => {
    try {
      const result = await api.acknowledgeMessage(id)
      if (result.acknowledged) {
        // Update the alert in the list
        set((state) => ({
          alerts: state.alerts.map((alert) =>
            alert.id === id
              ? { ...alert, acknowledged: true, status: 'acknowledged' as const }
              : alert
          ),
        }))
        return true
      }
      return false
    } catch (error) {
      logError(error, { operation: 'Acknowledge alert' })
      return false
    }
  },

  createAlert: async (alert) => {
    try {
      // Map alert to message format
      await api.createMessage({
        category: 'alert',
        severity: (alert.severity || 'info') as any,
        title: alert.title,
        message: alert.message,
        source: alert.source || 'manual',
      })
      // Refresh the alerts list after creating
      await get().fetchAlerts()
      return true
    } catch (error) {
      logError(error, { operation: 'Create alert' })
      return false
    }
  },
})
