import { useState, useRef, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { format } from 'date-fns'
import { UnifiedFormDialog } from '@/components/dialog/UnifiedFormDialog'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Calendar } from '@/components/ui/calendar'

import { Download, CalendarIcon } from 'lucide-react'
import { api } from '@/lib/api'
import { useToast } from '@/hooks/use-toast'
import { textNano } from '@/design-system/tokens/typography'
import { cn } from '@/lib/utils'
import type { UnifiedDataSourceInfo } from '@/types'

interface ExportDataDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  source: UnifiedDataSourceInfo | null
}

/** Excel cell max length */
const EXCEL_CELL_MAX = 32000

/** Format Date for display: "yyyy-MM-dd HH:mm:ss" */
function formatDateTimeDisplay(date: Date): string {
  return format(date, 'yyyy-MM-dd HH:mm:ss')
}

/** Format Unix seconds to human-readable "yyyy-MM-dd HH:mm:ss" */
function formatTimestamp(unixSeconds: number): string {
  return formatDateTimeDisplay(new Date(unixSeconds * 1000))
}

/** Sanitize string for use in filenames */
function sanitizeFilename(s: string): string {
  return s.replace(/[:\s]/g, '_').replace(/[^a-zA-Z0-9_\-.]/g, '')
}

/** Get current time as filename-safe string: "YYYYMMDD_HHmmss" */
function fileTimestamp(): string {
  return format(new Date(), 'yyyyMMdd_HHmmss')
}

/**
 * Time Picker following shadcn/ui convention:
 * Uses <Input type="time"> with native picker indicator hidden via CSS.
 * Renders as a clean text input (HH:mm:ss) that works in Tauri.
 */
function TimePicker({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <Input
      type="time"
      step="1"
      value={value}
      onChange={e => onChange(e.target.value)}
      className="h-9 w-full text-sm appearance-none bg-background [&::-webkit-calendar-picker-indicator]:hidden [&::-webkit-calendar-picker-indicator]:appearance-none"
    />
  )
}

/** Truncate value string to Excel cell limit */
function truncateForExcel(val: string): string {
  if (val.length <= EXCEL_CELL_MAX) return val
  return val.slice(0, EXCEL_CELL_MAX) + '...[truncated]'
}

/** Check if a value looks like base64 image data */
function isBase64Image(value: unknown): value is string {
  if (typeof value !== 'string') return false
  return value.startsWith('data:image/') || value.startsWith('data:application/octet-stream')
}

/** Check if a string is likely base64-encoded binary data */
function isBase64Binary(value: unknown): value is string {
  if (typeof value !== 'string') return false
  // Long strings that match base64 pattern (no data: prefix)
  if (value.length < 100) return false
  return /^[A-Za-z0-9+/=\s]+$/.test(value.slice(0, 200))
}

/** Detect if the data contains image/binary content that should go to ZIP */
function detectImageContent(data: Array<{ value: unknown }>): boolean {
  if (data.length === 0) return false
  // Check first few data points
  const sample = data.slice(0, 5)
  return sample.some(p => isBase64Image(p.value))
}

/** Extract MIME type and extension from data URI or guess from base64 */
function parseImageData(value: string): { mime: string; ext: string; base64: string } {
  const dataUriMatch = value.match(/^data:([^;]+);base64,(.+)$/s)
  if (dataUriMatch) {
    const mime = dataUriMatch[1]
    const base64 = dataUriMatch[2]
    const extMap: Record<string, string> = {
      'image/png': 'png', 'image/jpeg': 'jpg', 'image/jpg': 'jpg',
      'image/gif': 'gif', 'image/webp': 'webp', 'image/bmp': 'bmp',
      'image/svg+xml': 'svg',
    }
    return { mime, ext: extMap[mime] || 'bin', base64 }
  }
  // Raw base64 — assume png
  return { mime: 'image/png', ext: 'png', base64: value }
}

export function ExportDataDialog({ open, onOpenChange, source }: ExportDataDialogProps) {
  const { t } = useTranslation('data')
  const { toast } = useToast()
  const abortRef = useRef<AbortController | null>(null)

  const now = new Date()
  const defaultStart = new Date(now.getTime() - 24 * 3600 * 1000)
  const [startDate, setStartDate] = useState<Date>(defaultStart)
  const [startTime, setStartTime] = useState(format(defaultStart, 'HH:mm:ss'))
  const [endDate, setEndDate] = useState<Date>(now)
  const [endTime, setEndTime] = useState(format(now, 'HH:mm:ss'))
  const [exporting, setExporting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Reset state when source changes
  const handleOpenChange = useCallback((nextOpen: boolean) => {
    if (!nextOpen) {
      abortRef.current?.abort()
      setExporting(false)
      setError(null)
    }
    onOpenChange(nextOpen)
  }, [onOpenChange])

  const handleExport = useCallback(async () => {
    if (!source) return

    abortRef.current?.abort()
    const controller = new AbortController()
    abortRef.current = controller

    setError(null)
    setExporting(true)

    try {
      // Parse DataSourceId: "device:sensor1:temperature" → source="device:sensor1", metric="temperature"
      const parts = source.id.split(':')
      const sourcePart = `${parts[0]}:${parts[1]}`
      const metricPart = parts.slice(2).join(':')

      // Combine date + time into full Date objects, then convert to Unix seconds
      const [startH, startM, startS] = startTime.split(':').map(Number)
      const startDateFull = new Date(startDate)
      startDateFull.setHours(startH, startM, startS || 0, 0)

      const [endH, endM, endS] = endTime.split(':').map(Number)
      const endDateFull = new Date(endDate)
      endDateFull.setHours(endH, endM, endS || 0, 0)

      const startUnix = Math.floor(startDateFull.getTime() / 1000)
      const endUnix = Math.floor(endDateFull.getTime() / 1000)

      const result = await api.queryTelemetry(sourcePart, metricPart, startUnix, endUnix, 1000)
      if (controller.signal.aborted) return

      const data = result?.data || []
      if (data.length === 0) {
        setError(t('export.noData'))
        return
      }

      // Warn about truncation
      if (result.total_count != null && result.total_count > data.length) {
        toast({
          title: t('export.truncated', { count: data.length, total: result.total_count }),
        })
      }

      // Generate file — auto-detect image content from actual data values
      const baseName = `${sanitizeFilename(source.source_display_name)}_${sanitizeFilename(source.field_display_name)}_${fileTimestamp()}`
      const isImageContent = detectImageContent(data)

      if (isImageContent || source.data_type === 'binary') {
        await generateZip(baseName, data, source)
        toast({
          title: t('export.success'),
          description: `${baseName}.zip`,
        })
      } else {
        await generateExcel(baseName, data)
        toast({
          title: t('export.success'),
          description: `${baseName}.xlsx`,
        })
      }

      // Close dialog on success
      handleOpenChange(false)
    } catch (err) {
      if (controller.signal.aborted) return
      console.error('[ExportDataDialog] Export failed:', err)
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      if (!controller.signal.aborted) {
        setExporting(false)
      }
    }
  }, [source, startDate, startTime, endDate, endTime, t, toast, handleOpenChange])

  return (
    <UnifiedFormDialog
      open={open}
      onOpenChange={handleOpenChange}
      title={t('export.title')}
      description={source?.id}
      icon={<Download className="h-5 w-5" />}
      width="md"
      submitLabel={exporting ? t('export.exporting') : t('export.button')}
      onSubmit={handleExport}
      isSubmitting={exporting}
      submitDisabled={exporting}
      showCancelButton
    >
      {source && (
        <div className="space-y-4">
          {/* Source Info */}
          <div className="grid grid-cols-2 gap-3">
            <div>
              <p className="text-xs text-muted-foreground">{t('columns.source')}</p>
              <p className="text-sm font-medium">{source.source_display_name}</p>
            </div>
            <div>
              <p className="text-xs text-muted-foreground">{t('columns.field')}</p>
              <p className="text-sm font-medium">{source.field_display_name}</p>
            </div>
            <div>
              <p className="text-xs text-muted-foreground">{t('columns.dataType')}</p>
              <Badge variant="secondary" className={textNano}>{source.data_type}</Badge>
            </div>
            <div>
              <p className="text-xs text-muted-foreground">{t('export.format.auto')}</p>
              <p className="text-sm">{t('export.format.autoDesc')}</p>
            </div>
          </div>

          {/* Time Range - two columns: start | end */}
          <div className="grid grid-cols-2 gap-4">
            {/* Start */}
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground block">{t('export.startTime')}</label>
              <Popover>
                <PopoverTrigger asChild>
                  <Button
                    variant="outline"
                    className={cn(
                      "h-9 w-full justify-start text-left text-sm font-normal",
                      !startDate && "text-muted-foreground"
                    )}
                  >
                    <CalendarIcon className="mr-1.5 h-3.5 w-3.5 shrink-0" />
                    {startDate ? format(startDate, 'yyyy-MM-dd') : 'Pick date'}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-auto p-0" align="start">
                  <Calendar
                    mode="single"
                    selected={startDate}
                    onSelect={(d) => d && setStartDate(d)}
                    initialFocus
                  />
                </PopoverContent>
              </Popover>
              <TimePicker value={startTime} onChange={setStartTime} />
            </div>

            {/* End */}
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground block">{t('export.endTime')}</label>
              <Popover>
                <PopoverTrigger asChild>
                  <Button
                    variant="outline"
                    className={cn(
                      "h-9 w-full justify-start text-left text-sm font-normal",
                      !endDate && "text-muted-foreground"
                    )}
                  >
                    <CalendarIcon className="mr-1.5 h-3.5 w-3.5 shrink-0" />
                    {endDate ? format(endDate, 'yyyy-MM-dd') : 'Pick date'}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-auto p-0" align="start">
                  <Calendar
                    mode="single"
                    selected={endDate}
                    onSelect={(d) => d && setEndDate(d)}
                    initialFocus
                  />
                </PopoverContent>
              </Popover>
              <TimePicker value={endTime} onChange={setEndTime} />
            </div>
          </div>

          {/* Error */}
          {error && (
            <p className="text-sm text-error bg-error-light rounded px-3 py-2">{error}</p>
          )}
        </div>
      )}
    </UnifiedFormDialog>
  )
}

/** Generate and download an Excel file */
async function generateExcel(
  baseName: string,
  data: Array<{ timestamp: number; value: unknown; quality: number | null }>,
) {
  const XLSX = await import('xlsx')

  const rows = data.map(p => {
    const rawValue = typeof p.value === 'object' ? JSON.stringify(p.value) : String(p.value ?? '')
    return {
      Timestamp: formatTimestamp(p.timestamp),
      Value: truncateForExcel(rawValue),
      Quality: p.quality !== null ? `${(p.quality * 100).toFixed(0)}%` : '-',
    }
  })

  const ws = XLSX.utils.json_to_sheet(rows)
  // Auto-width columns
  ws['!cols'] = [
    { wch: 22 }, // Timestamp
    { wch: 30 }, // Value (wider to show more content)
    { wch: 10 }, // Quality
  ]

  const wb = XLSX.utils.book_new()
  XLSX.utils.book_append_sheet(wb, ws, 'Data')
  XLSX.writeFile(wb, `${baseName}.xlsx`)
}

/** Generate and download a ZIP file with decoded images */
async function generateZip(
  baseName: string,
  data: Array<{ timestamp: number; value: unknown; quality: number | null }>,
  source: UnifiedDataSourceInfo,
) {
  const JSZip = (await import('jszip')).default
  const zip = new JSZip()
  let imageCount = 0

  // manifest.json with metadata
  zip.file('manifest.json', JSON.stringify({
    source: source.id,
    source_name: source.source_display_name,
    field: source.field_display_name,
    data_type: source.data_type,
    point_count: data.length,
    exported_at: new Date().toISOString(),
  }, null, 2))

  // Process each data point
  for (let i = 0; i < data.length; i++) {
    const p = data[i]
    const valStr = typeof p.value === 'string' ? p.value : JSON.stringify(p.value)
    const timeStr = formatTimestamp(p.timestamp).replace(/[: ]/g, '-')

    if (isBase64Image(p.value) || isBase64Binary(p.value)) {
      // Decode base64 image → actual file
      const parsed = parseImageData(valStr)
      const filename = `images/${String(i + 1).padStart(4, '0')}_${timeStr}.${parsed.ext}`
      // Convert base64 to binary
      const binaryStr = atob(parsed.base64)
      const bytes = new Uint8Array(binaryStr.length)
      for (let j = 0; j < binaryStr.length; j++) {
        bytes[j] = binaryStr.charCodeAt(j)
      }
      zip.file(filename, bytes)
      imageCount++
    } else {
      // Non-image data point → CSV reference
      const q = p.quality !== null ? `${(p.quality * 100).toFixed(0)}%` : '-'
      const shortVal = valStr.length > 200 ? valStr.slice(0, 200) + '...' : valStr
      // Will be collected into data.csv below
    }
  }

  // data.csv with index mapping + non-image data
  const csvRows = ['Index,Filename,Timestamp,Quality,Size']
  for (let i = 0; i < data.length; i++) {
    const p = data[i]
    const valStr = typeof p.value === 'string' ? p.value : JSON.stringify(p.value)
    const timeStr = formatTimestamp(p.timestamp).replace(/[: ]/g, '-')
    const q = p.quality !== null ? `${(p.quality * 100).toFixed(0)}%` : '-'

    if (isBase64Image(p.value) || isBase64Binary(p.value)) {
      const parsed = parseImageData(valStr)
      const filename = `${String(i + 1).padStart(4, '0')}_${timeStr}.${parsed.ext}`
      csvRows.push(`${i + 1},${filename},${formatTimestamp(p.timestamp)},${q},${parsed.base64.length} bytes`)
    } else {
      csvRows.push(`${i + 1},-,${formatTimestamp(p.timestamp)},${q},"${valStr.replace(/"/g, '""')}"`)
    }
  }
  zip.file('data.csv', csvRows.join('\n'))

  const blob = await zip.generateAsync({ type: 'blob' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${baseName}.zip`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  setTimeout(() => URL.revokeObjectURL(url), 100)
}
