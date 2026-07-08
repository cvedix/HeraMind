import { useState } from 'react'
import type { ExecutionPlan } from '../../types'

type StepStatus = 'pending' | 'running' | 'completed' | 'failed'

interface ExecutionPlanPanelProps {
  plan: ExecutionPlan
  stepStates: Record<number, StepStatus>
}

export function ExecutionPlanPanel({ plan, stepStates }: ExecutionPlanPanelProps) {
  const [collapsed, setCollapsed] = useState(false)

  const allDone = plan.steps.every(
    (_, i) => {
      const s = stepStates[i]
      return s === 'completed' || s === 'failed'
    }
  )

  const statusIcon = (status: StepStatus) => {
    switch (status) {
      case 'completed': return '✅'
      case 'running': return '⏳'
      case 'failed': return '❌'
      default: return '⬜'
    }
  }

  return (
    <div className="my-2 border border-border rounded-lg overflow-hidden">
      <button
        className="w-full flex items-center justify-between px-3 py-2 bg-muted-50 text-sm hover:bg-muted transition-colors"
        onClick={() => setCollapsed(!collapsed)}
      >
        <span className="font-medium">
          Execution Plan ({plan.steps.length} steps, {plan.mode === 'keyword' ? 'fast' : 'detailed'})
        </span>
        <span className="text-xs text-muted-foreground">
          {collapsed ? '▶' : '▼'} {allDone ? 'Done' : 'Running...'}
        </span>
      </button>

      {!collapsed && (
        <div className="px-3 py-2 space-y-1.5">
          {plan.steps.map((step) => {
            const status = stepStates[step.id] ?? 'pending'
            return (
              <div key={step.id} className="flex items-start gap-2 text-sm">
                <span className="mt-0.5">{statusIcon(status)}</span>
                <div className="flex-1 min-w-0">
                  <div className="truncate">{step.description}</div>
                </div>
                <span className="text-xs text-muted-foreground shrink-0">
                  {step.tool_name}:{step.action}
                </span>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
