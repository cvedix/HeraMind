/**
 * Device Adapter Types
 *
 * Static list of built-in device adapter types.
 * Previously fetched from /api/device-adapters/types, now hardcoded.
 */

import type { AdapterType } from '@/types'

/**
 * Built-in device adapter types
 */
export const ADAPTER_TYPES: AdapterType[] = [
  {
    id: 'mqtt',
    name: 'MQTT',
    description: 'MQTT broker connections (built-in + external)',
    icon: 'Server',
    icon_bg: 'bg-info-light text-info',
    mode: 'push',
    can_add_multiple: true,
    builtin: true,
  },
  {
    id: 'webhook',
    name: 'Webhook',
    description: 'Devices push data via HTTP POST to your server',
    icon: 'Webhook',
    icon_bg: 'bg-success-light text-success dark:bg-success-light dark:text-success',
    mode: 'push',
    can_add_multiple: false,
    builtin: true,
  },
]

/**
 * Get adapter type by ID
 */
export const getAdapterType = (id: string): AdapterType | undefined => {
  return ADAPTER_TYPES.find((t) => t.id === id)
}

/**
 * Get adapter icon component name
 */
export const ADAPTER_ICONS: Record<string, string> = {
  mqtt: 'Server',
  webhook: 'Webhook',
}
