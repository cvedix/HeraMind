/**
 * Alert Selectors
 *
 * Memoized selectors for alert state using Zustand's optimized selector pattern.
 */

import type { Alert } from '@/types'
import type { HeraMindStore } from '../'

// Helper to extract timestamp number from Alert
const getAlertTimestamp = (alert: Alert): number => {
  if (typeof alert.created_at === 'number') {
    return alert.created_at
  }
  if (typeof alert.timestamp === 'string') {
    // Parse ISO string to number
    return new Date(alert.timestamp).getTime()
  }
  if (typeof alert.created_at === 'string') {
    return new Date(alert.created_at).getTime()
  }
  // Default to current time if no timestamp
  return Date.now()
}

// ============================================================================
// Base Selectors
// ============================================================================

/**
 * Get all alerts
 */
export const selectAlertsRaw = (state: HeraMindStore): Alert[] => state.alerts

// ============================================================================
// Status-Based Selectors
// ============================================================================

/**
 * Get unacknowledged alerts
 */
export const selectUnacknowledgedAlerts = (state: HeraMindStore): Alert[] =>
  state.alerts.filter((a) => !a.acknowledged)

/**
 * Get acknowledged alerts
 */
export const selectAcknowledgedAlerts = (state: HeraMindStore): Alert[] =>
  state.alerts.filter((a) => a.acknowledged)

/**
 * Get active (unresolved) alerts
 */
export const selectActiveAlerts = (state: HeraMindStore): Alert[] =>
  state.alerts.filter((a) => a.status !== 'resolved')

/**
 * Get resolved alerts
 */
export const selectResolvedAlerts = (state: HeraMindStore): Alert[] =>
  state.alerts.filter((a) => a.status === 'resolved')

// ============================================================================
// Severity-Based Selectors
// ============================================================================

/**
 * Get critical alerts (highest priority)
 */
export const selectCriticalAlerts = (state: HeraMindStore): Alert[] =>
  state.alerts.filter((a) => a.severity === 'critical')

/**
 * Get warning alerts
 */
export const selectWarningAlerts = (state: HeraMindStore): Alert[] =>
  state.alerts.filter((a) => a.severity === 'warning')

/**
 * Get info alerts
 */
export const selectInfoAlerts = (state: HeraMindStore): Alert[] =>
  state.alerts.filter((a) => a.severity === 'info')

/**
 * Get alerts by severity
 */
export const selectAlertsBySeverity = (state: HeraMindStore, severity: string): Alert[] =>
  state.alerts.filter((a) => a.severity === severity)

// ============================================================================
// Combined Filter Selectors
// ============================================================================

/**
 * Get unacknowledged critical alerts (highest priority for attention)
 */
export const selectUnacknowledgedCriticalAlerts = (state: HeraMindStore): Alert[] =>
  state.alerts.filter((a) => !a.acknowledged && a.severity === 'critical')

/**
 * Get unacknowledged alerts by severity
 */
export const selectUnacknowledgedBySeverity = (state: HeraMindStore, severity: string): Alert[] =>
  state.alerts.filter((a) => !a.acknowledged && a.severity === severity)

/**
 * Get active alerts by severity
 */
export const selectActiveBySeverity = (state: HeraMindStore, severity: string): Alert[] =>
  state.alerts.filter((a) => a.status !== 'resolved' && a.severity === severity)

// ============================================================================
// Alert Summary Selectors
// ============================================================================

/**
 * Get count of unacknowledged alerts
 */
export const selectUnacknowledgedAlertCount = (state: HeraMindStore): number =>
  state.alerts.filter((a) => !a.acknowledged).length

/**
 * Get count of critical alerts
 */
export const selectCriticalAlertCount = (state: HeraMindStore): number =>
  state.alerts.filter((a) => a.severity === 'critical').length

/**
 * Get count of active alerts
 */
export const selectActiveAlertCount = (state: HeraMindStore): number =>
  state.alerts.filter((a) => a.status !== 'resolved').length

/**
 * Get total alert count
 */
export const selectTotalAlertCount = (state: HeraMindStore): number =>
  state.alerts.length

/**
 * Get alert summary by severity
 */
export const selectAlertSummaryBySeverity = (state: HeraMindStore) => {
  const summary = {
    critical: 0,
    warning: 0,
    info: 0,
    total: state.alerts.length,
  }

  for (const alert of state.alerts) {
    if (alert.severity === 'critical') {
      summary.critical++
    } else if (alert.severity === 'warning') {
      summary.warning++
    } else if (alert.severity === 'info') {
      summary.info++
    }
  }

  return summary
}

/**
 * Get alert summary by status and severity
 */
export const selectAlertFullSummary = (state: HeraMindStore) => {
  const unacknowledged = {
    critical: 0,
    warning: 0,
    info: 0,
    total: 0,
  }

  const active = {
    critical: 0,
    warning: 0,
    info: 0,
    total: 0,
  }

  for (const alert of state.alerts) {
    // Count unacknowledged
    if (!alert.acknowledged) {
      unacknowledged.total++
      if (alert.severity === 'critical') {
        unacknowledged.critical++
      } else if (alert.severity === 'warning') {
        unacknowledged.warning++
      } else if (alert.severity === 'info') {
        unacknowledged.info++
      }
    }

    // Count active (not resolved)
    if (alert.status !== 'resolved') {
      active.total++
      if (alert.severity === 'critical') {
        active.critical++
      } else if (alert.severity === 'warning') {
        active.warning++
      } else if (alert.severity === 'info') {
        active.info++
      }
    }
  }

  return {
    unacknowledged,
    active,
    total: state.alerts.length,
  }
}

// ============================================================================
// Sorted Alert Selectors
// ============================================================================

/**
 * Get alerts sorted by severity (critical first)
 */
export const selectAlertsBySeverityPriority = (state: HeraMindStore): Alert[] => {
  const severityOrder = { critical: 0, warning: 1, info: 2, emergency: -1 }

  return [...state.alerts].sort((a, b) => {
    const aSeverity = severityOrder[a.severity as keyof typeof severityOrder] ?? 999
    const bSeverity = severityOrder[b.severity as keyof typeof severityOrder] ?? 999
    if (aSeverity !== bSeverity) {
      return aSeverity - bSeverity
    }
    // Secondary sort by timestamp (newest first)
    return getAlertTimestamp(b) - getAlertTimestamp(a)
  })
}

/**
 * Get alerts sorted by timestamp (newest first)
 */
export const selectAlertsByNewest = (state: HeraMindStore): Alert[] =>
  [...state.alerts].sort((a, b) => getAlertTimestamp(b) - getAlertTimestamp(a))

/**
 * Get alerts sorted by timestamp (oldest first)
 */
export const selectAlertsByOldest = (state: HeraMindStore): Alert[] =>
  [...state.alerts].sort((a, b) => getAlertTimestamp(a) - getAlertTimestamp(b))

// ============================================================================
// Alert Map Selector
// ============================================================================

/**
 * Get alerts as a Map for O(1) lookups by ID
 */
export const selectAlertMap = (state: HeraMindStore): Map<string, Alert> => {
  const map = new Map<string, Alert>()
  state.alerts.forEach(a => map.set(a.id, a))
  return map
}

/**
 * Get alert by ID using the map
 */
export const selectAlertById = (_state: HeraMindStore, alertId: string, alertMap: Map<string, Alert>): Alert | undefined =>
  alertMap.get(alertId)
