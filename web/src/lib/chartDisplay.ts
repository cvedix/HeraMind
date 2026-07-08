/**
 * Shared display name helpers for chart components.
 */

import type { DataSource } from '@/types/dashboard'
import { getSourceId } from '@/types/dashboard'

/**
 * Format a device ID into a human-readable name.
 */
export function getDeviceName(deviceId: string | undefined, t: (key: string) => string): string {
  if (!deviceId) return t('chart.value')
  return deviceId.replace(/[-_]/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

/**
 * Get a user-friendly display name for a telemetry property.
 */
export function getPropertyDisplayName(property: string | undefined, t: (key: string) => string): string {
  if (!property) return t('chart.value')
  const propertyNames: Record<string, string> = {
    temperature: t('chart.temperature'),
    humidity: t('chart.humidity'),
    temp: t('chart.temperature'),
    value: t('chart.value'),
  }
  return propertyNames[property] || property.replace(/[-_]/g, ' ')
}

/**
 * Get the display name for a data source series.
 * Handles extension and standard data sources.
 */
export function getSeriesName(
  ds: DataSource,
  idx: number,
  deps: {
    getDeviceName: (id: string | undefined) => string
    getPropertyDisplayName: (prop: string | undefined) => string
    t: (key: string, params?: Record<string, unknown>) => string
  }
): string {
  const { getDeviceName: _getDeviceName, getPropertyDisplayName: _getPropName, t } = deps

  if (ds.type === 'extension' || ds.type === 'extension-metric') {
    const extName = ds.extensionDisplayName || ds.extensionId || ''
    const metricPart = ds.extensionMetric
      ? _getPropName(ds.extensionMetric.includes(':') ? ds.extensionMetric.split(':').pop()! : ds.extensionMetric)
      : ''
    if (extName && metricPart) return `${extName} · ${metricPart}`
    if (extName) return extName
    if (metricPart) return metricPart
    return t('chart.series', { count: idx + 1 })
  }

  const metricName = ds.metricId || ds.property
  const deviceId = getSourceId(ds)
  if (deviceId) {
    return `${_getDeviceName(deviceId)} · ${_getPropName(ds.metricId || ds.property)}`
  }
  if (metricName) {
    return _getPropName(metricName.includes(':') ? metricName.split(':').pop()! : metricName)
  }
  return t('chart.series', { count: idx + 1 })
}
