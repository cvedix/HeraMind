/**
 * ParameterForm — iterates a command's parameters and renders them with
 * consistent grouping, conditional visibility, and validation hints.
 *
 * Responsibilities:
 *   - Hide parameters whose `default_value` is set AND `hideDefault=true`
 *     (these are "fixed" values the user shouldn't see).
 *   - Apply `visible_when` filtering using `lib/parameterExpr`.
 *   - Group parameters via `parameter_groups`, rendering each group inside
 *     a `Collapsible`. Ungrouped parameters render in a default group.
 *   - Render label + range/unit hint via `ParameterLabel`.
 *   - Route the actual input control to `ParameterInput`.
 *
 * This is the second half of the shared parameter rendering pipeline:
 *   ParameterForm (this file) → per-parameter shell → ParameterInput (control)
 */

import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Info } from 'lucide-react'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import { Button } from '@/components/ui/button'
import { ChevronRight } from 'lucide-react'
import type {
  ParameterDefinition,
  ParameterGroup,
} from '@/types/device'
import { evalVisibleWhen } from '@/lib/parameterExpr'
import {
  ParameterInput,
  ParameterLabel,
} from './ParameterInput'

export interface ParameterFormProps {
  /** All declared parameters of the command. */
  parameters: ParameterDefinition[]
  /** Optional grouping metadata from the command definition. */
  groups?: ParameterGroup[]
  /** Current values keyed by parameter name. */
  values: Record<string, unknown>
  /** Called whenever any parameter value changes. */
  onChange: (name: string, value: unknown) => void
  /**
   * If `true` (default), parameters that declare a `default_value` are
   * hidden from the UI — the parent is expected to seed the defaults
   * into `values`. Set to `false` to render everything explicitly.
   */
  hideDefault?: boolean
  /** Visual variant for compact (mobile) vs. default rendering. */
  variant?: 'default' | 'compact'
  /** Render style: flat list (no group headers) vs. grouped sections. */
  grouped?: boolean
}

interface ResolvedGroup {
  id: string
  display_name: string
  description?: string
  collapsed_default?: boolean
  params: ParameterDefinition[]
}

/**
 * Partition parameters into groups. Anything not mentioned in any group
 * lands in a synthetic "General" group at the start.
 */
function resolveGroups(
  parameters: ParameterDefinition[],
  groups: ParameterGroup[] | undefined,
  t: (key: string, opts?: Record<string, unknown>) => string,
): ResolvedGroup[] {
  if (!groups || groups.length === 0) {
    return [
      {
        id: '_default',
        display_name: '',
        params: parameters,
      },
    ]
  }

  // Map parameter name → group id for fast lookup.
  const nameToGroup = new Map<string, string>()
  groups.forEach((g) => {
    g.parameters.forEach((name) => nameToGroup.set(name, g.id))
  })

  const byId = new Map<string, ParameterDefinition[]>()
  parameters.forEach((p) => {
    const gid = nameToGroup.get(p.name) ?? '_default'
    const list = byId.get(gid) ?? []
    list.push(p)
    byId.set(gid, list)
  })

  const resolved: ResolvedGroup[] = []
  // Ungrouped first — keeps backward compatibility with flat layouts.
  if (byId.has('_default')) {
    resolved.push({
      id: '_default',
      display_name: t('command.dialog.generalGroup', { defaultValue: 'General' }),
      params: byId.get('_default')!,
    })
  }
  // Then declared groups, in their declared order.
  for (const g of groups) {
    const params = byId.get(g.id)
    if (params && params.length > 0) {
      resolved.push({
        id: g.id,
        display_name: g.display_name,
        description: g.description,
        collapsed_default: g.collapsed,
        params,
      })
    }
  }
  return resolved
}

export function ParameterForm({
  parameters,
  groups,
  values,
  onChange,
  hideDefault = true,
  variant = 'default',
  grouped = false,
}: ParameterFormProps) {
  const { t } = useTranslation('devices')

  // ---------------------------------------------------- filter + group
  const visibleGroups = useMemo(() => {
    // First apply visibility rules.
    const visible = parameters.filter((p) => {
      if (hideDefault && p.default_value !== undefined) return false
      return evalVisibleWhen(p.visible_when, values)
    })
    return resolveGroups(visible, grouped ? groups : undefined, t)
  }, [parameters, groups, values, hideDefault, grouped, t])

  const hasAnyParameters = parameters.length > 0
  const hasVisibleParameters = visibleGroups.some((g) => g.params.length > 0)

  // ---------------------------------------------------------- empty states
  if (!hasAnyParameters) {
    return (
      <div className="flex items-center gap-2 p-3 rounded-lg bg-muted-50">
        <Info className="h-4 w-4 text-muted-foreground" />
        <span className="text-sm text-muted-foreground">
          {t('command.dialog.noParameters', {
            defaultValue: 'No parameters required',
          })}
        </span>
      </div>
    )
  }

  if (!hasVisibleParameters) {
    return (
      <div className="flex items-center gap-2 p-3 rounded-lg bg-success-light border border-success-light">
        <Info className="h-4 w-4 text-success" />
        <span className="text-sm text-success">
          {t('command.dialog.allParametersFixed', {
            defaultValue: 'All parameters have fixed values',
          })}
        </span>
      </div>
    )
  }

  // ----------------------------------------------------------- render
  return (
    <div className="space-y-4">
      {visibleGroups.map((group) => {
        if (group.params.length === 0) return null

        // Single default group with no name → flat render (no header).
        const isFlat = visibleGroups.length === 1 && !group.display_name
        if (isFlat) {
          return (
            <div key={group.id} className="space-y-4">
              {group.params.map((param) => (
                <ParameterShell
                  key={param.name}
                  param={param}
                  value={values[param.name]}
                  onChange={(v) => onChange(param.name, v)}
                  variant={variant}
                />
              ))}
            </div>
          )
        }

        return (
          <GroupSection key={group.id} group={group}>
            {group.params.map((param) => (
              <ParameterShell
                key={param.name}
                param={param}
                value={values[param.name]}
                onChange={(v) => onChange(param.name, v)}
                variant={variant}
              />
            ))}
          </GroupSection>
        )
      })}
    </div>
  )
}

// ---------------------------------------------------------- shell + group

interface ParameterShellProps {
  param: ParameterDefinition
  value: unknown
  onChange: (v: unknown) => void
  variant: 'default' | 'compact'
}

/** Label + input + help-text wrapper for one parameter. */
function ParameterShell({ param, value, onChange, variant }: ParameterShellProps) {
  return (
    <div className="space-y-2">
      <ParameterLabel param={param} />
      <ParameterInput
        param={param}
        value={value}
        onChange={onChange}
        variant={variant}
      />
      {param.help_text && (
        <p className="text-xs text-muted-foreground">{param.help_text}</p>
      )}
    </div>
  )
}

/** Collapsible group section. */
function GroupSection({
  group,
  children,
}: {
  group: ResolvedGroup
  children: React.ReactNode
}) {
  const [open, setOpen] = useState(!group.collapsed_default)
  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <CollapsibleTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className="w-full justify-start px-2 -ml-2 text-sm font-medium"
        >
          <ChevronRight
            className={`h-4 w-4 transition-transform ${open ? 'rotate-90' : ''}`}
          />
          <span>{group.display_name}</span>
          {group.description && (
            <span className="text-xs text-muted-foreground font-normal truncate ml-2">
              {group.description}
            </span>
          )}
        </Button>
      </CollapsibleTrigger>
      <CollapsibleContent className="space-y-4 pt-2">
        {children}
      </CollapsibleContent>
    </Collapsible>
  )
}
