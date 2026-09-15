import { describe, expect, it } from 'vitest'

const english = import.meta.glob<Record<string, unknown>>('../locales/en/*.json', { eager: true, import: 'default' })
const vietnamese = import.meta.glob<Record<string, unknown>>('../locales/vi/*.json', { eager: true, import: 'default' })

function compare(source: Record<string, unknown>, translated: Record<string, unknown>, path: string, issues: string[]) {
  for (const [key, value] of Object.entries(source)) {
    const label = `${path}/${key}`
    const target = translated[key]
    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      compare(value as Record<string, unknown>, (target ?? {}) as Record<string, unknown>, label, issues)
    } else if (typeof value === 'string') {
      if (typeof target !== 'string' || (value.trim() !== '' && !target.trim())) issues.push(`${label}: missing translation`)
      else {
        const placeholders = (s: string) => (s.match(/{{.*?}}/g) ?? []).sort()
        if (JSON.stringify(placeholders(value)) !== JSON.stringify(placeholders(target))) issues.push(`${label}: interpolation mismatch`)
      }
    }
  }
}

describe('HeraMind Vietnamese locale', () => {
  it('covers the upstream UI and preserves interpolation parameters', () => {
    const issues: string[] = []
    for (const [path, source] of Object.entries(english)) {
      compare(source, vietnamese[path.replace('/en/', '/vi/')] ?? {}, path, issues)
    }
    expect(issues).toEqual([])
  })
})
