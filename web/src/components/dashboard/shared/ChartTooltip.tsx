/**
 * Shared chart tooltip component for Recharts.
 * Used by LineChart, BarChart, and PieChart.
 */

/**
 * Shared chart tooltip component for Recharts.
 * Used by LineChart, BarChart, and PieChart.
 */

/** Safely format a tooltip value — handles telemetry point objects */
function formatTooltipValue(val: unknown): string {
  if (val === null || val === undefined) return '-'
  if (typeof val === 'number') return String(val)
  if (typeof val === 'string') return val
  if (typeof val === 'boolean') return String(val)
  // Telemetry point object {timestamp, time, value}
  if (typeof val === 'object' && val !== null && 'value' in val) {
    return formatTooltipValue((val as { value: unknown }).value)
  }
  return JSON.stringify(val)
}

export function ChartTooltip({ active, payload, label }: { active?: boolean; payload?: any[]; label?: string }) {
  if (!active || !payload?.length) return null

  return (
    <div className="rounded-lg border bg-background p-2 shadow-md">
      {label && <div className="mb-1 text-xs text-muted-foreground">{label}</div>}
      <div className="grid gap-1.5 text-xs">
        {payload.map((entry: any, index: number) => (
          <div key={index} className="flex items-center gap-2">
            <div
              className="h-2 w-2 shrink-0 rounded-[2px]"
              style={{ backgroundColor: entry.color }}
            />
            <span className="text-muted-foreground font-medium">{entry.name}:</span>
            <span className="tabular-nums font-semibold">{formatTooltipValue(entry.value)}</span>
          </div>
        ))}
      </div>
    </div>
  )
}
