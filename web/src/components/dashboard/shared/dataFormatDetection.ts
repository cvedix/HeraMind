/**
 * Data Format Detection Utilities
 *
 * Typed guard functions replacing fragile runtime type sniffing
 * across chart components (BarChart, LineChart, PieChart).
 */

// { name: string, value: number } format — Bar/Pie
export function isNameValueData(data: unknown): data is Array<{ name: string; value: unknown }> {
  return (
    Array.isArray(data) &&
    data.length > 0 &&
    typeof data[0] === 'object' &&
    data[0] !== null &&
    'value' in data[0] &&
    'name' in data[0]
  )
}

// { name: string, data: number[] } format — Line/Area series
export function isSeriesDataArray(data: unknown): data is Array<{ name: string; data: number[] }> {
  return (
    Array.isArray(data) &&
    data.length > 0 &&
    typeof data[0] === 'object' &&
    data[0] !== null &&
    'data' in data[0] &&
    Array.isArray((data[0] as { data: unknown }).data)
  )
}

// { timestamp: number, value: unknown } format — telemetry points
// Has 'value' and a time key ('timestamp', 't', or 'time') but NOT a dedicated 'data' array
export function isTelemetryPointArray(data: unknown): data is Array<{ timestamp?: number; value: unknown }> {
  return (
    Array.isArray(data) &&
    data.length > 0 &&
    typeof data[0] === 'object' &&
    data[0] !== null &&
    'value' in data[0] &&
    !('data' in data[0]) &&
    // Must have a time-related key to distinguish from name-value data
    ('timestamp' in data[0] || 't' in data[0] || 'time' in data[0] || !('name' in data[0]))
  )
}

// number[] format
export function isNumberArray(data: unknown): data is number[] {
  return Array.isArray(data) && data.length > 0 && typeof data[0] === 'number'
}

// unknown[][] format — multi-source data
export function isMultiSourceData(data: unknown, sourceCount: number): data is unknown[][] {
  return (
    Array.isArray(data) &&
    data.length >= sourceCount &&
    sourceCount > 1 &&
    Array.isArray(data[0])
  )
}

// Extract numeric value from various point formats
export function extractNumericValue(item: unknown): number {
  if (typeof item === 'number') return item
  if (typeof item === 'object' && item !== null && 'value' in item) {
    const val = (item as { value: unknown }).value
    return typeof val === 'number' ? val : 0
  }
  return 0
}
