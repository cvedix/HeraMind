/**
 * DataSourceIndicator
 *
 * Shows data source binding status next to config fields.
 * - Unbound: Small [DB] button with title hint
 * - Bound: Source summary line + unbind button
 */

import { useTranslation } from 'react-i18next'
import { Database, X, Zap } from 'lucide-react'
import { Button } from '@/components/ui/button'
import type { DataSource, DataSourceOrList } from '@/types/dashboard'
import { normalizeDataSource } from '@/types/dashboard'

// ============================================================================
// Source Summary Utility
// ============================================================================

export function getSourceSummary(ds: DataSource): string {
  const sourceId = ds.sourceId ?? ''
  switch (ds.type) {
    case 'device':
    case 'telemetry':
    case 'metric':
      return sourceId ? `${sourceId}:${ds.metricId ?? ds.property ?? ''}` : 'device'
    case 'device-info':
      return sourceId ? `${sourceId}:${ds.infoProperty ?? ''}` : 'device-info'
    case 'command':
      return sourceId ? `${sourceId}:${ds.command ?? ''}` : 'command'
    case 'system':
      return `sys:${ds.systemMetric ?? ''}`
    case 'extension':
    case 'extension-metric':
      return ds.extensionId ? `ext:${ds.extensionId}:${ds.extensionMetric ?? ''}` : 'ext'
    case 'extension-command':
      return ds.extensionId ? `cmd:${ds.extensionId}:${ds.extensionCommand ?? ''}` : 'cmd'
    case 'transform':
      return `tf:${ds.transformId ?? ''}`
    default:
      return sourceId || 'source'
  }
}

// ============================================================================
// DataSourceIndicator Component
// ============================================================================

interface DataSourceIndicatorProps {
  /** Current data source binding */
  dataSource?: DataSourceOrList
  /** Unbind callback */
  onUnbind: () => void
  /** Use Zap icon instead of Database (for command-type bindings) */
  commandStyle?: boolean
}

export function DataSourceIndicator({
  dataSource,
  onUnbind,
  commandStyle = false,
}: DataSourceIndicatorProps) {
  const { t } = useTranslation('dashboardComponents')

  const sources = normalizeDataSource(dataSource)
  const isBound = sources.length > 0

  if (!isBound) {
    return (
      <Button
        variant="ghost"
        size="icon"
        className="h-5 w-5 text-muted-foreground hover:text-primary shrink-0"
        tabIndex={-1}
        title={t('dataSource.configureInTab')}
      >
        <Database className="h-3.5 w-3.5" />
      </Button>
    )
  }

  // Bound state: show summary + unbind
  const summary = sources.map(getSourceSummary).join(', ')
  const Icon = commandStyle ? Zap : Database

  return (
    <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
      <Icon className="h-3 w-3 shrink-0 text-success" />
      <span className="truncate max-w-[140px]" title={summary}>
        {summary}
      </span>
      <Button
        variant="ghost"
        size="icon"
        className="h-4 w-4 text-muted-foreground hover:text-error shrink-0"
        onClick={onUnbind}
        title={t('dataSource.unbindDataSource')}
        aria-label={t('dataSource.unbindDataSource')}
      >
        <X className="h-3 w-3" />
      </Button>
    </div>
  )
}
