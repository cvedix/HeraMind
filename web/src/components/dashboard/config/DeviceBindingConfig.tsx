/**
 * DeviceBindingConfig Component
 *
 * Device selector for community components that bind to specific device instances.
 * Filters devices by device_type_filter from manifest and groups them by type.
 */

import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { useStore } from '@/store'
import { ConfigSection } from './ConfigSection'
import { findDevice } from '@/lib/deviceUtils'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Wifi, WifiOff } from 'lucide-react'
import type { Device } from '@/types'

export interface DeviceBindingConfigProps {
  /** Currently selected device ID */
  deviceId?: string
  /** Allowed device types to filter by (from manifest's device_type_filter) */
  deviceTypeFilter?: string[]
  /** Callback when device selection changes */
  onChange: (deviceId: string | undefined) => void
}

export function DeviceBindingConfig({
  deviceId,
  deviceTypeFilter,
  onChange,
}: DeviceBindingConfigProps) {
  const { t } = useTranslation()
  const devices = useStore((s) => s.devices)

  // Filter devices by allowed types
  const filteredDevices = useMemo(() => {
    if (!deviceTypeFilter || deviceTypeFilter.length === 0) return devices
    return devices.filter((d) => deviceTypeFilter.includes(d.device_type))
  }, [devices, deviceTypeFilter])

  // Group by device type
  const grouped = useMemo(() => {
    const groups: Record<string, Device[]> = {}
    for (const d of filteredDevices) {
      const key = d.device_type || 'other'
      if (!groups[key]) groups[key] = []
      groups[key].push(d)
    }
    return groups
  }, [filteredDevices])

  const selectedDevice = useMemo(
    () => findDevice(devices, deviceId),
    [devices, deviceId],
  )

  if (filteredDevices.length === 0) {
    return (
      <ConfigSection title={t('dashboard.config.deviceBinding', 'Device Binding')}>
        <p className="text-sm text-muted-foreground py-2">
          {t('dashboard.config.noCompatibleDevices', 'No compatible devices found')}
        </p>
      </ConfigSection>
    )
  }

  return (
    <ConfigSection title={t('dashboard.config.deviceBinding', 'Device Binding')}>
      <Select
        value={deviceId || ''}
        onValueChange={(v) => onChange(v || undefined)}
      >
        <SelectTrigger className="w-full h-9">
          <SelectValue
            placeholder={t('dashboard.config.selectDevice', 'Select a device')}
          />
        </SelectTrigger>
        <SelectContent>
          {Object.entries(grouped).map(([type, devs]) => (
            <div key={type}>
              <div className="px-2 py-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                {type}
              </div>
              {devs.map((d) => (
                <SelectItem key={d.id} value={d.id}>
                  <span className="flex items-center gap-2">
                    {d.online ? (
                      <Wifi className="h-3 w-3 text-success" />
                    ) : (
                      <WifiOff className="h-3 w-3 text-muted-foreground" />
                    )}
                    {d.name}
                  </span>
                </SelectItem>
              ))}
            </div>
          ))}
        </SelectContent>
      </Select>
      {selectedDevice && (
        <div className="text-xs text-muted-foreground mt-1">
          {selectedDevice.device_type} · {selectedDevice.online ? t('online') : t('offline')}
        </div>
      )}
    </ConfigSection>
  )
}
