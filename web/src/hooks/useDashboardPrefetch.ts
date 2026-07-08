/**
 * useDashboardPrefetch Hook
 *
 * Pre-warms the telemetry cache for dashboard chart/sparkline components.
 * Batch device current-values fetching is already handled by the existing
 * dashboardDeviceIdsKey effect in VisualDashboard — this hook only handles
 * historical telemetry (the slow part).
 */

import { useEffect, useRef } from 'react'
import type { DashboardComponent, DataSource, TelemetryAggregate } from '@/types/dashboard'
import { getSourceId, normalizeDataSource } from '@/types/dashboard'
import { fetchHistoricalTelemetry } from '@/hooks/useDataSource'

function extractTelemetrySources(components: DashboardComponent[]) {
  const seen = new Set<string>()
  const sources: Array<{
    deviceId: string
    metricId: string
    timeRange: number
    limit: number
    aggregate: TelemetryAggregate
  }> = []

  for (const component of components) {
    const ds = (component as { dataSource?: DataSource | DataSource[] }).dataSource
    if (!ds) continue

    for (const source of normalizeDataSource(ds)) {
      if (source.type !== 'telemetry') continue
      const deviceId = getSourceId(source)
      const metricId = source.property
      if (!deviceId || !metricId) continue

      const timeRange = source.timeRange ?? 1
      const limit = source.limit ?? 50
      const aggregate = source.aggregate ?? 'raw'
      const key = `${deviceId}|${metricId}|${timeRange}|${limit}|${aggregate}`

      if (!seen.has(key)) {
        seen.add(key)
        sources.push({ deviceId, metricId, timeRange, limit, aggregate })
      }
    }
  }
  return sources
}

export function useDashboardPrefetch(components: DashboardComponent[]) {
  // Track which component set we last prefetched
  const prefetchedKeyRef = useRef('')

  useEffect(() => {
    if (components.length === 0) return

    const key = components.map((c) => c.id).join(',')
    if (key === prefetchedKeyRef.current) return
    prefetchedKeyRef.current = key

    const sources = extractTelemetrySources(components)
    if (sources.length === 0) return

    // Fire all telemetry fetches concurrently — they populate the shared cache
    // so individual useDataSource hooks find cached data and skip API calls
    Promise.all(
      sources.map((s) =>
        fetchHistoricalTelemetry(s.deviceId, s.metricId, s.timeRange, s.limit, s.aggregate)
      )
    ).catch((err) => {
      console.warn('[useDashboardPrefetch] telemetry prefetch failed:', err)
    })
  }, [components])
}
