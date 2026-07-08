/**
 * useBodyScrollLock Hook
 *
 * Prevents body scroll when a modal/dialog is open on mobile devices.
 * This prevents layout shifts and scroll position issues.
 *
 * @example
 * ```tsx
 * function MyModal({ isOpen, onClose }) {
 *   useBodyScrollLock(isOpen)
 *   return <dialog open={isOpen}>...</dialog>
 * }
 * ```
 */

import { useEffect, useRef } from 'react'

interface BodyScrollLockOptions {
  /**
   * Whether to lock scroll only on mobile devices
   * @default true - only lock on screens smaller than 768px
   */
  mobileOnly?: boolean

  /**
   * Breakpoint for mobile detection
   * @default 768
   */
  mobileBreakpoint?: number
}

/**
 * Hook to lock body scroll when a component is mounted
 *
 * This is useful for modals, dialogs, and full-screen overlays.
 * It saves the scroll position and restores it when unlocked.
 */
export function useBodyScrollLock(
  isLocked: boolean,
  options: BodyScrollLockOptions = {}
) {
  const { mobileOnly = true, mobileBreakpoint = 768 } = options
  const scrollPosition = useRef(0)
  const isLockedRef = useRef(false)

  useEffect(() => {
    // Check if should apply lock (based on mobileOnly option)
    const shouldLock = mobileOnly
      ? window.innerWidth < mobileBreakpoint
      : true

    if (!shouldLock) return
    if (!isLocked) {
      // Unlock if was locked
      if (isLockedRef.current) {
        document.body.style.overflow = ''
        document.body.style.position = ''
        document.body.style.top = ''
        document.body.style.width = ''
        window.scrollTo(0, scrollPosition.current)
        isLockedRef.current = false
      }
      return
    }

    // Lock body scroll
    if (!isLockedRef.current) {
      scrollPosition.current = window.scrollY

      document.body.style.overflow = 'hidden'
      document.body.style.position = 'fixed'
      document.body.style.top = `-${scrollPosition.current}px`
      document.body.style.width = '100%'

      isLockedRef.current = true
    }

    // Unlock on cleanup
    return () => {
      if (isLockedRef.current) {
        document.body.style.overflow = ''
        document.body.style.position = ''
        document.body.style.top = ''
        document.body.style.width = ''
        window.scrollTo(0, scrollPosition.current)
        isLockedRef.current = false
      }
    }
  }, [isLocked, mobileOnly, mobileBreakpoint])
}

/**
 * Hook specifically for locking body scroll on mobile devices
 * Convenience wrapper for useBodyScrollLock with mobileOnly=true
 */
export function useMobileBodyScrollLock(isLocked: boolean) {
  return useBodyScrollLock(isLocked, { mobileOnly: true })
}

export default useBodyScrollLock
