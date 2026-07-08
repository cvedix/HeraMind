import { useEffect, useRef, useCallback } from 'react'

/**
 * Like setInterval, but automatically pauses when the tab is hidden
 * and resumes (with an immediate refresh) when it becomes visible again.
 *
 * @param callback - function to call on each tick
 * @param intervalMs - polling interval in milliseconds
 * @param enabled - set to false to disable polling entirely
 */
export function useVisiblePolling(
  callback: () => void,
  intervalMs: number,
  enabled = true,
) {
  const callbackRef = useRef(callback)
  callbackRef.current = callback

  const intervalIdRef = useRef<ReturnType<typeof setInterval> | null>(null)

  const startPolling = useCallback(() => {
    if (intervalIdRef.current !== null) return
    intervalIdRef.current = setInterval(() => {
      callbackRef.current()
    }, intervalMs)
  }, [intervalMs])

  const stopPolling = useCallback(() => {
    if (intervalIdRef.current !== null) {
      clearInterval(intervalIdRef.current)
      intervalIdRef.current = null
    }
  }, [])

  useEffect(() => {
    if (!enabled) {
      stopPolling()
      return
    }

    // Start polling immediately
    startPolling()

    const handleVisibility = () => {
      if (document.hidden) {
        stopPolling()
      } else {
        // Tab became visible — trigger immediate refresh, then restart interval
        callbackRef.current()
        startPolling()
      }
    }

    document.addEventListener('visibilitychange', handleVisibility)
    return () => {
      document.removeEventListener('visibilitychange', handleVisibility)
      stopPolling()
    }
  }, [enabled, startPolling, stopPolling])
}
