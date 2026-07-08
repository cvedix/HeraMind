/**
 * Create a stable string key from any value for memoization/comparison purposes.
 * Handles objects with keys in sorted order to ensure stability.
 * Uses a depth limit to prevent stack overflow on deep/circular references.
 */

const MAX_DEPTH = 10

export function createStableKey(obj: unknown, _depth = 0): string {
  if (obj === null || obj === undefined) return ''
  if (_depth >= MAX_DEPTH) return '...'
  if (typeof obj !== 'object') return String(obj)
  if (obj instanceof Date) return obj.toISOString()
  if (Array.isArray(obj)) return '[' + obj.map(v => createStableKey(v, _depth + 1)).join(',') + ']'
  const sortedKeys = Object.keys(obj as Record<string, unknown>).sort()
  return '{' + sortedKeys.map(k => `"${k}":${createStableKey((obj as Record<string, unknown>)[k], _depth + 1)}`).join(',') + '}'
}
