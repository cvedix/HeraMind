/**
 * Shared Chart Data Pipeline Hook
 *
 * Encapsulates the repeated pattern across all chart components:
 * effectiveAggregate → toTelemetrySource → useDataSource → getSeriesName wrappers
 */

import { useMemo, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { useDataSource } from '@/hooks/useDataSource'
import type { DataSource, DataSourceOrList, TelemetryAggregate } from '@/types/dashboard'
import { toTelemetrySource } from '@/lib/chartTelemetry'
import { createChartTimeFormatter } from '@/lib/telemetryTransform'
import { DataMapper, type TimeSeriesMappingConfig } from '@/lib/dataMapping'
import { getDeviceName as _getDeviceName, getPropertyDisplayName as _getPropertyDisplayName, getSeriesName as _getSeriesName } from '@/lib/chartDisplay'

// ============================================================================
// Multi-source timestamp alignment (shared between LineChart and AreaChart)
// ============================================================================

export interface SeriesData {
  name: string
  data: number[]
  color?: string
}

export interface AlignedData {
  chartLabels: string[]
  series: SeriesData[]
}

/**
 * Align multi-source data by shared timestamps.
 * Returns null if multi-source alignment is not applicable
 * (e.g., not all sources have timestamps).
 */
export function alignMultiSource(
  data: unknown[],
  sources: DataSource[],
  getSeriesName: (ds: DataSource, idx: number) => string,
  dataMapping?: TimeSeriesMappingConfig
): AlignedData | null {
  const sourceTimeValues = sources.map((_, idx) => {
    const sourceData = data[idx]
    if (!Array.isArray(sourceData) || sourceData.length === 0) return []
    if (typeof sourceData[0] === 'number') return []
    return DataMapper.mapToTimeSeries(sourceData, dataMapping)
  })

  const allHaveTimestamps = sourceTimeValues.every(arr => arr.length > 0)
  if (!allHaveTimestamps) return null

  // Collect all unique timestamps, sorted ascending
  const allTimestamps = new Set<number>()
  for (const points of sourceTimeValues) {
    for (const p of points) {
      if (p.timestamp !== undefined) allTimestamps.add(p.timestamp)
    }
  }
  const sortedTimestamps = Array.from(allTimestamps).sort((a, b) => a - b)

  // Format timestamps as labels with date awareness
  const fmtTime = createChartTimeFormatter(sortedTimestamps)
  const chartLabels = sortedTimestamps.map(ts => fmtTime(ts) || String(ts))

  // Build timestamp → value map per source for O(1) lookup
  const series = sources.map((ds, idx) => {
    const tsMap = new Map<number, number>()
    for (const p of sourceTimeValues[idx]) {
      if (p.timestamp !== undefined) tsMap.set(p.timestamp, p.value)
    }
    return {
      name: getSeriesName(ds, idx),
      data: sortedTimestamps.map(ts => tsMap.get(ts) ?? null) as number[],
      color: undefined,
    } as SeriesData
  })

  return { chartLabels, series }
}

export interface UseChartPipelineOptions {
  dataSource?: DataSourceOrList
  aggregate?: TelemetryAggregate
  limit?: number
  timeRange?: number
  fallback?: unknown
  preserveMultiple?: boolean
}

export interface UseChartPipelineReturn<T = unknown> {
  sources: DataSource[]
  data: T
  loading: boolean
  error: unknown
  effectiveAggregate: TelemetryAggregate
  hasData: boolean
  showLoading: boolean
  getSeriesName: (ds: DataSource, idx: number) => string
  getDeviceName: (deviceId?: string) => string
}

/**
 * Hook that encapsulates the shared data pipeline for all chart components.
 * Returns normalized sources, fetched data, loading states, and series name helpers.
 */
export function useChartPipeline<T = unknown>(
  options: UseChartPipelineOptions
): UseChartPipelineReturn<T> {
  const {
    dataSource,
    aggregate = 'raw',
    limit = 50,
    timeRange = 1,
    fallback,
    preserveMultiple = true,
  } = options

  const { t } = useTranslation('dashboardComponents')

  // Convert to telemetry sources — applied as sourceTransform inside useDataSource
  // When all sources already have mode='timeseries', skip transform (Phase 4)
  const telemetrySourceTransform = useCallback((ds: DataSource): DataSource | undefined => {
    if (ds.mode === 'timeseries') return ds  // Already unified, no transform needed
    return toTelemetrySource(ds, ds.limit ?? limit, ds.timeRange ?? timeRange)
  }, [limit, timeRange])

  // Fetch data — useDataSource normalizes internally, returns normalized sources
  const { data, loading, error, sources } = useDataSource<T>(
    dataSource,
    {
      fallback: fallback as T,
      preserveMultiple,
      sourceTransform: telemetrySourceTransform,
    }
  )

  // Get effective aggregate from dataSource or props
  // KNOWN LIMITATION: Only reads aggregateExt from the first source (sources[0]).
  // When multiple sources have different aggregate settings, only the first is used.
  // Multi-source aggregation is not currently supported — all sources share the same
  // aggregate method. To fix this, each source's data would need independent aggregation
  // before being combined, which would require architectural changes to this pipeline.
  const effectiveAggregate = useMemo<TelemetryAggregate>(() => {
    if (sources.length > 0 && sources[0].aggregateExt) {
      return sources[0].aggregateExt
    }
    return aggregate
  }, [sources, aggregate])

  // Derived states
  const hasData = data !== null && data !== undefined && (Array.isArray(data) ? data.length > 0 : true)
  const showLoading = loading && !hasData

  // Series name helpers
  const getDeviceName = useCallback((deviceId?: string): string => _getDeviceName(deviceId, t), [t])
  const getPropertyDisplayName = useCallback((property?: string): string => _getPropertyDisplayName(property, t), [t])
  const getSeriesName = useCallback((ds: DataSource, idx: number): string => {
    return _getSeriesName(ds, idx, { getDeviceName, getPropertyDisplayName, t })
  }, [getDeviceName, getPropertyDisplayName, t])

  return {
    sources,
    data: data as T,
    loading,
    error,
    effectiveAggregate,
    hasData,
    showLoading,
    getSeriesName,
    getDeviceName,
  }
}
