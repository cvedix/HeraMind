/**
 * MobileHeaderActionsContext - lets children register action nodes that
 * surface in two host slots on mobile:
 *
 *   - "header"  → MobilePageHeader's actions row (icon buttons + overflow).
 *                 Used for primary actions (Add, Refresh, …) that belong in
 *                 the top app bar.
 *   - "content" → a sticky toolbar at the top of the scroll container, below
 *                 the header. Used for search/filter controls and other wide
 *                 inputs that don't fit in the header's icon-only slot.
 *
 * Primary use case: PageTabsBar wants its tab actions to render in the
 * MobilePageHeader, while its `actionsExtra` (search input, filter popover,
 * import/export menu) should stay with the content. Because PageTabsBar lives
 * inside PageLayout.headerContent and MobilePageHeader is a sibling, they
 * communicate via this context, which PageLayout provides.
 *
 * PageLayout creates the registry via `useMobileHeaderActionsRegistry()`,
 * passes the value through the Provider, and forwards:
 *   - header-collected nodes → MobilePageHeader's `actions` slot
 *   - content-collected nodes → sticky toolbar above the scroll content
 *
 * Registrations are keyed by an arbitrary id so multiple children can register
 * without colliding, and unregister on cleanup.
 */

import {
  createContext,
  useCallback,
  useMemo,
  useState,
  useContext,
  type ReactNode,
} from "react"

export type MobileActionsSlot = "header" | "content"

export interface MobileHeaderActionsContextValue {
  /** Register a node under `id` in the given slot. Returns an unregister function. */
  register: (slot: MobileActionsSlot, id: string, node: ReactNode) => () => void
}

const MobileHeaderActionsContext = createContext<MobileHeaderActionsContextValue | null>(null)

export function useMobileHeaderActionsRegistry() {
  // Two independent registries so a single child can push primary actions to
  // the header AND wide controls (search/filter) to the content toolbar.
  const [headerEntries, setHeaderEntries] = useState<Record<string, ReactNode>>({})
  const [contentEntries, setContentEntries] = useState<Record<string, ReactNode>>({})

  const register = useCallback(
    (slot: MobileActionsSlot, id: string, node: ReactNode) => {
      const setEntries = slot === "header" ? setHeaderEntries : setContentEntries
      setEntries((prev) => ({ ...prev, [id]: node }))
      return () =>
        setEntries((prev) => {
          if (!(id in prev)) return prev
          const next = { ...prev }
          delete next[id]
          return next
        })
    },
    [],
  )

  const value = useMemo<MobileHeaderActionsContextValue>(
    () => ({ register }),
    [register],
  )

  // Order: insertion-order of object keys (stable for V8).
  const collectedHeader = Object.values(headerEntries)
  const collectedContent = Object.values(contentEntries)

  return { value, collectedHeader, collectedContent }
}

export function useMobileHeaderActionsRegistrar() {
  return useContext(MobileHeaderActionsContext)
}

export { MobileHeaderActionsContext }
