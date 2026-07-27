import { describe, expect, it } from 'vitest'
import type { DataSource } from '@/types/dashboard'
import { processTelemetryEvent } from '../useDataSource/eventProcessors'

describe('processTelemetryEvent aggregate sources', () => {
  const countSource: DataSource = {
    type: 'telemetry',
    sourceId: 'camera-1',
    metricId: '_raw',
    source: 'device',
    id: 'camera-1',
    field: '_raw',
    mode: 'timeseries',
    timeRange: 24,
    limit: 1,
    aggregateExt: 'count',
  }

  it('increments a server-side count when a matching live event arrives', () => {
    let data: unknown = [37]
    let lastUpdate = 0

    processTelemetryEvent(
      { value: { event: 'bodycam' }, timestamp: Math.floor(Date.now() / 1000) },
      '_raw',
      'camera-1',
      [countSource],
      false,
      undefined,
      updater => { data = updater(data) },
      timestamp => { lastUpdate = timestamp },
    )

    expect(data).toEqual([38])
    expect(lastUpdate).toBeGreaterThan(0)
  })

  it('does not increment for a different metric', () => {
    let data: unknown = [37]

    processTelemetryEvent(
      { value: 25, timestamp: Math.floor(Date.now() / 1000) },
      'temperature',
      'camera-1',
      [countSource],
      false,
      undefined,
      updater => { data = updater(data) },
      () => {},
    )

    expect(data).toEqual([37])
  })
})
