/**
 * Event List
 *
 * Reviews structured events stored as raw telemetry. The normalizer supports
 * HeraMind telemetry points, JSON strings, and HERACAM-style record arrays
 * containing event, crop, and attribute records.
 */

import { memo, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ChevronLeft,
  ChevronRight,
  Eye,
  ImageOff,
  ListFilter,
  Search,
} from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button, IconButton } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useDataSource } from '@/hooks/useDataSource'
import { normalizeImageUrl } from '@/lib/imageUtils'
import { cn } from '@/lib/utils'
import type { DataSource } from '@/types/dashboard'
import { getSourceId } from '@/types/dashboard'
import { LoadingState } from '../shared'

type UnknownRecord = Record<string, unknown>

export interface EventListItem {
  id: string
  timestamp: number
  eventType: string
  objectClass: string
  color: string
  direction: string
  confidence?: number
  tripwire: string
  image?: string
  attributes: Record<string, unknown>
  event: UnknownRecord
  crop?: UnknownRecord
}

export interface EventListProps {
  dataSource?: DataSource
  title?: string
  limit?: number
  timeRange?: number
  pageSize?: number
  showSearch?: boolean
  showFilters?: boolean
  showImage?: boolean
  className?: string
}

function isRecord(value: unknown): value is UnknownRecord {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function asString(value: unknown, fallback = ''): string {
  if (value === null || value === undefined) return fallback
  const text = String(value).trim()
  return text || fallback
}

function asNumber(value: unknown): number | undefined {
  const number = Number(value)
  return Number.isFinite(number) ? number : undefined
}

function parseJson(value: unknown): unknown {
  if (typeof value !== 'string') return value
  try {
    return JSON.parse(value)
  } catch {
    return value
  }
}

function normalizeTimestamp(value: unknown, fallback: number): number {
  if (typeof value === 'string') {
    const parsed = Date.parse(value)
    if (!Number.isNaN(parsed)) return parsed
  }
  const numeric = asNumber(value)
  if (numeric === undefined || numeric <= 0) return fallback
  // Camera `event_timestamp_ms` values can be monotonic offsets since boot,
  // not Unix timestamps. Prefer the telemetry point time for those values.
  if (numeric < 946684800) return fallback
  if (numeric < 1e11) return numeric * 1000
  return numeric
}

function extractImage(value: unknown): string | undefined {
  if (value !== null && value !== undefined && typeof value !== 'string' && typeof value !== 'number') {
    return undefined
  }
  const normalized = normalizeImageUrl(value)
  return normalized?.src
}

function telemetryPayload(point: unknown): { payload: unknown; timestamp: number } {
  if (!isRecord(point)) {
    return { payload: parseJson(point), timestamp: Date.now() }
  }

  const timestamp = normalizeTimestamp(
    point.timestamp ?? point.time ?? point.t,
    Date.now(),
  )
  const payload = point.value ?? point.v ?? point.payload ?? point.data ?? point
  return { payload: parseJson(payload), timestamp }
}

function eventRecordsFromPayload(payload: unknown): UnknownRecord[] {
  const parsed = parseJson(payload)
  if (Array.isArray(parsed)) return parsed.filter(isRecord)
  if (!isRecord(parsed)) return []

  for (const key of ['events', 'records', 'items', 'data', 'values']) {
    const nested = parseJson(parsed[key])
    if (Array.isArray(nested)) return nested.filter(isRecord)
  }
  return [parsed]
}

function matchesReference(record: UnknownRecord, event: UnknownRecord): boolean {
  const eventId = asString(event.event_id ?? event.id)
  const trackingId = asString(event.ref_tracking_id ?? event.tracking_id)
  const recordEventId = asString(record.ref_event_id ?? record.event_id)
  const recordTrackingId = asString(record.ref_tracking_id ?? record.tracking_id)

  return Boolean(
    (eventId && recordEventId && eventId === recordEventId) ||
    (trackingId && recordTrackingId && trackingId === recordTrackingId),
  )
}

function isPrimaryEvent(record: UnknownRecord): boolean {
  const type = asString(record.$id ?? record.event_type ?? record.type).toLowerCase()
  return type.startsWith('event-') ||
    Boolean(record.object_class) ||
    type === 'vehicle' ||
    type === 'vehicle_event'
}

function normalizePayload(payload: unknown, pointTimestamp: number, pointIndex: number): EventListItem[] {
  const records = eventRecordsFromPayload(payload)
  if (records.length === 0) return []

  const eventRecords = records.filter(isPrimaryEvent)
  // A pre-normalized object is also a valid event even without `$id`.
  const primaryEvents = eventRecords.length > 0
    ? eventRecords
    : records.length === 1 ? records : []

  const attributeRecords = records.filter((record) => {
    const type = asString(record.$id ?? record.type).toLowerCase()
    return type === 'attribute' || (record.name !== undefined && record.value !== undefined)
  })
  const cropRecords = records.filter((record) => {
    const type = asString(record.$id ?? record.type).toLowerCase()
    return type === 'crop' || record.image !== undefined || record.image_url !== undefined
  })

  return primaryEvents.map((event, eventIndex) => {
    const relatedAttributes = attributeRecords.filter((attribute) =>
      matchesReference(attribute, event) || primaryEvents.length === 1,
    )
    const attributes: Record<string, unknown> = {}
    for (const attribute of relatedAttributes) {
      const name = asString(attribute.name ?? attribute.attribute)
      if (name) attributes[name] = attribute.value
    }

    // Pre-normalized event objects can carry attributes as an object.
    if (isRecord(event.attributes)) {
      Object.assign(attributes, event.attributes)
    }

    const crop = cropRecords
      .filter((candidate) => matchesReference(candidate, event) || primaryEvents.length === 1)
      .sort((left, right) => (asNumber(right.confidence) ?? 0) - (asNumber(left.confidence) ?? 0))[0]

    const timestamp = normalizeTimestamp(
      event.system_timestamp ??
      event.timestamp ??
      event.system_datetime ??
      event.event_timestamp_ms ??
      crop?.system_timestamp ??
      crop?.timestamp,
      pointTimestamp,
    )
    const eventId = asString(
      event.event_id ?? event.id ?? crop?.ref_event_id,
      `event-${pointTimestamp}-${pointIndex}-${eventIndex}`,
    )
    const eventType = asString(event.$id ?? event.event_type ?? event.type, 'event')
    const objectClass = asString(
      attributes.vehicle_class ?? attributes.object_class ?? event.vehicle_class ?? event.object_class,
      'unknown',
    )
    const color = asString(
      attributes.vehicle_color ?? attributes.color ?? event.vehicle_color ?? event.color,
      'unknown',
    )
    const direction = asString(
      attributes.crossing_direction ?? event.crossing_direction ?? event.direction,
      'unknown',
    )
    const confidence = asNumber(
      crop?.confidence ?? attributes.confidence ?? event.confidence,
    )
    const tripwire = asString(
      event.tripwire_name ?? attributes.tripwire_name ?? event.tripwire_id,
      'unknown',
    )

    return {
      id: eventId,
      timestamp,
      eventType,
      objectClass,
      color,
      direction,
      confidence,
      tripwire,
      image: extractImage(crop?.image ?? crop?.image_url ?? event.image),
      attributes,
      event,
      crop,
    }
  })
}

/** Exported for focused unit tests and extension authors reusing the format. */
// eslint-disable-next-line react-refresh/only-export-components
export function toEventListItems(data: unknown): EventListItem[] {
  const source = Array.isArray(data) ? data : data == null ? [] : [data]
  const result = source.flatMap((point, index) => {
    const { payload, timestamp } = telemetryPayload(point)
    return normalizePayload(payload, timestamp, index)
  })

  return result
    .sort((left, right) => right.timestamp - left.timestamp)
    .filter((item, index, items) =>
      items.findIndex((candidate) =>
        candidate.id === item.id && candidate.timestamp === item.timestamp,
      ) === index,
    )
}

function normalizeDataSourceForEvents(
  dataSource: DataSource | undefined,
  limit: number,
  timeRange: number,
): DataSource | undefined {
  if (!dataSource) return undefined
  if (dataSource.type === 'telemetry' || dataSource.type === 'transform') {
    return {
      ...dataSource,
      timeRange,
      limit,
      aggregateExt: 'raw',
      transform: 'raw',
      params: { ...dataSource.params, includeRawPoints: true },
    }
  }

  if (dataSource.type === 'device' || dataSource.type === 'metric') {
    return {
      type: 'telemetry',
      sourceId: getSourceId(dataSource),
      metricId: dataSource.metricId ?? dataSource.property ?? '_raw',
      timeRange,
      limit,
      aggregateExt: 'raw',
      transform: 'raw',
      params: { includeRawPoints: true },
      refresh: dataSource.refresh ?? 30,
    }
  }
  return dataSource
}

function formatEventTime(timestamp: number): string {
  return new Intl.DateTimeFormat(undefined, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(timestamp)
}

function formatConfidence(value: number | undefined): string {
  if (value === undefined) return '—'
  const percentage = value <= 1 ? value * 100 : value
  return `${percentage.toFixed(1)}%`
}

function EventThumbnail({ item, className }: { item: EventListItem; className?: string }) {
  if (!item.image) {
    return (
      <div className={cn('flex items-center justify-center rounded-md bg-muted text-muted-foreground', className)}>
        <ImageOff className="h-4 w-4" />
      </div>
    )
  }
  return (
    <img
      src={item.image}
      alt={item.objectClass}
      className={cn('rounded-md bg-muted object-cover', className)}
    />
  )
}

export const EventList = memo(function EventList({
  dataSource,
  title,
  limit = 200,
  timeRange = 48,
  pageSize = 10,
  showSearch = true,
  showFilters = true,
  showImage = true,
  className,
}: EventListProps) {
  const { t } = useTranslation('dashboardComponents')
  const effectiveSource = useMemo(
    () => normalizeDataSourceForEvents(dataSource, limit, timeRange),
    [dataSource, limit, timeRange],
  )
  const { data, loading, error, lastUpdate } = useDataSource(effectiveSource)
  const events = useMemo(() => toEventListItems(data), [data])
  const [search, setSearch] = useState('')
  const [classFilter, setClassFilter] = useState('__all__')
  const [page, setPage] = useState(1)
  const [selectedEvent, setSelectedEvent] = useState<EventListItem | null>(null)

  const classOptions = useMemo(
    () => Array.from(new Set(events.map((event) => event.objectClass).filter(Boolean))).sort(),
    [events],
  )
  const filteredEvents = useMemo(() => {
    const query = search.trim().toLowerCase()
    return events.filter((event) => {
      if (classFilter !== '__all__' && event.objectClass !== classFilter) return false
      if (!query) return true
      const searchable = [
        event.id,
        event.eventType,
        event.objectClass,
        event.color,
        event.direction,
        event.tripwire,
        ...Object.entries(event.attributes).flatMap(([key, value]) => [key, String(value)]),
      ].join(' ').toLowerCase()
      return searchable.includes(query)
    })
  }, [events, search, classFilter])

  const totalPages = Math.max(1, Math.ceil(filteredEvents.length / Math.max(1, pageSize)))
  const paginatedEvents = filteredEvents.slice((page - 1) * pageSize, page * pageSize)

  useEffect(() => setPage(1), [search, classFilter, pageSize])
  useEffect(() => {
    if (page > totalPages) setPage(totalPages)
  }, [page, totalPages])

  if (loading && !lastUpdate) {
    return <LoadingState className={className} />
  }

  return (
    <>
      <Card className={cn('h-full min-h-0 overflow-hidden flex flex-col', className)}>
        <div className="flex flex-wrap items-center gap-2 border-b px-3 py-2.5">
          <div className="mr-auto min-w-0">
            <div className="flex items-center gap-2">
              <ListFilter className="h-4 w-4 text-primary" />
              <h3 className="truncate text-sm font-semibold">
                {title || t('eventList.title')}
              </h3>
              <Badge variant="secondary" className="h-5 px-1.5 text-[10px]">
                {filteredEvents.length}
              </Badge>
            </div>
          </div>

          {showSearch && (
            <div className="relative w-full sm:w-56">
              <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder={t('eventList.searchPlaceholder')}
                aria-label={t('eventList.searchPlaceholder')}
                className="h-8 pl-8 text-xs"
              />
            </div>
          )}

          {showFilters && classOptions.length > 1 && (
            <Select value={classFilter} onValueChange={setClassFilter}>
              <SelectTrigger className="h-8 w-full text-xs sm:w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__all__">{t('eventList.allClasses')}</SelectItem>
                {classOptions.map((option) => (
                  <SelectItem key={option} value={option}>{option}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
        </div>

        {error ? (
          <div className="flex flex-1 items-center justify-center p-6 text-sm text-destructive">
            {error}
          </div>
        ) : paginatedEvents.length === 0 ? (
          <div className="flex flex-1 flex-col items-center justify-center gap-2 p-6 text-center">
            <ListFilter className="h-8 w-8 text-muted-foreground" />
            <p className="text-sm font-medium">{t('eventList.noEvents')}</p>
            <p className="text-xs text-muted-foreground">{t('eventList.noEventsHint')}</p>
          </div>
        ) : (
          <div className="min-h-0 flex-1 overflow-auto">
            {/* Desktop table */}
            <table className="hidden w-full border-collapse text-left text-xs md:table">
              <thead className="sticky top-0 z-10 bg-muted/95 text-muted-foreground backdrop-blur">
                <tr>
                  {showImage && <th className="w-16 px-3 py-2 font-medium">{t('eventList.image')}</th>}
                  <th className="px-3 py-2 font-medium">{t('eventList.time')}</th>
                  <th className="px-3 py-2 font-medium">{t('eventList.vehicle')}</th>
                  <th className="px-3 py-2 font-medium">{t('eventList.color')}</th>
                  <th className="px-3 py-2 font-medium">{t('eventList.direction')}</th>
                  <th className="px-3 py-2 font-medium">{t('eventList.confidence')}</th>
                  <th className="px-3 py-2 font-medium">{t('eventList.attributes')}</th>
                  <th className="w-12 px-3 py-2"><span className="sr-only">{t('eventList.details')}</span></th>
                </tr>
              </thead>
              <tbody>
                {paginatedEvents.map((item) => (
                  <tr
                    key={`${item.id}-${item.timestamp}`}
                    tabIndex={0}
                    className="cursor-pointer border-t transition-colors hover:bg-muted/50 focus:bg-muted/50 focus:outline-none"
                    onClick={() => setSelectedEvent(item)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter' || event.key === ' ') setSelectedEvent(item)
                    }}
                  >
                    {showImage && (
                      <td className="px-3 py-2">
                        <EventThumbnail item={item} className="h-10 w-14" />
                      </td>
                    )}
                    <td className="whitespace-nowrap px-3 py-2 text-muted-foreground">
                      {formatEventTime(item.timestamp)}
                    </td>
                    <td className="px-3 py-2">
                      <div className="font-medium">{item.objectClass}</div>
                      <div className="max-w-40 truncate font-mono text-[10px] text-muted-foreground">{item.id}</div>
                    </td>
                    <td className="px-3 py-2"><Badge variant="outline">{item.color}</Badge></td>
                    <td className="px-3 py-2">{item.direction}</td>
                    <td className="px-3 py-2 tabular-nums">{formatConfidence(item.confidence)}</td>
                    <td className="px-3 py-2">
                      <div className="flex max-w-64 flex-wrap gap-1">
                        {Object.entries(item.attributes).slice(0, 3).map(([key, value]) => (
                          <Badge key={key} variant="secondary" className="max-w-28 truncate font-normal">
                            {key}: {String(value)}
                          </Badge>
                        ))}
                        {Object.keys(item.attributes).length > 3 && (
                          <Badge variant="secondary">+{Object.keys(item.attributes).length - 3}</Badge>
                        )}
                      </div>
                    </td>
                    <td className="px-3 py-2">
                      <IconButton aria-label={t('eventList.details')}>
                        <Eye className="h-4 w-4" />
                      </IconButton>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            {/* Mobile cards */}
            <div className="divide-y md:hidden">
              {paginatedEvents.map((item) => (
                <button
                  key={`${item.id}-${item.timestamp}`}
                  type="button"
                  onClick={() => setSelectedEvent(item)}
                  className="flex w-full gap-3 p-3 text-left transition-colors hover:bg-muted/50"
                >
                  {showImage && <EventThumbnail item={item} className="h-16 w-20 shrink-0" />}
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="truncate text-sm font-medium">{item.objectClass}</span>
                      <Badge variant="outline" className="h-5">{item.color}</Badge>
                    </div>
                    <p className="mt-1 text-xs text-muted-foreground">{formatEventTime(item.timestamp)}</p>
                    <p className="mt-1 truncate text-xs">{item.direction} · {formatConfidence(item.confidence)}</p>
                  </div>
                  <Eye className="mt-1 h-4 w-4 shrink-0 text-muted-foreground" />
                </button>
              ))}
            </div>
          </div>
        )}

        {filteredEvents.length > pageSize && (
          <div className="flex items-center justify-between gap-3 border-t px-3 py-2">
            <span className="text-xs text-muted-foreground">
              {t('eventList.page', { page, total: totalPages })}
            </span>
            <div className="flex gap-1">
              <IconButton
                aria-label={t('eventList.previous')}
                disabled={page <= 1}
                onClick={() => setPage((current) => Math.max(1, current - 1))}
              >
                <ChevronLeft className="h-4 w-4" />
              </IconButton>
              <IconButton
                aria-label={t('eventList.next')}
                disabled={page >= totalPages}
                onClick={() => setPage((current) => Math.min(totalPages, current + 1))}
              >
                <ChevronRight className="h-4 w-4" />
              </IconButton>
            </div>
          </div>
        )}
      </Card>

      <Dialog open={selectedEvent !== null} onOpenChange={(open) => !open && setSelectedEvent(null)}>
        <DialogContent className="max-h-[90vh] max-w-3xl overflow-hidden">
          {selectedEvent && (
              <DialogHeader>
                <DialogTitle>{t('eventList.eventDetails')}</DialogTitle>
                <DialogDescription>
                  {selectedEvent.eventType} · {formatEventTime(selectedEvent.timestamp)}
                </DialogDescription>
              </DialogHeader>
          )}
          {selectedEvent && (
            <div className="min-h-0 space-y-4 overflow-y-auto pr-1">
                <div className="grid gap-4 sm:grid-cols-[220px_1fr]">
                  {showImage && (
                    <EventThumbnail item={selectedEvent} className="h-44 w-full" />
                  )}
                  <dl className="grid grid-cols-[auto_1fr] content-start gap-x-4 gap-y-2 text-sm">
                    <dt className="text-muted-foreground">{t('eventList.eventId')}</dt>
                    <dd className="break-all font-mono text-xs">{selectedEvent.id}</dd>
                    <dt className="text-muted-foreground">{t('eventList.vehicle')}</dt>
                    <dd>{selectedEvent.objectClass}</dd>
                    <dt className="text-muted-foreground">{t('eventList.color')}</dt>
                    <dd>{selectedEvent.color}</dd>
                    <dt className="text-muted-foreground">{t('eventList.direction')}</dt>
                    <dd>{selectedEvent.direction}</dd>
                    <dt className="text-muted-foreground">{t('eventList.tripwire')}</dt>
                    <dd>{selectedEvent.tripwire}</dd>
                    <dt className="text-muted-foreground">{t('eventList.confidence')}</dt>
                    <dd>{formatConfidence(selectedEvent.confidence)}</dd>
                  </dl>
                </div>

                <section>
                  <h4 className="mb-2 text-sm font-semibold">{t('eventList.attributes')}</h4>
                  {Object.keys(selectedEvent.attributes).length > 0 ? (
                    <div className="grid gap-2 sm:grid-cols-2">
                      {Object.entries(selectedEvent.attributes).map(([key, value]) => (
                        <div key={key} className="rounded-md border bg-muted/30 px-3 py-2">
                          <div className="text-[11px] text-muted-foreground">{key}</div>
                          <div className="mt-0.5 break-words text-sm font-medium">{String(value)}</div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className="text-sm text-muted-foreground">{t('eventList.noAttributes')}</p>
                  )}
                </section>

                <details className="rounded-md border">
                  <summary className="cursor-pointer px-3 py-2 text-sm font-medium">
                    {t('eventList.rawData')}
                  </summary>
                  <pre className="max-h-64 overflow-auto border-t bg-muted/40 p-3 text-[11px]">
                    {JSON.stringify({
                      event: selectedEvent.event,
                      attributes: selectedEvent.attributes,
                      crop: selectedEvent.crop
                        ? { ...selectedEvent.crop, image: selectedEvent.image ? '[image data]' : undefined }
                        : undefined,
                    }, null, 2)}
                  </pre>
                </details>
              </div>
          )}
          {selectedEvent && (
              <div className="flex justify-end">
                <Button variant="outline" onClick={() => setSelectedEvent(null)}>
                  {t('eventList.close')}
                </Button>
              </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  )
})
