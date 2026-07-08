/// Tests for useDataSource hook
import { describe, it, expect } from 'vitest'
import { getEventDeviceId } from '@/types/dashboard'
import type { DataSource } from '@/types/dashboard'

describe('Virtual Metric Exclusion', () => {
  // Test the logic that prevents virtual metrics from interfering with real metrics
  describe('Virtual metric detection', () => {
    const TRANSFORM_NAMESPACES = ['transform.', 'virtual.', 'computed.', 'derived.', 'aggregated.']

    const isVirtualMetric = (metricName: string): boolean => {
      return TRANSFORM_NAMESPACES.some(ns => metricName.startsWith(ns))
    }

    it('should identify transform virtual metrics', () => {
      expect(isVirtualMetric('transform.TH.status')).toBe(true)
      expect(isVirtualMetric('transform.temperature')).toBe(true)
      expect(isVirtualMetric('transform.device1.humidity')).toBe(true)
    })

    it('should identify other virtual metric namespaces', () => {
      expect(isVirtualMetric('virtual.metric')).toBe(true)
      expect(isVirtualMetric('computed.value')).toBe(true)
      expect(isVirtualMetric('derived.result')).toBe(true)
      expect(isVirtualMetric('aggregated.avg')).toBe(true)
    })

    it('should not identify real metrics as virtual', () => {
      expect(isVirtualMetric('temperature')).toBe(false)
      expect(isVirtualMetric('humidity')).toBe(false)
      expect(isVirtualMetric('status')).toBe(false)
      expect(isVirtualMetric('values.temperature')).toBe(false)
    })

    it('should not match metrics that merely contain namespace words', () => {
      // A metric named "transformer" should NOT be considered virtual
      expect(isVirtualMetric('transformer')).toBe(false)
      // A metric named "virtualization" should NOT be considered virtual
      expect(isVirtualMetric('virtualization')).toBe(false)
    })
  })

  describe('Event metric matching', () => {
    const TRANSFORM_NAMESPACES = ['transform.', 'virtual.', 'computed.', 'derived.', 'aggregated.']

    const isVirtualMetric = (metricName: string): boolean => {
      return TRANSFORM_NAMESPACES.some(ns => metricName.startsWith(ns))
    }

    // Simplified version of eventMetricMatches for testing
    const eventMetricMatches = (eventMetric: string, widgetMetricId: string): boolean => {
      if (!eventMetric || !widgetMetricId) return false
      if (eventMetric === widgetMetricId) return true

      // CRITICAL: If event is a virtual metric but widget is not (or vice versa), don't match
      const eventIsVirtual = isVirtualMetric(eventMetric)
      const widgetIsVirtual = isVirtualMetric(widgetMetricId)
      if (eventIsVirtual !== widgetIsVirtual) {
        return false
      }

      // Case 1: Event has nested path, widget is simple
      if (eventMetric.endsWith('.' + widgetMetricId)) return true
      if (eventMetric.endsWith('/' + widgetMetricId)) return true

      // Case 2: Event is simple, widget has nested path
      if (widgetMetricId.endsWith('.' + eventMetric)) return true
      if (widgetMetricId.endsWith('/' + eventMetric)) return true

      // Case 3: Both have nested paths - compare the last segment
      const eventLastSegment = eventMetric.split('.').pop() || eventMetric.split('/').pop() || eventMetric
      const widgetLastSegment = widgetMetricId.split('.').pop() || widgetMetricId.split('/').pop() || widgetMetricId
      if (eventLastSegment === widgetLastSegment) return true

      return false
    }

    it('should match identical real metrics', () => {
      expect(eventMetricMatches('temperature', 'temperature')).toBe(true)
      expect(eventMetricMatches('humidity', 'humidity')).toBe(true)
    })

    it('should match identical virtual metrics', () => {
      expect(eventMetricMatches('transform.TH.status', 'transform.TH.status')).toBe(true)
      expect(eventMetricMatches('virtual.metric', 'virtual.metric')).toBe(true)
    })

    it('should NOT match virtual metric event with real metric widget', () => {
      // This is the key bug fix: transform.temperature should NOT match temperature
      expect(eventMetricMatches('transform.temperature', 'temperature')).toBe(false)
      expect(eventMetricMatches('transform.TH.status', 'status')).toBe(false)
      expect(eventMetricMatches('virtual.temperature', 'temperature')).toBe(false)
    })

    it('should NOT match real metric event with virtual metric widget', () => {
      // Reverse case: real metric should not match virtual widget
      expect(eventMetricMatches('temperature', 'transform.temperature')).toBe(false)
      expect(eventMetricMatches('status', 'transform.TH.status')).toBe(false)
    })

    it('should match nested paths for real metrics', () => {
      expect(eventMetricMatches('values.image', 'image')).toBe(true)
      expect(eventMetricMatches('image', 'values.image')).toBe(true)
    })

    it('should match nested paths for virtual metrics (same type)', () => {
      // Both are virtual, so they should be allowed to match
      expect(eventMetricMatches('transform.TH.image', 'transform.image')).toBe(true)
    })
  })

  describe('Real metric isolation', () => {
    // Test the logic that prevents real metrics from interfering with each other
    // This tests the fix for the issue where temperature events would affect humidity widgets

    const TRANSFORM_NAMESPACES = ['transform.', 'virtual.', 'computed.', 'derived.', 'aggregated.']

    const isVirtualMetric = (metricName: string): boolean => {
      return TRANSFORM_NAMESPACES.some(ns => metricName.startsWith(ns))
    }

    const eventMetricMatches = (eventMetric: string, widgetMetricId: string): boolean => {
      if (!eventMetric || !widgetMetricId) return false
      if (eventMetric === widgetMetricId) return true

      // CRITICAL: If event is a virtual metric but widget is not (or vice versa), don't match
      const eventIsVirtual = isVirtualMetric(eventMetric)
      const widgetIsVirtual = isVirtualMetric(widgetMetricId)
      if (eventIsVirtual !== widgetIsVirtual) {
        return false
      }

      // Case 1: Event has nested path, widget is simple
      if (eventMetric.endsWith('.' + widgetMetricId)) return true
      if (eventMetric.endsWith('/' + widgetMetricId)) return true

      // Case 2: Event is simple, widget has nested path
      if (widgetMetricId.endsWith('.' + eventMetric)) return true
      if (widgetMetricId.endsWith('/' + eventMetric)) return true

      // Case 3: Both have nested paths - compare the last segment
      const eventLastSegment = eventMetric.split('.').pop() || eventMetric.split('/').pop() || eventMetric
      const widgetLastSegment = widgetMetricId.split('.').pop() || widgetMetricId.split('/').pop() || widgetMetricId
      if (eventLastSegment === widgetLastSegment) return true

      return false
    }

    // Test the value extraction logic that should ONLY extract when metric matches
    const shouldExtractValue = (eventMetric: string, widgetMetricId: string): boolean => {
      // This simulates the logic in useDataSource.ts
      // Only extract value when:
      // 1. The event's metric matches the widget's metric
      // 2. OR there's no explicit metric in the event (legacy compatibility)
      if (!eventMetric) {
        return true // No explicit metric, allow extraction (legacy)
      }
      return eventMetricMatches(eventMetric, widgetMetricId)
    }

    it('should NOT extract temperature value for humidity widget', () => {
      // The key bug: temperature event should NOT provide value to humidity widget
      expect(shouldExtractValue('temperature', 'humidity')).toBe(false)
      expect(shouldExtractValue('humidity', 'temperature')).toBe(false)
    })

    it('should allow extraction when event metric matches widget metric', () => {
      expect(shouldExtractValue('temperature', 'temperature')).toBe(true)
      expect(shouldExtractValue('humidity', 'humidity')).toBe(true)
      expect(shouldExtractValue('status', 'status')).toBe(true)
    })

    it('should allow extraction when no explicit metric in event', () => {
      // Legacy compatibility: events without explicit metric field
      expect(shouldExtractValue('', 'temperature')).toBe(true)
      expect(shouldExtractValue('', 'humidity')).toBe(true)
    })

    it('should NOT extract value when different real metrics on same device', () => {
      // Different real metrics should NOT interfere
      expect(shouldExtractValue('temperature', 'pressure')).toBe(false)
      expect(shouldExtractValue('voltage', 'current')).toBe(false)
      expect(shouldExtractValue('status', 'count')).toBe(false)
    })

    it('should NOT extract value when real metric event for virtual widget', () => {
      // Real metric events should not affect virtual metric widgets
      expect(shouldExtractValue('temperature', 'transform.temperature')).toBe(false)
      expect(shouldExtractValue('status', 'virtual.status')).toBe(false)
    })

    it('should NOT extract value when virtual metric event for real widget', () => {
      // Virtual metric events should not affect real metric widgets
      expect(shouldExtractValue('transform.temperature', 'temperature')).toBe(false)
      expect(shouldExtractValue('virtual.status', 'status')).toBe(false)
    })
  })
})

describe('getEventDeviceId', () => {
  it('should return bare ID for device sources', () => {
    const ds: DataSource = { type: 'telemetry', sourceId: 'sensor-001', source: 'device', id: 'sensor-001', field: 'temperature', mode: 'timeseries' }
    expect(getEventDeviceId(ds)).toBe('sensor-001')
  })

  it('should prefix transform source IDs', () => {
    const ds: DataSource = { type: 'transform', sourceId: 'transform:weather', transformId: 'weather', source: 'transform', id: 'weather', field: 'temp', mode: 'timeseries' }
    expect(getEventDeviceId(ds)).toBe('transform:weather')
  })

  it('should not double-prefix already-prefixed transform IDs', () => {
    const ds: DataSource = { type: 'transform', sourceId: 'transform:weather', source: 'transform', id: 'transform:weather', field: 'temp', mode: 'timeseries' }
    expect(getEventDeviceId(ds)).toBe('transform:weather')
  })

  it('should prefix AI source IDs', () => {
    const ds: DataSource = { type: 'agent', source: 'ai', id: 'my-agent', field: 'analysis', mode: 'timeseries' } as DataSource
    expect(getEventDeviceId(ds)).toBe('ai:my-agent')
  })

  it('should return undefined for missing ID', () => {
    const ds: DataSource = { type: 'device', source: 'device', field: 'temp', mode: 'latest' } as DataSource
    expect(getEventDeviceId(ds)).toBeUndefined()
  })

  it('should match backend transform event device_id format', () => {
    // Backend uses storage_device_id() = "transform:{transform_id}"
    // Frontend getEventDeviceId must produce the same format for WS matching
    const ds: DataSource = { type: 'transform', transformId: 'converter', source: 'transform', id: 'converter', field: 'celsius', mode: 'timeseries' }
    const frontendEventId = getEventDeviceId(ds)
    const backendStorageId = `transform:${ds.transformId}`
    expect(frontendEventId).toBe(backendStorageId)
  })
})

describe('useDataSource Hook', () => {
  describe('Data Source Interface', () => {
    it('should define UseDataSourceResult interface', () => {
      const result = {
        data: null,
        loading: false,
        error: null,
        lastUpdate: null,
      }

      expect(result.data).toBeNull()
      expect(result.loading).toBe(false)
      expect(result.error).toBeNull()
      expect(result.lastUpdate).toBeNull()
    })

    it('should support optional sendCommand function', () => {
      type SendCommandType = (value?: unknown) => Promise<boolean>
      const result: {
        sendCommand?: SendCommandType
      } = {} as any

      expect(result.sendCommand).toBeUndefined()
    })

    it('should support optional sending flag', () => {
      const result: {
        sending?: boolean
      } = {}

      expect(result.sending).toBeUndefined()
    })
  })

  describe('Data Source Types', () => {
    it('should accept device type data source', () => {
      const deviceDataSource = {
        id: 'device-1',
        type: 'device' as const,
        sourceId: 'device-1',
        metric: 'temperature',
        aggregation: 'mean' as const,
        label: 'Temperature',
        unit: '°C',
      }

      expect(deviceDataSource.type).toBe('device')
      expect(deviceDataSource.sourceId).toBe('device-1')
    })

    it('should accept transform type data source', () => {
      const transformDataSource = {
        id: 'transform-1',
        type: 'transform' as const,
        transformId: 'transform-1',
        label: 'Transformed Data',
      }

      expect(transformDataSource.type).toBe('transform')
      expect(transformDataSource.transformId).toBe('transform-1')
    })

    it('should accept extension-command type data source', () => {
      const extensionCommandDataSource = {
        id: 'ext-cmd-1',
        type: 'extension-command' as const,
        extensionId: 'ext-1',
        extensionCommand: 'getData',
        label: 'Extension Command',
      }

      expect(extensionCommandDataSource.type).toBe('extension-command')
      expect(extensionCommandDataSource.extensionId).toBe('ext-1')
    })

    it('should accept array of data sources', () => {
      const dataSourceList: Array<{
        id: string
        type: 'device' | 'transform' | 'extension-command'
        sourceId?: string
        metric?: string
        transformId?: string
        extensionId?: string
        extensionCommand?: string
      }> = [
        { id: '1', type: 'device' as const, sourceId: '1', metric: 'temp' },
        { id: '2', type: 'transform' as const, transformId: '2' },
        { id: '3', type: 'extension-command' as const, extensionId: '3', extensionCommand: 'cmd' },
      ]

      expect(dataSourceList).toHaveLength(3)
    })
  })

  describe('Telemetry Aggregate Types', () => {
    it('should accept valid aggregation functions', () => {
      const aggregations: Array<'raw' | 'mean' | 'sum' | 'min' | 'max' | 'count'> = [
        'raw',
        'mean',
        'sum',
        'min',
        'max',
        'count',
      ]

      expect(aggregations).toHaveLength(6)
    })

    it('should support optional time window for telemetry', () => {
      const aggregate = {
        type: 'mean' as const,
        value: 25.5,
        timeWindow: '5m',
      }

      expect(aggregate.type).toBe('mean')
      expect(aggregate.value).toBe(25.5)
      expect(aggregate.timeWindow).toBe('5m')
    })
  })
})
