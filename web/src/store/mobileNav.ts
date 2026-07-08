/**
 * Mobile navigation drawer state (module-level store).
 *
 * Decoupled from the main HeraMind store so any component (page headers,
 * the MobileNav drawer, etc.) can open the hamburger menu without prop
 * drilling or context providers.
 */
import { create } from "zustand"

interface MobileNavState {
  open: boolean
  setOpen: (open: boolean) => void
  toggle: () => void
}

export const useMobileNav = create<MobileNavState>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
  toggle: () => set((s) => ({ open: !s.open })),
}))
