import { useState, useEffect, useCallback } from 'react'
import { api } from '@/lib/api'
import { useErrorHandler } from '@/hooks/useErrorHandler'

/**
 * Fetches all transforms once on mount and computes per-device / per-device-type counts.
 * Call this ONCE in the parent list component and pass counts down to TransformsBadge.
 *
 * Does NOT use fetchCache — the computed counts live in component-local state which is
 * destroyed on unmount. fetchCache only stores timestamps, so a cache hit after remount
 * would skip the fetch but leave the state empty (all counts = 0).
 */
export function useTransformCounts() {
  const { handleError } = useErrorHandler()
  const [deviceCounts, setDeviceCounts] = useState<Record<string, number>>({})
  const [deviceTypeCounts, setDeviceTypeCounts] = useState<Record<string, number>>({})

  const fetchCounts = useCallback(async () => {
    try {
      const result = await api.listTransforms()
      const dMap: Record<string, number> = {}
      const tMap: Record<string, number> = {}

      for (const tr of result.transforms || []) {
        if (typeof tr.scope === 'object' && tr.scope !== null) {
          if ('device' in tr.scope && tr.scope.device) {
            dMap[tr.scope.device] = (dMap[tr.scope.device] || 0) + 1
          } else if ('device_type' in tr.scope && tr.scope.device_type) {
            tMap[tr.scope.device_type] = (tMap[tr.scope.device_type] || 0) + 1
          }
        }
      }

      setDeviceCounts(dMap)
      setDeviceTypeCounts(tMap)
    } catch (error) {
      handleError(error, { operation: 'Fetch transform counts', showToast: false })
    }
  }, [handleError])

  useEffect(() => {
    fetchCounts()
  }, [fetchCounts])

  const refresh = useCallback(() => {
    fetchCounts()
  }, [fetchCounts])

  return { deviceCounts, deviceTypeCounts, refresh }
}
