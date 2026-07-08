import type { LucideIcon } from 'lucide-react'
import { Unplug } from 'lucide-react'
import type { Rule } from '@/types/rule'

export type DurationUnit = 'minutes' | 'hours'
export type Severity = 'info' | 'warning' | 'critical' | 'emergency'

export interface DeviceOfflineTemplateOptions {
  deviceId: string
  durationValue: number
  durationUnit: DurationUnit
  severity: Severity
}

export interface RuleTemplate<TOptions> {
  id: string
  labelKey: string
  descriptionKey: string
  icon: LucideIcon
  /** Tailwind accent token base, e.g. 'error' → bg-error-light text-error */
  accent?: 'primary' | 'success' | 'warning' | 'error' | 'info'
  build: (opts: TOptions) => Partial<Rule>
}

export const DEVICE_OFFLINE_DEFAULTS: DeviceOfflineTemplateOptions = {
  deviceId: '',
  durationValue: 12,
  durationUnit: 'hours',
  severity: 'critical',
}

function toSeconds(value: number, unit: DurationUnit): number {
  return unit === 'hours' ? value * 3600 : value * 60
}

export const RULE_TEMPLATES: RuleTemplate<DeviceOfflineTemplateOptions>[] = [
  {
    id: 'device_offline',
    labelKey: 'rules.templates.deviceOffline.label',
    descriptionKey: 'rules.templates.deviceOffline.description',
    icon: Unplug,
    accent: 'warning',
    build: ({ deviceId, durationValue, durationUnit, severity }) => {
      const seconds = toSeconds(durationValue, durationUnit)
      return {
        name: '',  // user fills
        condition: {
          condition_type: 'comparison',
          source: `device:${deviceId}:__last_seen_age_secs`,
          operator: '>',
          threshold: seconds,
        },
        actions: [{
          type: 'notify',
          message: `设备 ${deviceId} 已静默超过 ${durationValue} ${durationUnit === 'hours' ? '小时' : '分钟'}`,
          severity,
        }],
        trigger: {
          trigger_type: 'data_change',
          sources: [`device:${deviceId}:__last_seen_age_secs`],
        },
        // Cooldown serialized as ms; ≥60s per validator. Production guidance is 5min+.
        cooldown: Math.max(seconds * 1000, 60 * 1000),
      }
    },
  },
]
