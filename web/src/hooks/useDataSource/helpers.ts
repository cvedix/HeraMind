/**
 * Pure utility functions for useDataSource.
 * Extracted from dedup.ts + extractors.ts — no external imports.
 */

import type { Device } from '@/types'
import { findDevice, buildDeviceMap } from '@/lib/deviceUtils'

// Re-export for backward compatibility
export { findDevice, buildDeviceMap }

// ============================================================================
// Value extraction (from extractors.ts)
// ============================================================================

/** Extract value from nested object: direct key → dot path → case-insensitive. */
export function extractValueFromData(rawData: string | unknown, property: string): unknown {
  if (rawData === null || rawData === undefined) return undefined
  if (typeof rawData !== 'object') return rawData

  const dataObj = rawData as Record<string, unknown>

  // Special handling for "_raw" style events where value is a JSON string
  if ('value' in dataObj && typeof dataObj.value === 'string' && 'metric' in dataObj) {
    const metric = dataObj.metric as string
    if (metric === '_raw' || (dataObj.value as string).trim().startsWith('{')) {
      try {
        const parsed = JSON.parse(dataObj.value as string)
        const extracted = extractValueFromParsed(parsed, property)
        if (extracted !== undefined) return extracted
      } catch { /* not valid JSON */ }
    }
  }

  // Direct key match
  if (property in dataObj) return dataObj[property]

  // Dot notation traversal
  if (property.includes('.')) {
    const parts = property.split('.')
    let current: unknown = dataObj
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]
      if (typeof current === 'object' && current !== null && part in (current as Record<string, unknown>)) {
        current = (current as Record<string, unknown>)[part]
        if (i === parts.length - 1 || typeof current !== 'object') return current
      } else {
        // Try case-insensitive at each level
        if (typeof current === 'object' && current !== null) {
          const found = findPropertyCI(current as Record<string, unknown>, part)
          if (found !== undefined) {
            current = found
            if (i === parts.length - 1 || typeof current !== 'object') return current
          }
        }
        return undefined
      }
    }
    return current
  }

  // Case-insensitive match
  const found = findPropertyCI(dataObj, property)
  if (found !== undefined) return found

  // Try nested in common properties
  // NOTE: If property contains '.', we already handled dot-notation above and returned.
  // The nested-property loop only needs to check for plain (non-dotted) properties.
  for (const nestedProp of ['current_values', 'currentValues', 'metrics', 'data', 'values', 'device_info', 'deviceInfo']) {
    if (nestedProp in dataObj && typeof dataObj[nestedProp] === 'object') {
      const nested = dataObj[nestedProp] as Record<string, unknown>
      const nestedValue = findPropertyCI(nested, property)
      if (nestedValue !== undefined) return nestedValue
    }
  }

  return undefined
}

/** Extract value from a parsed JSON object (no alias matching, just key + dot + case-insensitive). */
function extractValueFromParsed(parsed: unknown, property: string): unknown {
  if (parsed === null || parsed === undefined) return undefined
  if (typeof parsed !== 'object') return parsed

  const dataObj = parsed as Record<string, unknown>

  if (property in dataObj) return dataObj[property]

  if (property.includes('.')) {
    const parts = property.split('.')
    let current: unknown = dataObj
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]
      if (typeof current === 'object' && current !== null && part in (current as Record<string, unknown>)) {
        current = (current as Record<string, unknown>)[part]
        if (i === parts.length - 1 || typeof current !== 'object') return current
      } else {
        return undefined
      }
    }
    return current
  }

  const lowerProp = property.toLowerCase()
  for (const key of Object.keys(dataObj)) {
    if (key.toLowerCase() === lowerProp) return dataObj[key]
  }

  return undefined
}

/** Case-insensitive property lookup. */
function findPropertyCI(obj: Record<string, unknown>, property: string): unknown {
  if (property in obj) return obj[property]
  const lowerProp = property.toLowerCase()
  for (const key of Object.keys(obj)) {
    if (key.toLowerCase() === lowerProp) return obj[key]
  }
  return undefined
}

/** Null-safe value extraction, unwraps {value:...} wrapper objects. */
export function safeExtractValue(data: unknown, fallback: number | string | boolean = 0): unknown {
  if (data === null || data === undefined) return fallback
  if (typeof data === 'string' || typeof data === 'number' || typeof data === 'boolean') return data
  if (typeof data === 'object' && data !== null && 'value' in data) {
    return safeExtractValue((data as { value: unknown }).value, fallback)
  }
  return data
}

// ============================================================================
// Event metric matching (from extractors.ts:77-105)
// ============================================================================

const TRANSFORM_NAMESPACES = ['transform.', 'virtual.', 'computed.', 'derived.', 'aggregated.']

function isVirtualMetric(metricId: string): boolean {
  return TRANSFORM_NAMESPACES.some(ns => metricId.startsWith(ns))
}

/**
 * Check if event metric matches widget metricId.
 * Handles nested paths like values.image vs image, with virtual metric protection.
 */
export function eventMetricMatches(eventMetric: string, targetMetric: string): boolean {
  if (!eventMetric || !targetMetric) return false
  if (eventMetric === targetMetric) return true

  // Virtual metric protection — prevent "transform.temperature" matching "temperature"
  const eventIsVirtual = isVirtualMetric(eventMetric)
  const widgetIsVirtual = isVirtualMetric(targetMetric)
  if (eventIsVirtual !== widgetIsVirtual) return false

  // Nested path: "values.image" matches "image"
  if (eventMetric.endsWith('.' + targetMetric)) return true
  if (eventMetric.endsWith('/' + targetMetric)) return true
  if (targetMetric.endsWith('.' + eventMetric)) return true
  if (targetMetric.endsWith('/' + eventMetric)) return true

  return false
}

// ============================================================================
// Dedup helpers (from dedup.ts)
// ============================================================================

/** Get value from a data point. */
export function getPointValue(p: unknown): unknown {
  if (p == null) return undefined
  const o = p as Record<string, unknown>
  return o.value ?? o.v ?? o.val
}

/** Check if a point contains image data. */
export function isImageData(p: unknown): boolean {
  if (p == null || typeof p !== 'object') return false
  const o = p as Record<string, unknown>
  if ('src' in o || 'url' in o) return true
  if ('value' in o || 'v' in o) {
    const val = (o.value ?? o.v) as string | undefined
    if (typeof val === 'string') {
      return val.startsWith('data:image/') || val.startsWith('data:base64,') ||
             val.startsWith('http') || val.length > 2000
    }
  }
  if ('data' in o && typeof o.data === 'string' && o.data.length > 100) return true
  return false
}

/** Check if metric is image-related. */
function isImageMetric(metricId: string | undefined): boolean {
  if (!metricId) return false
  const lower = metricId.toLowerCase()
  return lower.includes('image') || lower.includes('img') || metricId.includes('values.image')
}

/**
 * Convert raw base64 to data URL using fast prefix-based format detection.
 * Detects PNG/JPEG/GIF/WebP/BMP by their base64-encoded magic bytes.
 * Default: image/jpeg (camera frames).
 */
export function base64ToDataUrl(raw: string): string {
  let mime = 'image/jpeg'
  if (raw.startsWith('iVBOR')) mime = 'image/png'
  else if (raw.startsWith('R0lGOD')) mime = 'image/gif'
  else if (raw.startsWith('UklGR')) mime = 'image/webp'
  else if (raw.startsWith('Qk')) mime = 'image/bmp'
  return `data:${mime};base64,${raw}`
}

/** Pre-normalize raw base64 to data URL for image sources. No-op for non-base64. */
export function normalizeImageValue(value: unknown): unknown {
  if (typeof value === 'string' && value.length > 100 && !value.startsWith('data:') && !value.startsWith('http')) {
    return base64ToDataUrl(value)
  }
  return value
}

/** Detect image data source from params/transform/field. */
export function isImageDataSource(
  ds: { params?: { includeRawPoints?: boolean }; transform?: string; field?: string; metricId?: string }
): boolean {
  const field = ds.field ?? ds.metricId
  return (ds.params?.includeRawPoints === true || ds.transform === 'raw') || isImageMetric(field)
}

/** Get data point limit — image sources get 200, others 50. */
export function getDataSourceLimit(
  ds: { params?: { includeRawPoints?: boolean }; transform?: string; field?: string; metricId?: string; limit?: number }
): number {
  const isImage = isImageDataSource(ds)
  return ds.limit ?? (isImage ? 200 : 50)
}

/**
 * Check if a data point already exists (timestamp within 1s + value match).
 * For long content (base64 images), compares first and last 500 chars.
 */
export function isDuplicatePoint(
  existingPoints: unknown[],
  newTimestamp: number,
  newValue: unknown,
  getTs: (p: unknown) => number,
): boolean {
  if (existingPoints.length === 0) return false

  // Fast path: for string values (common case — base64 images, URLs),
  // skip JSON.stringify entirely. Only stringify objects.
  const newIsStr = typeof newValue === 'string'
  const newStr = newIsStr ? newValue as string : JSON.stringify(newValue)
  if (!newStr) return false

  // Extract content after data: prefix for comparison
  let newContent: string
  if (newIsStr && newStr.startsWith('data:')) {
    const ci = newStr.indexOf(',')
    newContent = ci !== -1 ? newStr.slice(ci + 1) : newStr
  } else {
    newContent = newStr
  }

  // For large strings (images), use prefix+suffix fingerprint to avoid
  // full-string comparison. Pre-compute once instead of per-iteration.
  const isLarge = newContent.length > 2000
  const newPre = isLarge ? newContent.slice(0, 64) : ''
  const newSuf = isLarge ? newContent.slice(-64) : ''
  const newLen = newContent.length

  // Only check last few points — duplicates are almost always the newest.
  // Scanning the full array is O(n) per point but rarely useful beyond the tail.
  const startIdx = Math.max(0, existingPoints.length - 5)

  for (let i = existingPoints.length - 1; i >= startIdx; i--) {
    const p = existingPoints[i]
    const existingTs = getTs(p)
    if (Math.abs(existingTs - newTimestamp) > 1) continue

    const existingVal = getPointValue(p)
    if (existingVal === undefined || existingVal === null) continue

    const exIsStr = typeof existingVal === 'string'
    let existingContent: string
    if (exIsStr) {
      const es = existingVal as string
      if (es.startsWith('data:')) {
        const ci = es.indexOf(',')
        existingContent = ci !== -1 ? es.slice(ci + 1) : es
      } else {
        existingContent = es
      }
    } else {
      existingContent = JSON.stringify(existingVal)
      if (!existingContent) continue
    }

    if (isLarge && existingContent.length > 2000) {
      if (newLen === existingContent.length &&
          newPre === existingContent.slice(0, 64) &&
          newSuf === existingContent.slice(-64)) return true
    } else if (newContent === existingContent) return true
  }
  return false
}

/**
 * Deduplicate telemetry points. Skips dedup for image data.
 * Removes exact timestamp duplicates, caps at maxLimit.
 * O(n) using a Set for seen timestamps.
 */
export function dedupeTelemetryPoints(
  points: unknown[],
  getTs: (p: unknown) => number,
  maxLimit: number,
): unknown[] {
  if (points.some(isImageData)) return points.slice(0, maxLimit)

  const deduped: unknown[] = []
  const seen = new Set<number>()
  for (const p of points) {
    const ts = getTs(p)
    // Quantize to 1-second buckets to match the old Math.abs(ts - existing) < 1 behavior
    const bucket = Math.round(ts)
    if (seen.has(bucket)) continue
    seen.add(bucket)
    deduped.push(p)
    if (deduped.length >= maxLimit) break
  }
  return deduped
}

/** Resolve a device-info property value from a device object. */
export function resolveDeviceInfoValue(device: Device | undefined, infoProperty: string, fallback: unknown): unknown {
  if (!device) return fallback ?? '-'
  switch (infoProperty) {
    case 'name': return device.name || '-'
    case 'status': return device.status || 'unknown'
    case 'online': return device.online ?? false
    case 'last_seen': return device.last_seen || '-'
    case 'device_type': return device.device_type || '-'
    case 'plugin_name': return device.plugin_name || device.adapter_id || '-'
    case 'adapter_id': return device.adapter_id || '-'
    default: return fallback ?? '-'
  }
}

/**
 * Insert a new point into an ascending-sorted array (oldest-first).
 * Fast paths for sequential (newest-last) and prepend cases;
 * falls back to binary search for out-of-order insertion.
 * Returns the same array reference if no structural change occurred.
 */
export function insertAndMaintain(
  current: unknown[],
  newPoint: unknown,
  getTs: (p: unknown) => number,
  maxLimit: number,
  isImage: boolean,
): unknown[] {
  if (current.length === 0) return [newPoint]

  const newTs = getTs(newPoint)
  const newVal = getPointValue(newPoint)

  // For image data, check content duplication
  if (isImage && isDuplicatePoint(current, newTs, newVal, getTs)) return current

  // Fast path: most WS events arrive in chronological order (newest last)
  const lastTs = getTs(current[current.length - 1])
  if (newTs >= lastTs) {
    // Check if same timestamp bucket already exists (quantized to 1s)
    const bucket = Math.round(newTs)
    if (!isImage && Math.round(lastTs) === bucket) {
      const updated = current.slice()
      updated[current.length - 1] = newPoint
      return updated
    }
    // Append at end
    const result = [...current, newPoint]
    return result.length > maxLimit ? result.slice(result.length - maxLimit) : result
  }

  // Fast path: older than first point → prepend and trim from front
  const firstTs = getTs(current[0])
  if (newTs < firstTs) {
    if (current.length >= maxLimit) return current // Already full, oldest point is less valuable
    return [newPoint, ...current]
  }

  // Slow path: binary search for out-of-order insertion
  let lo = 0, hi = current.length
  while (lo < hi) {
    const mid = (lo + hi) >>> 1
    if (getTs(current[mid]) < newTs) lo = mid + 1
    else hi = mid
  }

  // Check if same timestamp bucket already exists (quantized to 1s)
  const bucket = Math.round(newTs)
  if (!isImage) {
    for (let i = Math.max(0, lo - 1); i <= Math.min(lo, current.length - 1); i++) {
      if (Math.round(getTs(current[i])) === bucket) {
        const updated = current.slice()
        updated[i] = newPoint
        return updated
      }
    }
  }

  // Insert at position `lo`
  const result = current.slice(0, lo)
  result.push(newPoint)
  for (let i = lo; i < current.length; i++) result.push(current[i])

  if (result.length > maxLimit) return result.slice(result.length - maxLimit)

  return result
}

/** Sort points by timestamp descending, dedup, and cap. */
export function sortAndDedup(
  points: unknown[],
  getTs: (p: unknown) => number,
  maxLimit: number,
  isImage: boolean,
): unknown[] {
  if (!Array.isArray(points) || points.length === 0) return points

  // Sort indices in-place to avoid creating wrapper objects
  const indices = points.map((_, i) => i)
  indices.sort((a, b) => {
    const d = getTs(points[b]) - getTs(points[a])
    return d !== 0 ? d : a - b
  })
  const sorted = indices.map(i => points[i])

  if (isImage) {
    const out: unknown[] = []
    for (const pt of sorted) {
      if (!isDuplicatePoint(out, getTs(pt), getPointValue(pt), getTs)) out.push(pt)
      if (out.length >= maxLimit) break
    }
    return out
  }
  return dedupeTelemetryPoints(sorted, getTs, maxLimit)
}

// ============================================================================
// Source transforms for single-value components
// ============================================================================

import type { DataSource } from '@/types/dashboard'

/**
 * Convert telemetry sources to device type for instant store-based reads.
 *
 * Components that only need the latest single value (LED, ProgressBar, ValueCard,
 * MarkdownDisplay) should use this as their `sourceTransform` option in useDataSource.
 *
 * Instead of fetching historical telemetry via API (slow HTTP round-trip), this
 * converts the source to `type: 'device'` which reads directly from
 * store.current_values (synchronous, instant). If the store has no data yet,
 * useStoreSource automatically triggers fetchDeviceTelemetry to populate it.
 */
export function latestValueSourceTransform(ds: DataSource): DataSource | undefined {
  if (ds.mode === 'timeseries' && ds.source === 'device') {
    const property = ds.field ?? ds.metricId ?? 'value'
    return {
      ...ds,
      type: 'device',
      sourceId: ds.id ?? ds.sourceId,
      property,
      source: 'device',
      mode: 'latest',
      id: ds.id,
      field: property,
    }
  }
  return ds
}
