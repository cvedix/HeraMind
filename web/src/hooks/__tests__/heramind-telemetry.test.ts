import { afterAll, afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { api } from '@/lib/api'
import { clearGlobalCacheIntervals, clearTelemetryCache, fetchHistoricalTelemetry } from '@/hooks/useDataSource/fetch'
import { getComponentMeta } from '@/components/dashboard/registry/registry'

vi.mock('@/store', () => ({ useStore: { getState: () => ({}) } }))
const fetchMock = vi.fn()
const respond = (data: unknown) => fetchMock.mockResolvedValue(new Response(JSON.stringify(data), { status: 200 }))
const query = () => new URL(String(fetchMock.mock.calls[0][0]), 'http://localhost').searchParams

beforeEach(() => {
  vi.stubGlobal('fetch', fetchMock)
  fetchMock.mockReset()
  clearTelemetryCache()
})
afterEach(() => vi.unstubAllGlobals())
afterAll(() => clearGlobalCacheIntervals())

describe('HeraMind telemetry on the updated API', () => {
  it('keeps numeric pagination distinct from an aggregation method', async () => {
    respond({ data: [], count: 0 })
    await api.queryTelemetry('device:camera', 'events', 100, 200, 50, false, 25)
    expect(query().get('offset')).toBe('25')
    expect(query().has('aggregate')).toBe(false)
  })

  it('preserves the existing aggregation argument', async () => {
    respond({ value: 37, count: 37 })
    await api.queryTelemetry('transform:camera', 'events', 100, 200, 50, false, 'sum')
    expect(query().get('aggregate')).toBe('sum')
    expect(query().has('offset')).toBe(false)
  })

  it('uses the server event count instead of counting the single aggregate row', async () => {
    respond({ data: { events: [{ timestamp: 100, value: 37, count: 37 }] } })
    const result = await fetchHistoricalTelemetry('camera', 'events', 1, 50, 'count')
    expect(query().get('aggregate')).toBe('count')
    expect(result).toMatchObject({ success: true, data: [37] })
  })

  it('accepts scalar aggregates from unified transform sources', async () => {
    respond({ value: 42, count: 42 })
    const result = await fetchHistoricalTelemetry('transform:camera', 'events', 1, 50, 'sum')
    expect(result).toMatchObject({ success: true, data: [42] })
  })

  it('keeps EventList available in the replacement dashboard registry', () => {
    const meta = getComponentMeta('event-list')
    expect(meta?.hasDataSource).toBe(true)
    expect(meta?.defaultProps).toMatchObject({ pageSize: 10, showImage: true })
    expect(meta?.sizeConstraints.defaultH).toBeGreaterThanOrEqual(4)
  })
})
