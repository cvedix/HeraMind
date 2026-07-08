/**
 * DeliveryHistoryPanel - delivery logs for a push target in FullScreenDialog with ResponsiveTable.
 */

import { useEffect, useState, useCallback } from 'react'
import { useStore } from '@/store'
import { useTranslation } from 'react-i18next'
import { FileText, CheckCircle2, XCircle, Clock, AlertTriangle } from 'lucide-react'
import { ResponsiveTable, type TableColumn, EmptyState, Pagination } from '@/components/shared'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import { textMini, textNano } from '@/design-system/tokens/typography'
import {
  FullScreenDialog,
  FullScreenDialogHeader,
  FullScreenDialogContent,
  FullScreenDialogMain,
} from '@/components/automation/dialog'
import type { DeliveryLog, DeliveryStatus } from '@/types'

const PAGE_SIZE = 10

interface DeliveryHistoryPanelProps {
  targetId: string
  open: boolean
  onOpenChange: (open: boolean) => void
}

const STATUS_CONFIG: Record<DeliveryStatus, { color: string; bg: string; icon: typeof CheckCircle2; label: string }> = {
  success: { color: 'text-success', bg: 'bg-success-light', icon: CheckCircle2, label: 'SUCCESS' },
  failed: { color: 'text-error', bg: 'bg-error-light', icon: XCircle, label: 'FAILED' },
  pending: { color: 'text-warning', bg: 'bg-warning-light', icon: Clock, label: 'PENDING' },
  retrying: { color: 'text-warning', bg: 'bg-warning-light', icon: AlertTriangle, label: 'RETRYING' },
}

export function DeliveryHistoryPanel({ targetId, open, onOpenChange }: DeliveryHistoryPanelProps) {
  const { t } = useTranslation()
  const { deliveryLogs, deliveryLogsTotal, deliveryLogsLoading, fetchDeliveryLogs } = useStore()
  const [page, setPage] = useState(1)

  const loadPage = useCallback((p: number) => {
    setPage(p)
    if (targetId) {
      fetchDeliveryLogs(targetId, PAGE_SIZE, (p - 1) * PAGE_SIZE)
    }
  }, [targetId, fetchDeliveryLogs])

  useEffect(() => {
    if (open && targetId) {
      setPage(1)
      fetchDeliveryLogs(targetId, PAGE_SIZE, 0)
    }
  }, [open, targetId, fetchDeliveryLogs])

  const columns: TableColumn[] = [
    { key: 'status', label: t('common:dataPush.status', 'Status'), width: '12%' },
    { key: 'source', label: 'Source', width: '30%' },
    { key: 'payload', label: 'Payload', width: '35%' },
    { key: 'attempts', label: t('common:dataPush.attemptsLabel', 'Attempts'), width: '10%' },
    { key: 'time', label: t('common:dataPush.updated', 'Time'), width: '13%' },
  ]

  const renderCell = (columnKey: string, rowData: Record<string, unknown>) => {
    const log = rowData as unknown as DeliveryLog
    switch (columnKey) {
      case 'status': {
        const cfg = STATUS_CONFIG[log.status] || STATUS_CONFIG.pending
        const Icon = cfg.icon
        return (
          <Badge variant="outline" className={cn(textNano, "h-5 gap-1 px-1.5 border-0", cfg.bg, cfg.color)}>
            <Icon className="h-3 w-3" />
            {cfg.label}
          </Badge>
        )
      }
      case 'source':
        return (
          <span className={cn(textMini, "truncate block max-w-[250px]")}>{log.data_source_id}</span>
        )
      case 'payload': {
        if (!log.payload_sent) return <span className={cn(textMini, "text-muted-foreground")}>-</span>
        let preview = log.payload_sent
        try {
          const p = JSON.parse(log.payload_sent)
          preview = JSON.stringify(p)
        } catch { /* use raw */ }
        return (
          <code className={cn(textNano, "text-muted-foreground font-mono truncate block max-w-[250px]")}>
            {preview.length > 120 ? preview.slice(0, 117) + '...' : preview}
          </code>
        )
      }
      case 'attempts':
        return <span className={cn(textMini)}>{log.attempts}</span>
      case 'time':
        return (
          <span className={cn(textMini, "text-muted-foreground")}>
            {log.created_at ? new Date(log.created_at * 1000).toLocaleString() : '-'}
          </span>
        )
      default:
        return null
    }
  }

  return (
    <FullScreenDialog open={open} onOpenChange={onOpenChange}>
      <FullScreenDialogHeader
        icon={<FileText className="h-5 w-5" />}
        iconBg="bg-info-light"
        iconColor="text-info"
        title={t('common:dataPush.deliveryHistory', 'Delivery History')}
        onClose={() => onOpenChange(false)}
      />
      <FullScreenDialogContent>
        <FullScreenDialogMain className="overflow-hidden">
          <div className="h-full flex flex-col">
            <div className="flex-1 overflow-y-auto px-4 py-4">
              <ResponsiveTable
                columns={columns}
                data={deliveryLogs as unknown as Record<string, unknown>[]}
                renderCell={renderCell}
                rowKey={(row) => (row as unknown as DeliveryLog).id}
                loading={deliveryLogsLoading}
                flexHeight={false}
                emptyState={
                  <EmptyState
                    icon={<FileText className="h-12 w-12" />}
                    title={t('common:dataPush.noLogs', 'No delivery logs')}
                    description={t('common:dataPush.noLogsHint', 'Logs will appear here after data is pushed')}
                  />
                }
              />
            </div>
            {deliveryLogsTotal > PAGE_SIZE && (
              <div className="border-t px-4 py-3">
                <Pagination
                  total={deliveryLogsTotal}
                  pageSize={PAGE_SIZE}
                  currentPage={page}
                  onPageChange={loadPage}
                  hideOnMobile={false}
                />
              </div>
            )}
          </div>
        </FullScreenDialogMain>
      </FullScreenDialogContent>
    </FullScreenDialog>
  )
}
