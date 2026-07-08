/**
 * useDataAvailability Hook
 *
 * Checks if historical telemetry data exists for device metrics.
 * Uses the telemetry summary API to determine which metrics have available data.
 * Also returns the full summary data including virtual metrics.
 */

import { useState, useEffect, useRef, useCallback } from 'react'
import { api } from '@/lib/api'
import type { TelemetryMetricSummary } from '@/types'

export interface MetricAvailability {
  metricId: string
  hasData: boolean
  dataPointCount?: number
  lastTimestamp?: number | null
}

export interface UseDataAvailabilityResult {
  availability: Map<string, MetricAvailability>
  loading: boolean
  error: string | null
  refresh: () => void
  checkDevice: (deviceId: string, metricIds: string[]) => void
  // Full summary data per device (includes virtual metrics)
  summaries: Map<string, Record<string, TelemetryMetricSummary>>
}

// Cache for availability data to avoid redundant API calls
const availabilityCache = new Map<string, {
  availability: Map<string, MetricAvailability>
  summary: Record<string, TelemetryMetricSummary>
  timestamp: number
}>()
const CACHE_TTL = 30000 // 30 seconds cache

/**
 * Check if cached data is still valid
 */
function getCachedData(deviceId: string):
  { availability: Map<string, MetricAvailability>; summary: Record<string, TelemetryMetricSummary> } | null {
  const cached = availabilityCache.get(deviceId)
  if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
    return {
      availability: cached.availability,
      summary: cached.summary,
    }
  }
  return null
}

/**
 * Set cached availability data
 */
function setCachedData(
  deviceId: string,
  availability: Map<string, MetricAvailability>,
  summary: Record<string, TelemetryMetricSummary>
) {
  availabilityCache.set(deviceId, {
    availability,
    summary,
    timestamp: Date.now()
  })
}

/**
 * Hook to check data availability for device metrics
 */
export function useDataAvailability(): UseDataAvailabilityResult {
  const [availability, setAvailability] = useState<Map<string, MetricAvailability>>(new Map())
  const [summaries, setSummaries] = useState<Map<string, Record<string, TelemetryMetricSummary>>>(new Map())
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const activeFetches = useRef<Set<string>>(new Set())

  /**
   * Fetch telemetry summary for a device to determine which metrics have data
   */
  const checkDevice = useCallback(async (deviceId: string, metricIds: string[]) => {
    // Always check, even if metricIds is empty - we want to get ALL metrics including virtual ones
    // Check cache first
    const cached = getCachedData(deviceId)
    if (cached) {
      setAvailability(prev => {
        const next = new Map(prev)
        // Update all metrics from cache (including virtual ones not in metricIds)
        for (const [key, value] of cached.availability) {
          next.set(key, value)
        }
        return next
      })
      setSummaries(prev => {
        const next = new Map(prev)
        next.set(deviceId, cached.summary)
        return next
      })
      return
    }

    // Avoid duplicate fetches
    const fetchKey = `${deviceId}:${metricIds.sort().join(',')}`
    if (activeFetches.current.has(fetchKey)) {
      return
    }

    activeFetches.current.add(fetchKey)
    setLoading(true)
    setError(null)

    try {
      const summary = await api.getDeviceTelemetrySummary(deviceId, 24) // Check last 24 hours

      const newAvailability = new Map<string, MetricAvailability>()

      if (summary?.summary) {
        // Process ALL metrics from summary (including virtual metrics)
        for (const [metricId, metricSummary] of Object.entries(summary.summary)) {
          newAvailability.set(`${deviceId}:${metricId}`, {
            metricId,
            hasData: metricSummary.count > 0,
            dataPointCount: metricSummary.count,
            lastTimestamp: metricSummary.current_timestamp,
          })
        }

        // Store the full summary for access to virtual metrics
        setSummaries(prev => {
          const next = new Map(prev)
          next.set(deviceId, summary.summary)
          return next
        })
      }

      // Cache the results
      setCachedData(deviceId, newAvailability, summary?.summary || {})

      setAvailability(prev => {
        const next = new Map(prev)
        for (const [key, value] of newAvailability) {
          next.set(key, value)
        }
        return next
      })
    } catch (err) {
      console.error('[useDataAvailability] Error:', err)
      setError(err instanceof Error ? err.message : 'Failed to check data availability')
    } finally {
      activeFetches.current.delete(fetchKey)
      setLoading(false)
    }
  }, [])

  /**
   * Refresh all cached availability data
   */
  const refresh = useCallback(() => {
    // Clear cache
    availabilityCache.clear()
    setAvailability(new Map())
    setSummaries(new Map())
  }, [])

  return {
    availability,
    summaries,
    loading,
    error,
    refresh,
    checkDevice,
  }
}

/**
 * Convenience hook to check availability for a specific device
 */
export function useDeviceDataAvailability(deviceId: string, metricIds: string[]) {
  const { availability, loading, checkDevice } = useDataAvailability()

  useEffect(() => {
    if (deviceId && metricIds.length > 0) {
      checkDevice(deviceId, metricIds)
    }
  }, [deviceId, metricIds.join(',')])

  const deviceAvailability: Record<string, MetricAvailability> = {}
  for (const metricId of metricIds) {
    const key = `${deviceId}:${metricId}`
    const data = availability.get(key)
    if (data) {
      deviceAvailability[metricId] = data
    }
  }

  return {
    availability: deviceAvailability,
    loading,
    hasAnyData: Object.values(deviceAvailability).some(a => a.hasData),
  }
}
