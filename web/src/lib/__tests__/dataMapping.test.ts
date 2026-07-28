import { describe, expect, it } from 'vitest'
import { DataMapper } from '../dataMapping'

describe('DataMapper.mapToTimeSeries', () => {
  it('keeps raw telemetry points sorted by timestamp', () => {
    const result = DataMapper.mapToTimeSeries([
      { timestamp: 1_700_000_120, value: 3 },
      { timestamp: 1_700_000_000, value: 1 },
    ])

    expect(result).toEqual([
      { timestamp: 1_700_000_000, value: 1, label: 'Item 2' },
      { timestamp: 1_700_000_120, value: 3, label: 'Item 1' },
    ])
  })

  it('sums events per minute and fills idle minutes with zero', () => {
    const minute = 1_700_000_040
    const alignedMinute = Math.floor(minute / 60) * 60
    const result = DataMapper.mapToTimeSeries([
      { timestamp: minute, value: 1 },
      { timestamp: minute + 10, value: 1 },
      { timestamp: minute + 125, value: 1 },
    ], {
      timeAggregate: '1m',
      aggregate: 'sum',
      fillMissingBuckets: true,
    })

    expect(result).toEqual([
      { timestamp: alignedMinute, value: 2 },
      { timestamp: alignedMinute + 60, value: 0 },
      { timestamp: alignedMinute + 120, value: 1 },
    ])
  })

  it('preserves millisecond timestamps when aggregating', () => {
    const minuteMs = 1_700_000_040_000
    const alignedMinuteMs = Math.floor(minuteMs / 60_000) * 60_000
    const result = DataMapper.mapToTimeSeries([
      { timestamp: minuteMs, value: 2 },
      { timestamp: minuteMs + 5_000, value: 3 },
    ], {
      timeAggregate: '1m',
      aggregate: 'sum',
    })

    expect(result).toEqual([
      { timestamp: alignedMinuteMs, value: 5 },
    ])
  })
})
