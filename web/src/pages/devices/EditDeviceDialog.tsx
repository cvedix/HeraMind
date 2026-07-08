import { useState, useEffect, useMemo, useCallback } from "react"
import { useTranslation } from "react-i18next"
import { useServerUrl } from "@/lib/server-url"
import { Input } from "@/components/ui/input"
import { toast } from "@/components/ui/use-toast"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Edit2 } from "lucide-react"
import type { Device, DeviceType, ConnectionConfig } from "@/types"
import { FormSection, FormSectionGroup } from "@/components/ui/form-section"
import { FormField } from "@/components/ui/field"
import { UnifiedFormDialog } from "@/components/dialog/UnifiedFormDialog"

interface EditDeviceDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  device: Device | null
  deviceTypes: DeviceType[]
  onEdit: (id: string, data: Partial<{ name: string; adapter_type: string; connection_config: ConnectionConfig; offline_timeout_secs: number | null }>) => Promise<boolean>
  editing: boolean
}

export function EditDeviceDialog({
  open,
  onOpenChange,
  device,
  deviceTypes,
  onEdit,
  editing,
}: EditDeviceDialogProps) {
  const { t } = useTranslation(['common', 'devices'])
  const serverUrl = useServerUrl()

  const [deviceName, setDeviceName] = useState("")
  const [adapterType, setAdapterType] = useState<string>("mqtt")
  const [connectionConfig, setConnectionConfig] = useState<ConnectionConfig>({})
  // Per-device offline-timeout override (seconds). Empty string → cleared (use template/global).
  const [offlineTimeout, setOfflineTimeout] = useState<string>("")

  // Memoize device type info to prevent unnecessary re-renders
  const deviceTypeInfo = useMemo(() => {
    if (!device?.device_type) return null
    return deviceTypes.find(t => t.device_type === device.device_type) || null
  }, [device?.device_type, deviceTypes])

  const hasCommands = useMemo(() => {
    return (deviceTypeInfo?.commands?.length || 0) > 0
  }, [deviceTypeInfo?.commands?.length])

  // Initialize form with device data when dialog opens
  useEffect(() => {
    if (open && device) {
      setDeviceName(device.name || "")
      setAdapterType(device.adapter_type || "mqtt")
      setOfflineTimeout(
        device.offline_timeout_secs != null ? String(device.offline_timeout_secs) : "",
      )

      const config = device.connection_config || {}

      if (hasCommands && !config.command_topic && device.device_type && device.id) {
        config.command_topic = `device/${device.device_type}/${device.id}/downlink`
      }

      setConnectionConfig(config)
    }
  }, [open, device, hasCommands])

  const handleEdit = async () => {
    if (!device) return

    // Parse offline-timeout override: empty → null (clear), otherwise seconds.
    const trimmed = offlineTimeout.trim()
    let offlineTimeoutSecs: number | null = null
    if (trimmed) {
      const parsed = Number(trimmed)
      const MIN = 30
      const MAX = 86400
      if (!Number.isFinite(parsed) || parsed < MIN || parsed > MAX) {
        toast({
          title: t('common:failed'),
          description: t('devices:edit.invalidOfflineTimeout', {
            defaultValue: 'Offline timeout must be between {{min}} and {{max}} seconds',
            min: MIN,
            max: MAX,
          }),
          variant: "destructive",
        })
        return
      }
      offlineTimeoutSecs = Math.floor(parsed)
    }

    const success = await onEdit(device.id, {
      name: deviceName,
      adapter_type: adapterType,
      connection_config: connectionConfig,
      offline_timeout_secs: offlineTimeoutSecs,
    })

    if (success) {
      onOpenChange(false)
      toast({
        title: t('common:success'),
        description: t('devices:edit.success'),
      })
    } else {
      toast({
        title: t('common:failed'),
        description: t('devices:edit.error'),
        variant: "destructive",
      })
    }
  }

  const handleClose = useCallback(() => {
    if (!editing) {
      setDeviceName("")
      setAdapterType("mqtt")
      setConnectionConfig({})
      onOpenChange(false)
    }
  }, [editing, onOpenChange])

  return (
    <UnifiedFormDialog
      open={open}
      onOpenChange={(newOpen) => { if (!newOpen && !editing) handleClose() }}
      title={t('devices:edit.title')}
      icon={<Edit2 className="h-5 w-5 text-primary" />}
      width="sm"
      onSubmit={handleEdit}
      isSubmitting={editing}
      submitLabel={t('common:save')}
    >
      <FormSectionGroup>
        {/* Device ID (read-only) */}
        <FormField label={t('devices:deviceId')}>
          <Input
            value={device?.id || ''}
            readOnly
            disabled
            className="font-mono bg-muted"
          />
        </FormField>

        {/* Device Type (read-only) */}
        <FormField label={t('devices:deviceType')}>
          <Input
            value={deviceTypeInfo?.name || device?.device_type || ''}
            readOnly
            disabled
            className="bg-muted"
          />
        </FormField>

        {/* Device Name */}
        <FormField label={t('devices:deviceName')}>
          <Input
            value={deviceName}
            onChange={(e) => setDeviceName(e.target.value)}
            placeholder={t('common:optional')}
          />
        </FormField>

        {/* Adapter Type */}
        <FormField label={t('devices:add.adapterType')}>
          <Select
            value={adapterType}
            onValueChange={(v) => setAdapterType(v)}
            disabled={adapterType === 'extension'}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="mqtt">MQTT</SelectItem>
              <SelectItem value="webhook">Webhook</SelectItem>
              <SelectItem value="extension" disabled>Extension</SelectItem>
            </SelectContent>
          </Select>
        </FormField>

        {/* Adapter Config */}
        {adapterType === 'mqtt' && (
          <FormSection
            title={t('devices:add.mqttConfig', { defaultValue: 'MQTT Configuration' })}
            collapsible
            defaultExpanded
          >
            <div className="space-y-3">
              <FormField label={t('devices:add.telemetryTopic')}>
                <Input
                  value={connectionConfig.telemetry_topic || ''}
                  onChange={(e) => setConnectionConfig({ ...connectionConfig, telemetry_topic: e.target.value })}
                  placeholder="device/{type}/{id}/uplink"
                  className="font-mono text-sm"
                />
              </FormField>
              <FormField
                label={t('devices:add.commandTopic')}
                helpText={t('devices:add.commandTopicHint', {
                  defaultValue: 'Topic the device subscribes to for commands. Leave blank if the device has no downlink.',
                })}
              >
                <Input
                  value={connectionConfig.command_topic || ''}
                  onChange={(e) => setConnectionConfig({ ...connectionConfig, command_topic: e.target.value })}
                  placeholder="device/{type}/{id}/downlink"
                  className="font-mono text-sm"
                />
              </FormField>
            </div>
          </FormSection>
        )}

        {adapterType === 'webhook' && (
          <FormSection
            title={t('devices:add.webhookConfig', { defaultValue: 'Webhook Configuration' })}
          >
            <div className="rounded-lg border bg-muted p-4">
              <p className="text-sm text-muted-foreground mb-2">
                {t('devices:add.webhookUrlDescription')}
              </p>
              <code className="text-xs break-all block">
                {serverUrl}/api/devices/{device?.id}/webhook
              </code>
            </div>
          </FormSection>
        )}

        {/* Per-device offline timeout override (seconds) — applies to all adapter types */}
        <FormField
          label={t('devices:edit.offlineTimeoutLabel', {
            defaultValue: 'Offline Timeout (seconds)',
          })}
          helpText={t('devices:edit.offlineTimeoutHint', {
            defaultValue: 'Range 30–86400. Leave blank to use the default ({{default}}s).',
            default: device?.effective_offline_timeout_secs ?? 300,
          })}
        >
          <Input
            value={offlineTimeout}
            onChange={(e) => setOfflineTimeout(e.target.value)}
            placeholder={t('devices:edit.offlineTimeoutPlaceholder', {
              defaultValue: 'Default: {{default}}s',
              default: device?.effective_offline_timeout_secs ?? 300,
            })}
            inputMode="numeric"
          />
        </FormField>
      </FormSectionGroup>
    </UnifiedFormDialog>
  )
}
