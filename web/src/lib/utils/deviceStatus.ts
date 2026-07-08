/**
 * Device connection state model (4-state UI).
 *
 * The backend distinguishes two independent signals:
 * 1. `online` — data-driven online flag (active publish within offline timeout).
 * 2. `transport_connected` — MQTT session connected flag.
 *
 * Combining them yields a 4-state model that resolves the
 * "Never Connected vs Offline" ambiguity customers complained about:
 *
 *   ┌─────────────────────┬──────────────────────────────┐
 *   │ online=true         │ "online"  (Online, active)   │
 *   ├─────────────────────┼──────────────────────────────┤
 *   │ online=false        │ "connectedIdle"              │
 *   │ transport=true      │ (MQTT alive, no fresh data)  │
 *   ├─────────────────────┼──────────────────────────────┤
 *   │ online=false        │ "offline"                    │
 *   │ transport=false     │ (Was online, timed out)      │
 *   │ last_seen>0         │                              │
 *   ├─────────────────────┼──────────────────────────────┤
 *   │ online=false        │ "disconnected"               │
 *   │ transport=false     │ (Never reported data)        │
 *   │ last_seen=0         │                              │
 *   └─────────────────────┴──────────────────────────────┘
 *
 * `transport_connected` is `undefined` against older backends — we fall
 * back to the legacy 3-state model in that case (online / offline /
 * disconnected) so existing deployments keep rendering correctly.
 */

import type { Device } from '@/types'

export type DeviceConnectionState =
  | 'online'
  | 'connectedIdle'
  | 'offline'
  | 'disconnected'

export interface DeviceStateInfo {
  state: DeviceConnectionState
  /** True when the device is actively reporting fresh data. */
  online: boolean
  /** True when an MQTT session is alive (independent of data activity). */
  transportConnected: boolean
  /** Mirrors backend `config.last_seen == 0` — device has never reported. */
  neverSeen: boolean
}

/**
 * Compute the 4-state device status from the device record.
 *
 * Falls back to legacy 3-state resolution when the backend doesn't
 * populate `transport_connected` (undefined).
 */
export function getDeviceState(device: Pick<Device, 'online' | 'transport_connected' | 'last_seen'>): DeviceStateInfo {
  const online = !!device.online
  const transportConnected = device.transport_connected
  // `last_seen` arrives as an ISO string from the API; empty/missing means never.
  const lastSeenStr = device.last_seen ?? ''
  const lastSeenEpoch = lastSeenStr ? Date.parse(lastSeenStr) : 0
  const neverSeen = !lastSeenEpoch || Number.isNaN(lastSeenEpoch) || lastSeenEpoch <= 0

  // New backend → 4-state resolution (null/undefined → legacy fallback)
  if (transportConnected != null) {
    if (online) {
      return { state: 'online', online, transportConnected, neverSeen }
    }
    if (transportConnected) {
      return { state: 'connectedIdle', online, transportConnected, neverSeen }
    }
    return {
      state: neverSeen ? 'disconnected' : 'offline',
      online,
      transportConnected,
      neverSeen,
    }
  }

  // Legacy fallback (older backend without transport_connected)
  if (online) {
    return { state: 'online', online, transportConnected: false, neverSeen }
  }
  return {
    state: neverSeen ? 'disconnected' : 'offline',
    online,
    transportConnected: false,
    neverSeen,
  }
}

/**
 * Stable status string suitable for `StatusBadge` / i18n lookup.
 *
 * `connectedIdle` is a synthetic key — the i18n catalogue must define it
 * under `statusLabels.connectedIdle`. Falls back to `info` color variant.
 */
export function getDeviceStateStatus(state: DeviceConnectionState): string {
  return state
}

/**
 * Color variant for the `StatusBadge` component.
 *
 * - online        → success (green, animated)
 * - connectedIdle → info    (blue, calm — MQTT session alive, awaiting data; UI label: "连接中·待机")
 * - offline       → warning (orange — recently went silent)
 * - disconnected  → muted   (gray — never seen)
 */
export function getDeviceStateColor(state: DeviceConnectionState): 'success' | 'info' | 'warning' | 'muted' {
  switch (state) {
    case 'online':
      return 'success'
    case 'connectedIdle':
      return 'info'
    case 'offline':
      return 'warning'
    case 'disconnected':
      return 'muted'
  }
}
