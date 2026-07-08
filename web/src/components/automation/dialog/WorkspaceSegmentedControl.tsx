/**
 * WorkspaceSegmentedControl — accessible segmented control for switching
 * the builder workspace canvas (e.g. 触发条件 / 执行动作).
 */
import { cn } from '@/lib/utils'
import type { BuilderAccent } from './BuilderShell'

export interface WorkspaceSegment {
  value: string
  label: string
  count?: number
}

export interface WorkspaceSegmentedControlProps {
  segments: WorkspaceSegment[]
  value: string
  onChange: (value: string) => void
  accent: BuilderAccent
  className?: string
}

const accentText: Record<BuilderAccent, string> = {
  indigo: 'text-accent-indigo',
  emerald: 'text-accent-emerald',
}

export function WorkspaceSegmentedControl({
  segments,
  value,
  onChange,
  accent,
  className,
}: WorkspaceSegmentedControlProps) {
  return (
    <div
      className={cn('inline-flex w-fit gap-1 rounded-lg bg-muted-30 p-1', className)}
      role="tablist"
    >
      {segments.map((s) => {
        const active = s.value === value
        return (
          <button
            key={s.value}
            type="button"
            role="tab"
            aria-selected={active}
            onClick={() => onChange(s.value)}
            className={cn(
              'rounded-md px-3.5 py-1.5 text-sm font-medium transition-colors',
              active
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'
            )}
          >
            {s.label}
            {s.count !== undefined && (
              <span className={cn('ml-1.5', active ? accentText[accent] : 'text-muted-foreground')}>
                {s.count}
              </span>
            )}
          </button>
        )
      })}
    </div>
  )
}