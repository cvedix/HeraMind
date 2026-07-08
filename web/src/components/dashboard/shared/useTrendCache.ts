/**
 * Trend Cache Hook
 *
 * Simple hook wrapping a module-level Map for caching trend calculations.
 * Persists across component remounts, with TTL-based expiry.
 */

import { useRef } from 'react'

interface TrendCacheEntry {
  direction: 'up' | 'down' | 'neutral' | null
  value: number
  timestamp: number
  dataHash: string
}

const TTL = 60000 // 60 seconds
const MAX_SIZE = 100

const cache = new Map<string, TrendCacheEntry>()

export function useTrendCache() {
  // Hook exists to provide a stable API; the cache itself is module-level
  const cacheRef = useRef(cache)

  const getCached = (cacheKey: string, dataHash: string): TrendCacheEntry | null => {
    if (!cacheKey) return null
    const cached = cacheRef.current.get(cacheKey)
    if (cached && cached.dataHash === dataHash && Date.now() - cached.timestamp < TTL) {
      return cached
    }
    return null
  }

  const setCached = (cacheKey: string, dataHash: string, direction: 'up' | 'down' | 'neutral' | null, value: number): void => {
    if (!cacheKey) return
    if (cacheRef.current.size >= MAX_SIZE) {
      const firstKey = cacheRef.current.keys().next().value
      if (firstKey) cacheRef.current.delete(firstKey)
    }
    cacheRef.current.set(cacheKey, {
      direction,
      value,
      timestamp: Date.now(),
      dataHash,
    })
  }

  const getLastCached = (cacheKey: string): TrendCacheEntry | null => {
    if (!cacheKey) return null
    return cacheRef.current.get(cacheKey) ?? null
  }

  return { getCached, setCached, getLastCached }
}
