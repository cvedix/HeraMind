/**
 * ParameterInput — single command-parameter renderer.
 *
 * This is the SINGLE source of truth for how a `ParameterDefinition`
 * becomes an interactive form control. Both the dashboard CommandButton
 * and the device-detail command dialog route through this component.
 *
 * Field coverage (vs. ParameterDefinition in `types/device.ts`):
 *   ✅ data_type: string | integer | float | boolean | binary | array
 *   ✅ data_type: { enum: [...] }  (object form)
 *   ✅ allowed_values             (preferred over enum-object when both exist)
 *   ✅ min / max / unit / step
 *   ✅ default_value              (used as initial value by parent)
 *   ✅ help_text                  (rendered below)
 *   ✅ required                   (red asterisk on the label)
 *   ✅ validation: pattern | range | length  (HTML5 + onSubmit checks)
 *
 * Non-goals:
 *   - visible_when / parameter_groups: handled by `ParameterForm`.
 *   - validation.custom: requires function injection, intentionally skipped.
 */

import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type {
  ParameterDefinition,
  ValidationRule,
} from '@/types/device'

export interface ParameterInputProps {
  param: ParameterDefinition
  value: unknown
  onChange: (value: unknown) => void
  /** Optional placeholder override; otherwise derived from param. */
  placeholder?: string
  /** Compact mode — pill-style booleans instead of Select (mobile-friendly). */
  variant?: 'default' | 'compact'
}

/**
 * Pull a `pattern` validation rule out of the schema if present.
 * Used to set the native `pattern` attribute for browser-side feedback.
 */
function findPattern(validations?: ValidationRule[]): string | undefined {
  if (!validations) return undefined
  const rule = validations.find((v) => v.type === 'pattern')
  return rule && rule.type === 'pattern' ? rule.regex : undefined
}

/**
 * Find a length rule, if any. Returns {min, max} where each may be undefined.
 */
function findLengthRule(
  validations?: ValidationRule[],
): { min?: number; max?: number } | undefined {
  if (!validations) return undefined
  const rule = validations.find((v) => v.type === 'length')
  if (rule && rule.type === 'length') {
    return { min: rule.min, max: rule.max }
  }
  return undefined
}

export function ParameterInput({
  param,
  value,
  onChange,
  placeholder,
  variant = 'default',
}: ParameterInputProps) {
  const { t } = useTranslation('devices')

  // ---------------------------------------------------------------- enum
  // Object-form enum on data_type — { enum: string[] }
  const enumOptions = useMemo(() => {
    if (
      param.data_type &&
      typeof param.data_type === 'object' &&
      'enum' in param.data_type
    ) {
      return param.data_type.enum
    }
    return null
  }, [param.data_type])

  // Explicit allowed_values array — preferred over data_type.enum because
  // it can carry non-string values (numbers, booleans).
  const allowedValues = param.allowed_values && param.allowed_values.length > 0
    ? param.allowed_values
    : null

  // Combined list of {label, value} pairs for Select/ButtonGroup rendering.
  const options = useMemo(() => {
    if (allowedValues) {
      return allowedValues.map((v) => ({ label: String(v), value: v }))
    }
    if (enumOptions) {
      return enumOptions.map((s) => ({ label: s, value: s }))
    }
    return null
  }, [allowedValues, enumOptions])

  // ------------------------------------------------------------ select (options)
  if (options && options.length > 0) {
    // Compact mode → pill-style button group. Good for small enums on mobile.
    if (variant === 'compact' && options.length <= 4) {
      return (
        <div className="flex flex-wrap gap-2">
          {options.map((opt) => (
            <Button
              key={opt.label}
              type="button"
              variant={value === opt.value ? 'default' : 'outline'}
              size="sm"
              onClick={() => onChange(opt.value)}
              className="rounded-full"
            >
              {opt.label}
            </Button>
          ))}
        </div>
      )
    }
    return (
      <Select
        value={value === undefined || value === null ? '' : String(value)}
        onValueChange={(v) => {
          // Attempt to preserve original type of the option.
          const matched = options.find((o) => String(o.value) === v)
          onChange(matched ? matched.value : v)
        }}
      >
        <SelectTrigger>
          <SelectValue placeholder={placeholder ?? t('command.dialog.selectValue', { defaultValue: 'Select…' })} />
        </SelectTrigger>
        <SelectContent>
          {options.map((opt) => (
            <SelectItem key={opt.label} value={String(opt.value)}>
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    )
  }

  // ------------------------------------------------------------- boolean
  if (param.data_type === 'boolean') {
    if (variant === 'compact') {
      return (
        <div className="flex gap-2">
          <Button
            type="button"
            variant={value === true ? 'default' : 'outline'}
            size="sm"
            onClick={() => onChange(true)}
            className="rounded-full"
          >
            {t('command.dialog.yes', { defaultValue: 'Yes' })}
          </Button>
          <Button
            type="button"
            variant={value === false ? 'default' : 'outline'}
            size="sm"
            onClick={() => onChange(false)}
            className="rounded-full"
          >
            {t('command.dialog.no', { defaultValue: 'No' })}
          </Button>
        </div>
      )
    }
    return (
      <Select
        value={value === undefined ? '' : String(value)}
        onValueChange={(v) => onChange(v === 'true')}
      >
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="true">
            {t('command.dialog.yes', { defaultValue: 'Yes' })}
          </SelectItem>
          <SelectItem value="false">
            {t('command.dialog.no', { defaultValue: 'No' })}
          </SelectItem>
        </SelectContent>
      </Select>
    )
  }

  // ------------------------------------------------------- integer / float
  if (param.data_type === 'integer' || param.data_type === 'float') {
    const isFloat = param.data_type === 'float'
    const lengthRule = findLengthRule(param.validation)
    return (
      <div className="flex items-center gap-2">
        <Input
          type="number"
          value={value === undefined || value === null ? '' : String(value)}
          onChange={(e) => {
            const raw = e.target.value
            if (raw === '') {
              onChange(undefined)
              return
            }
            const parsed = isFloat ? parseFloat(raw) : parseInt(raw, 10)
            onChange(isNaN(parsed) ? 0 : parsed)
          }}
          min={param.min}
          max={param.max}
          maxLength={lengthRule?.max}
          step={isFloat ? 'any' : '1'}
          placeholder={
            placeholder ??
            (param.min !== undefined || param.max !== undefined
              ? `${param.min ?? 0} ~ ${param.max ?? '∞'}${param.unit ? ' ' + param.unit : ''}`
              : undefined)
          }
        />
        {param.unit && (
          <span className="text-xs text-muted-foreground whitespace-nowrap">
            {param.unit}
          </span>
        )}
      </div>
    )
  }

  // ------------------------------------------------------------- binary
  if (param.data_type === 'binary') {
    return (
      <Textarea
        value={typeof value === 'string' ? value : ''}
        onChange={(e) => onChange(e.target.value)}
        placeholder={
          placeholder ??
          t('command.dialog.binaryPlaceholder', {
            defaultValue: 'Base64 encoded data…',
          })
        }
        rows={3}
      />
    )
  }

  // ------------------------------------------------------------- array
  if (param.data_type === 'array') {
    return (
      <Textarea
        value={
          typeof value === 'string'
            ? value
            : value === undefined
              ? ''
              : JSON.stringify(value, null, 2)
        }
        onChange={(e) => {
          const raw = e.target.value
          try {
            onChange(JSON.parse(raw))
          } catch {
            // Keep raw text so the user can keep typing.
            onChange(raw)
          }
        }}
        placeholder={placeholder ?? '[1, 2, 3]'}
        rows={3}
      />
    )
  }

  // -------------------------------------------------------- string (default)
  const pattern = findPattern(param.validation)
  const lengthRule = findLengthRule(param.validation)
  return (
    <Input
      type="text"
      value={value === undefined || value === null ? '' : String(value)}
      onChange={(e) => onChange(e.target.value)}
      pattern={pattern}
      maxLength={lengthRule?.max}
      placeholder={placeholder ?? param.display_name ?? param.name}
    />
  )
}

// ----------------------------------------------------------------- Label
/**
 * Standard label block for a parameter — name, required asterisk,
 * and a right-aligned range/unit hint. Reused by ParameterForm so all
 * parameters render with identical header chrome.
 */
export interface ParameterLabelProps {
  param: ParameterDefinition
}

export function ParameterLabel({ param }: ParameterLabelProps) {
  const { t } = useTranslation('devices')
  const hasRange =
    (param.min !== undefined && param.min !== null) ||
    (param.max !== undefined && param.max !== null)
  const rangeText = (() => {
    if (!hasRange) return null
    const min = param.min !== undefined && param.min !== null
      ? `${t('range.min', { defaultValue: 'min' })} ${param.min}`
      : null
    const max = param.max !== undefined && param.max !== null
      ? `${t('range.max', { defaultValue: 'max' })} ${param.max}`
      : null
    return [min, max].filter(Boolean).join(' | ')
  })()

  return (
    <div className="flex items-center justify-between gap-2">
      <Label className="text-sm">
        {param.display_name || param.name}
        {param.required && <span className="text-error ml-1">*</span>}
      </Label>
      {rangeText && (
        <span className="text-xs text-muted-foreground whitespace-nowrap">
          {rangeText}
        </span>
      )}
    </div>
  )
}
