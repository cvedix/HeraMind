/**
 * Batches rapid state updates into a single RAF tick.
 * Collects all pending updates and applies them atomically.
 */
export class BatchUpdater<TUpdate> {
  private pending: Map<string, TUpdate> = new Map()
  private rafId: number | null = null
  private applyFn: (updates: Map<string, TUpdate>) => void

  constructor(applyFn: (updates: Map<string, TUpdate>) => void) {
    this.applyFn = applyFn
  }

  push(key: string, update: TUpdate): void {
    this.pending.set(key, update)
    if (this.rafId === null) {
      this.rafId = requestAnimationFrame(() => this.flush())
    }
  }

  private flush(): void {
    this.rafId = null
    if (this.pending.size === 0) return
    const updates = this.pending
    this.pending = new Map()
    this.applyFn(updates)
  }

  destroy(): void {
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId)
      this.rafId = null
    }
    this.pending.clear()
  }
}
