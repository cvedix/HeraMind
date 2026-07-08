/**
 * Device lookup utilities.
 *
 * Extracted from useDataSource/helpers.ts for use across the entire codebase.
 * Eliminates 11+ occurrences of inline `devices.find(d => d.id === id || d.device_id === id)`.
 */

import type { Device } from '@/types'

// Module-level cache so ALL callers share the same Map — avoids rebuilding
// per-component when the devices array reference hasn't changed.
let cachedFindDeviceMap: { devicesRef: Device[]; map: Map<string, Device> } | null = null

/** Find a device by id or device_id. O(1) via cached Map. */
export function findDevice(devices: Device[], id: string | undefined): Device | undefined {
  if (!id) return undefined
  if (!cachedFindDeviceMap || cachedFindDeviceMap.devicesRef !== devices) {
    cachedFindDeviceMap = { devicesRef: devices, map: buildDeviceMap(devices) }
  }
  return cachedFindDeviceMap.map.get(id)
}

/** Build a Map for O(1) device lookups by both id and device_id. */
export function buildDeviceMap(devices: Device[]): Map<string, Device> {
  const map = new Map<string, Device>()
  for (const d of devices) {
    if (d.id) map.set(d.id, d)
    if (d.device_id && d.device_id !== d.id) map.set(d.device_id, d)
  }
  return map
}
