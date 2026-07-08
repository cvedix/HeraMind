import { useRef, useCallback, useEffect } from 'react'

/**
 * Hook that provides an AbortController that auto-aborts on unmount.
 * Use for cancelling in-flight fetch requests when navigating away.
 *
 * @example
 * ```tsx
 * const getSignal = useAbortController()
 * const res = await fetch('/api/data', { signal: getSignal() })
 * ```
 */
export function useAbortController() {
  const controllerRef = useRef<AbortController | null>(null)

  const getSignal = useCallback(() => {
    // Abort any previous in-flight request
    controllerRef.current?.abort()
    const controller = new AbortController()
    controllerRef.current = controller
    return controller.signal
  }, [])

  useEffect(() => {
    return () => {
      controllerRef.current?.abort()
    }
  }, [])

  return getSignal
}
