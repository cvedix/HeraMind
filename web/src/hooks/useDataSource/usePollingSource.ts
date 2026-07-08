/**
 * usePollingSource — generic HTTP polling for sources without real-time WS.
 *
 * Handles: system stats, rule lists, message lists, external APIs, etc.
 * Source-specific fetch logic lives in pollDataSource (fetch.ts).
 *
 * Supports two data modes:
 * - latest/list: replaces data on each poll (single value or array)
 * - timeseries: accumulates {timestamp, value} points, pruned by timeRange + limit
 */

import { useEffect, useRef } from 'react'
import type { DataSource } from '@/types/dashboard'
import { logError } from '@/lib/errors'
import { pollDataSource } from './fetch'

export interface PollingSourceState {
  setData: (value: unknown | ((prev: unknown) => unknown)) => void
  setDataRaw: (d: unknown) => void
  setLoading: (l: boolean) => void
  setError: (e: string | null) => void
  setLastUpdate: (ts: number | null) => void
  optionsRef: React.MutableRefObject<{
    enabled: boolean
    transform?: (data: unknown) => unknown
    fallback?: unknown
    preserveMultiple: boolean
  }>
  sourceAdapters?: {
    startLoading: () => void
    finishLoading: () => void
    failLoading: (error: string) => void
  }
}

/** Merge a new value into a timeseries point array, pruning by time range and limit. */
function mergePoint(
  prev: unknown,
  newValue: unknown,
  now: number,
  timeRangeHours: number,
  maxLimit: number
): unknown[] {
  const newPoint = { timestamp: now, time: now, value: newValue }
  if (!Array.isArray(prev)) return [newPoint]

  const merged = [...(prev as unknown[]), newPoint]

  // Prune points outside time range
  const cutoff = now - timeRangeHours * 3600
  const pruned = merged.filter(p => {
    const ts = (p as Record<string, unknown>).timestamp as number ?? 0
    return ts >= cutoff
  })

  // Prune to max limit (keep newest)
  return pruned.length > maxLimit ? pruned.slice(-maxLimit) : pruned
}

export function usePollingSource(
  sources: DataSource[],
  sourceKey: string,
  enabled: boolean,
  state: PollingSourceState
): void {
  const initialDoneRef = useRef(false)
  const prevKeyRef = useRef('')
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const fetchGenerationRef = useRef(0)

  useEffect(() => {
    if (sources.length === 0 || !enabled) {
      if (intervalRef.current) { clearInterval(intervalRef.current); intervalRef.current = null }
      return
    }

    // Reset when config changes
    if (prevKeyRef.current !== sourceKey) {
      initialDoneRef.current = false
      prevKeyRef.current = sourceKey
    }

    const currentGeneration = ++fetchGenerationRef.current

    const fetchData = async () => {
      // Discard stale invocations
      if (fetchGenerationRef.current !== currentGeneration) return

      if (!initialDoneRef.current) {
        if (state.sourceAdapters) state.sourceAdapters.startLoading()
        else state.setLoading(true)
      }
      state.setError(null)

      try {
        const results = await Promise.all(
          sources.map(async (ds) => {
            try {
              const data = await pollDataSource(ds)
              return { data, success: true }
            } catch {
              return { data: null, success: false }
            }
          })
        )

        // Discard stale results
        if (fetchGenerationRef.current !== currentGeneration) return

        const { transform: transformFn } = state.optionsRef.current
        const hasAccumulation = sources.some(ds => ds.mode === 'timeseries')

        if (hasAccumulation) {
          // Timeseries mode: accumulate points with setData(prev => ...) to avoid races
          state.setData((prevData: unknown) => {
            const now = Math.floor(Date.now() / 1000)

            if (sources.length === 1) {
              const ds = sources[0]
              const resultData = results[0]?.data ?? null
              let finalData: unknown

              if (ds.mode === 'timeseries') {
                finalData = mergePoint(prevData, resultData, now, ds.timeRange ?? 1, ds.limit ?? 50)
              } else {
                finalData = resultData
              }

              return transformFn ? transformFn(finalData) : finalData
            }

            // Multiple sources: each slot handled independently
            const finalData = sources.map((ds, i) => {
              const resultData = results[i]?.data ?? null

              if (ds.mode === 'timeseries') {
                const prev = Array.isArray(prevData) ? (prevData as unknown[])[i] : undefined
                return mergePoint(prev, resultData, now, ds.timeRange ?? 1, ds.limit ?? 50)
              }
              return resultData
            })

            return transformFn ? transformFn(finalData) : finalData
          })
        } else {
          // All latest/list mode: direct set (no accumulation)
          let finalData: unknown
          if (results.length > 1) finalData = results.map((r) => r.data)
          else finalData = results[0]?.data ?? null

          const transformedData = transformFn ? transformFn(finalData) : finalData
          state.setDataRaw(transformedData)
        }

        state.setLastUpdate(Date.now())
        initialDoneRef.current = true
      } catch (err) {
        if (fetchGenerationRef.current !== currentGeneration) return
        logError(err, { operation: 'Fetch polling data' })
        state.setError(err instanceof Error ? err.message : 'Failed to fetch data')
        state.setDataRaw(state.optionsRef.current.fallback ?? null)
        initialDoneRef.current = true
      } finally {
        if (fetchGenerationRef.current === currentGeneration) {
          if (state.sourceAdapters) state.sourceAdapters.finishLoading()
          else state.setLoading(false)
        }
      }
    }

    if (intervalRef.current) { clearInterval(intervalRef.current); intervalRef.current = null }
    fetchData()

    // Periodic refresh
    const refreshIntervals = sources.map((ds) => ds.refresh).filter(Boolean) as number[]
    const minRefresh = refreshIntervals.length > 0
      ? refreshIntervals.reduce((a, b) => Math.min(a, b), Infinity)
      : null
    if (minRefresh) {
      intervalRef.current = setInterval(fetchData, minRefresh * 1000)
    }

    return () => {
      if (intervalRef.current) { clearInterval(intervalRef.current); intervalRef.current = null }
    }
  }, [sourceKey, enabled])
}
