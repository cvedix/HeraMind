/**
 * Async operation utilities
 */

/**
 * Debounce function execution
 *
 * @example
 * const debouncedSearch = debounce((query: string) => {
 *   console.log('Searching for:', query)
 * }, 300)
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null

  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

/**
 * Throttle function execution
 *
 * @example
 * const throttledScroll = throttle(() => {
 *   console.log('Scrolling')
 * }, 100)
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean = false

  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args)
      inThrottle = true
      setTimeout(() => inThrottle = false, limit)
    }
  }
}

/**
 * Retry async function with exponential backoff
 *
 * @example
 * const result = await retry(
 *   () => fetch('/api/data').then(r => r.json()),
 *   3,
 *   1000
 * )
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  initialDelay: number = 1000
): Promise<T> {
  let lastError: Error | null = null

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error as Error
      if (attempt < maxRetries) {
        const delay = initialDelay * Math.pow(2, attempt)
        await new Promise(resolve => setTimeout(resolve, delay))
      }
    }
  }

  throw lastError || new Error('Max retries exceeded')
}

/**
 * Add timeout to a promise
 *
 * @example
 * const result = await withTimeout(
 *   fetch('/api/data'),
 *   5000
 * )
 */
export async function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number
): Promise<T> {
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => reject(new Error('Operation timeout')), timeoutMs)
  })

  return Promise.race([promise, timeoutPromise])
}

/**
 * Delay execution
 *
 * @example
 * await delay(1000) // Wait 1 second
 */
export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * Poll a function until it returns true or timeout
 *
 * @example
 * const success = await poll(
 *   () => document.querySelector('.loaded') !== null,
 *   5000,
 *   500
 * )
 */
export async function poll(
  condition: () => boolean | Promise<boolean>,
  timeoutMs: number = 5000,
  intervalMs: number = 500
): Promise<boolean> {
  const startTime = Date.now()

  while (Date.now() - startTime < timeoutMs) {
    if (await condition()) {
      return true
    }
    await delay(intervalMs)
  }

  return false
}

/**
 * TTL-based fetch deduplication cache.
 *
 * Prevents redundant API calls when the same data was fetched recently.
 * Shared across all store slices — use a unique `key` per fetch action.
 *
 * @example
 * // Inside a Zustand slice:
 * fetchDevices: async () => {
 *   if (!fetchCache.shouldFetch('devices')) return
 *   fetchCache.markFetching('devices')
 *   try {
 *     const data = await api.getDevices()
 *     set({ devices: data.devices })
 *     fetchCache.markFetched('devices')
 *   } catch { fetchCache.invalidate('devices') }
 * }
 */
class FetchCache {
  private cache = new Map<string, { timestamp: number; fetching: boolean }>()

  /** Returns true if the key should be fetched (stale or never fetched). */
  shouldFetch(key: string, ttlMs: number = 10_000): boolean {
    const entry = this.cache.get(key)
    if (!entry) return true
    if (entry.fetching) return false // already in-flight
    return Date.now() - entry.timestamp > ttlMs
  }

  /** Mark a key as currently being fetched (prevents concurrent duplicate calls). */
  markFetching(key: string): void {
    this.cache.set(key, { timestamp: 0, fetching: true })
  }

  /** Mark a key as successfully fetched with the given TTL. */
  markFetched(key: string): void {
    this.cache.set(key, { timestamp: Date.now(), fetching: false })
  }

  /** Invalidate a specific key (force next fetch to go through). */
  invalidate(key: string): void {
    this.cache.delete(key)
  }

  /** Invalidate all keys. */
  invalidateAll(): void {
    this.cache.clear()
  }
}

export const fetchCache = new FetchCache()
