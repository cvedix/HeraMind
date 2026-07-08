/**
 * UIConfigSections Component
 *
 * Pre-built configuration sections for common UI patterns.
 * Provides consistent styling and validation for reusable config options.
 */

import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { SingleValueMappingConfig, TimeSeriesMappingConfig, CategoricalMappingConfig } from '@/lib/dataMapping'

// ============================================================================
// Type Exports
// ============================================================================

export type SizeOption = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl'

export interface BooleanOption {
  key: string
  label: string
  value?: boolean
  description?: string
}

export interface SelectOption {
  value: string
  label: string
  description?: string
}

export type Orientation = 'horizontal' | 'vertical'

export type DataMappingType = 'single' | 'time-series' | 'categorical'

// ============================================================================
// Data Mapping Configuration Section
// ============================================================================

export interface DataMappingConfigProps {
  dataMapping?: SingleValueMappingConfig | TimeSeriesMappingConfig | CategoricalMappingConfig
  onChange?: (config: any) => void
  mappingType: DataMappingType
  label?: string
  readonly?: boolean
}

/**
 * Simplified Data Mapping Configuration
 * Only shows format options - fields are auto-detected by DataMapper
 */
export function DataMappingConfig({
  dataMapping,
  onChange,
  mappingType,
  label,
  readonly = false,
}: DataMappingConfigProps) {
  const { t } = useTranslation('dashboardComponents')

  const updateField = (field: string, value: any) => {
    const updated = { ...dataMapping, [field]: value }
    onChange?.(updated)
  }

  // Only show format options for single value mapping
  if (mappingType !== 'single') {
    return null // Charts use auto-detection
  }

  const config = dataMapping as SingleValueMappingConfig

  return (
    <div className="space-y-3">
      <h4 className="text-sm font-medium">{label || t('formatConfig.title')}</h4>

      {/* Format Type */}
      <div className="space-y-2">
        <Label className="text-xs">{t('formatConfig.type')}</Label>
        <Select
          value={config?.format || 'auto'}
          onValueChange={(v) => updateField('format', v)}
          disabled={readonly}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="auto">{t('formatConfig.auto')}</SelectItem>
            <SelectItem value="number">{t('formatConfig.number')}</SelectItem>
            <SelectItem value="currency">{t('formatConfig.currency')}</SelectItem>
            <SelectItem value="percent">{t('formatConfig.percent')}</SelectItem>
            <SelectItem value="bytes">{t('formatConfig.bytes')}</SelectItem>
            <SelectItem value="duration">{t('formatConfig.duration')}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Unit */}
      <div className="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] gap-2">
        <div className="space-y-2">
          <Label className="text-xs">{t('formatConfig.unit')}</Label>
          <Input
            placeholder={t('formatConfig.unitPlaceholder')}
            value={config?.unit || ''}
            onChange={(e) => updateField('unit', e.target.value || undefined)}
            disabled={readonly}
          />
        </div>
        <div className="space-y-2">
          <Label className="text-xs">{t('formatConfig.unitPosition')}</Label>
          <Select
            value={config?.unitPosition || 'suffix'}
            onValueChange={(v) => updateField('unitPosition', v)}
            disabled={readonly}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="prefix">{t('formatConfig.prefix')}</SelectItem>
              <SelectItem value="suffix">{t('formatConfig.suffix')}</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  )
}
