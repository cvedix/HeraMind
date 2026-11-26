import { useEffect, useState } from 'react'
import { Activity, RefreshCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Link, useMatch } from 'react-router-dom'
import { Bar, BarChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import { Button } from '@/components/ui/button'
import { chartColorsHex } from '@/design-system/tokens/color'
import { ocrApi, type OcrMonitorSnapshot, type OcrRequestRoute, type OcrRequestSample } from '@/lib/api/ocr'
import { cn } from '@/lib/utils'
import { useStore } from '@/store'

const HOUR_MS = 60 * 60 * 1000
const REFRESH_MS = 10_000
const REQUESTS_SHOWN = 50
const JOB_STATES = ['queued', 'running', 'completed', 'failed'] as const
const ROUTES = {
  submit: 'POST /api/ocr/jobs',
  get_job: 'GET /api/ocr/jobs/{id}',
  list_jobs: 'GET /api/ocr/jobs',
  status: 'GET /api/ocr/status',
}
const AUTH_HEADERS = { ocr_key: 'X-OCR-Key', session: 'Authorization', none: '—' }
const ENGINES = { tesseract: 'Tesseract', paddle_ocr_vl: 'PaddleOCR-VL' }
const SELECT = 'h-8 rounded-md border border-input bg-background px-2 text-sm'
const CELL = 'px-3 py-2 text-left whitespace-nowrap'

type RequestFilters = { hours: number; route: OcrRequestRoute | 'all'; status: string }

export function OcrMonitorWidget({ title, className }: { title?: string; className?: string }) {
  const { t } = useTranslation('common')
  const instanceId = useStore(state => state.currentInstanceId)
  const isAdmin = useStore(state => state.user?.role === 'admin')
  const isShared = useMatch('/share/:token') !== null

  if (!isAdmin || isShared) {
    return <p className="p-4 text-sm text-muted-foreground">{t('common:ocr.monitor.adminOnly')}</p>
  }

  return <OcrMonitor key={instanceId} title={title || 'OCR Monitor'} className={className} />
}

function OcrMonitor({ title, className }: { title: string; className?: string }) {
  const { t, i18n } = useTranslation('common')
  const label = (key: string) => t(`ocr.monitor.${key}`)
  const [view, setView] = useState<'requests' | 'jobs'>('requests')
  const [filters, setFilters] = useState<RequestFilters>({ hours: 1, route: 'all', status: 'all' })
  const [snapshot, setSnapshot] = useState<OcrMonitorSnapshot>()
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(true)
  const [refresh, setRefresh] = useState(0)
  const date = new Intl.DateTimeFormat(i18n.language, { dateStyle: 'short', timeStyle: 'medium' })
  const time = new Intl.DateTimeFormat(i18n.language, { hour: '2-digit', minute: '2-digit' })
  const number = new Intl.NumberFormat(i18n.language)

  useEffect(() => {
    const controller = new AbortController()
    let timer: ReturnType<typeof setTimeout>

    async function read() {
      try {
        if (document.visibilityState === 'hidden') return
        const current = await ocrApi.monitor(controller.signal)
        if (!controller.signal.aborted) {
          setSnapshot(current)
          setError('')
        }
      } catch (cause) {
        if (!controller.signal.aborted) setError(cause instanceof Error ? cause.message : String(cause))
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false)
          timer = setTimeout(read, REFRESH_MS)
        }
      }
    }

    void read()
    return () => {
      controller.abort()
      clearTimeout(timer)
    }
  }, [refresh])

  return (
    <div className={cn('min-w-0 overflow-y-auto rounded-xl border border-border bg-card p-4 sm:p-5', className)}>
      <header className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="flex items-center gap-2 text-lg font-semibold"><Activity className="h-5 w-5 text-primary" aria-hidden="true" />{title}</h2>
        <div className="flex flex-wrap items-center gap-3">
          {snapshot && <span className="text-xs text-muted-foreground">{t('common:ocr.monitor.updated', { time: date.format(snapshot.generated_at_ms) })}</span>}
          <Button size="sm" variant="outline" className="gap-2" onClick={() => setRefresh(current => current + 1)}>
            <RefreshCw className="h-4 w-4" aria-hidden="true" />{label('refresh')}
          </Button>
        </div>
      </header>
      {error && <p role="alert" className="mt-3 text-sm text-destructive">{t('common:ocr.monitor.fetchError', { error })}</p>}
      {!snapshot && loading && <p role="status" className="mt-3 text-sm text-muted-foreground">{label('loading')}</p>}
      {snapshot && (
        <>
          <div className="mb-4 mt-4 flex flex-wrap gap-2 border-b border-border pb-3" aria-label={label('views')}>
            <Button size="sm" variant={view === 'requests' ? 'secondary' : 'ghost'} aria-pressed={view === 'requests'} onClick={() => setView('requests')}>
              {label('requests')}
            </Button>
            <Button size="sm" variant={view === 'jobs' ? 'secondary' : 'ghost'} aria-pressed={view === 'jobs'} onClick={() => setView('jobs')}>
              {label('jobs')}
            </Button>
          </div>
          {view === 'requests' ? (
            <RequestView data={snapshot} filters={filters} setFilters={setFilters} label={label} date={date} time={time} number={number} />
          ) : (
            <JobView jobs={snapshot.jobs} label={label} date={date} />
          )}
        </>
      )}
    </div>
  )
}

function Metrics({ items }: { items: { label: string; value: React.ReactNode; hint?: string }[] }) {
  return (
    <dl className="grid grid-cols-2 gap-3 sm:grid-cols-4">
      {items.map(item => (
        <div key={item.label} title={item.hint} className="rounded-lg border border-border bg-muted/20 p-3">
          <dt className="text-xs text-muted-foreground">{item.label}</dt>
          <dd className="mt-1 text-xl font-semibold tabular-nums">{item.value}</dd>
        </div>
      ))}
    </dl>
  )
}

function requestVolume(rows: OcrRequestSample[], start: number, end: number, coverage: number) {
  const width = (end - start) / 12
  const bars = Array.from({ length: 12 }, (_, index) => ({
    time: start + index * width,
    regular: 0,
    error: 0,
  }))
  for (const row of rows) {
    const bucket = bars[Math.min(11, Math.floor((row.at_ms - start) / width))]
    if (row.status >= 400) bucket.error += 1
    else bucket.regular += 1
  }
  return bars
    .filter(bar => bar.time + width > coverage || bar.regular > 0 || bar.error > 0)
    .map(bar => ({ ...bar, time: Math.max(bar.time, coverage) }))
}

function RequestView({ data, filters, setFilters, label, date, time, number }: {
  data: OcrMonitorSnapshot
  filters: RequestFilters
  setFilters: (filters: RequestFilters) => void
  label: (key: string) => string
  date: Intl.DateTimeFormat
  time: Intl.DateTimeFormat
  number: Intl.NumberFormat
}) {
  const { t } = useTranslation('common')
  const { hours, route, status } = filters
  const start = data.generated_at_ms - hours * HOUR_MS
  const rows = data.requests.filter(row => row.at_ms >= start && (route === 'all' || route === row.route) &&
    (status === 'all' || Math.floor(row.status / 100).toString() === status))
  const errors = rows.filter(row => row.status >= 400).length
  const accepted = rows.filter(row => row.route === 'submit' && row.status === 202).length
  const durations = rows.map(row => row.duration_ms).sort((a, b) => a - b)
  const p95 = durations[Math.ceil(durations.length * 0.95) - 1]

  return (
    <section className="space-y-4">
      <div className="flex flex-wrap gap-3">
        <label className="flex items-center gap-2 text-sm">{label('period')}
          <select className={SELECT} value={hours} onChange={event => setFilters({ ...filters, hours: Number(event.target.value) })}>
            <option value={1}>{label('oneHour')}</option>
            <option value={24}>{label('day')}</option>
          </select>
        </label>
        <label className="flex items-center gap-2 text-sm">{label('endpoint')}
          <select className={cn(SELECT, 'max-w-64')} value={route} onChange={event => setFilters({ ...filters, route: event.target.value as RequestFilters['route'] })}>
            <option value="all">{label('all')}</option>
            {Object.entries(ROUTES).map(([key, path]) => <option key={key} value={key}>{path}</option>)}
          </select>
        </label>
        <label className="flex items-center gap-2 text-sm">HTTP
          <select className={SELECT} value={status} onChange={event => setFilters({ ...filters, status: event.target.value })}>
            <option value="all">{label('all')}</option>
            {['2', '3', '4', '5'].map(group => <option key={group} value={group}>{group}xx</option>)}
          </select>
        </label>
      </div>
      <Metrics items={[
        { label: label('total'), value: number.format(rows.length) },
        { label: label('accepted'), value: number.format(accepted) },
        { label: label('errors'), value: number.format(errors) },
        { label: label('latency'), value: p95 == null ? '—' : `${number.format(p95)} ms`, hint: label('latencyHint') },
      ]} />
      <p className="text-xs text-muted-foreground">
        {t('common:ocr.monitor.coverage', { time: date.format(Math.max(start, data.observed_since_ms)), count: number.format(data.max_requests) })}
      </p>
      {rows.length === 0 ? <p className="text-sm text-muted-foreground">{label('emptyRequests')}</p> : (
        <>
          <figure className="rounded-lg border border-border px-2 py-3 sm:px-4">
            <figcaption className="mb-2 px-2 text-sm font-medium">{label('volume')}</figcaption>
            <div className="h-40 min-w-0">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={requestVolume(rows, start, data.generated_at_ms, data.observed_since_ms)} maxBarSize={56} accessibilityLayer>
                  <XAxis dataKey="time" tickFormatter={value => time.format(value)} tick={{ fontSize: 11 }} minTickGap={24} />
                  <YAxis allowDecimals={false} width={30} tick={{ fontSize: 11 }} />
                  <Tooltip cursor={false} labelFormatter={value => date.format(Number(value))} contentStyle={{ color: 'var(--foreground)', background: 'var(--card)' }} />
                  <Bar dataKey="regular" name="HTTP <400" stackId="requests" fill={chartColorsHex[0]} isAnimationActive={false} />
                  <Bar dataKey="error" name="HTTP 4xx/5xx" stackId="requests" fill="#c2410c" isAnimationActive={false} />
                </BarChart>
              </ResponsiveContainer>
            </div>
            <div className="flex flex-wrap gap-4 px-2 text-xs text-muted-foreground">
              <span className="flex items-center gap-1.5"><i aria-hidden="true" className="size-2 rounded-sm bg-[#005bea]" />HTTP &lt;400</span>
              <span className="flex items-center gap-1.5"><i aria-hidden="true" className="size-2 rounded-sm bg-[#c2410c]" />HTTP 4xx/5xx</span>
            </div>
          </figure>
          <div className="overflow-x-auto rounded-lg border border-border">
            <table className="w-full text-sm">
              <caption className="px-3 py-2 text-left text-xs text-muted-foreground">{t('common:ocr.monitor.requestRows', { count: Math.min(rows.length, REQUESTS_SHOWN) })}</caption>
              <thead className="bg-muted/40 text-xs text-muted-foreground">
                <tr>{[label('time'), label('endpoint'), 'HTTP', label('duration'), label('authHeader'), label('jobId')].map(column => (
                  <th key={column} scope="col" className={CELL}>{column}</th>
                ))}</tr>
              </thead>
              <tbody className="divide-y divide-border">
                {rows.slice(0, REQUESTS_SHOWN).map(row => (
                  <tr key={row.sequence}>
                    <td className={CELL}>{date.format(row.at_ms)}</td>
                    <td className={cn(CELL, 'font-mono text-xs')}>{ROUTES[row.route]}</td>
                    <td className={cn(CELL, 'font-semibold tabular-nums', row.status >= 400 && 'text-destructive')}>{row.status}</td>
                    <td className={CELL}>{number.format(row.duration_ms)} ms</td>
                    <td className={CELL}>{AUTH_HEADERS[row.authentication]}</td>
                    <td className={cn(CELL, 'font-mono text-xs')}>{row.job_id ?? '—'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
    </section>
  )
}

function JobView({ jobs, label, date }: {
  jobs: OcrMonitorSnapshot['jobs']
  label: (key: string) => string
  date: Intl.DateTimeFormat
}) {
  const { t } = useTranslation('common')

  return (
    <section className="space-y-4">
      <Metrics items={JOB_STATES.map(state => ({ label: t(`ocr.${state}`), value: jobs.filter(job => job.status === state).length }))} />
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="text-xs text-muted-foreground">{t('common:ocr.monitor.jobRows', { count: jobs.length })}</p>
        <Link className="text-sm underline underline-offset-4" to="/ocr">{label('openOcr')}</Link>
      </div>
      {jobs.length === 0 ? <p className="text-sm text-muted-foreground">{t('common:ocr.noJobs')}</p> : (
        <div className="overflow-x-auto rounded-lg border border-border">
          <table className="w-full text-sm">
            <thead className="bg-muted/40 text-xs text-muted-foreground">
              <tr>{[label('received'), label('jobId'), t('common:ocr.engine'), label('jobStatus'), label('pages'), label('processingTime'), label('errorCode')].map(column => (
                <th key={column} scope="col" className={CELL}>{column}</th>
              ))}</tr>
            </thead>
            <tbody className="divide-y divide-border">
              {jobs.map(job => (
                <tr key={job.id}>
                  <td className={CELL}>{date.format(job.created_at * 1000)}</td>
                  <td className={cn(CELL, 'font-mono text-xs')}>{job.id}</td>
                  <td className={CELL}>{ENGINES[job.engine]}</td>
                  <td className={cn(CELL, job.status === 'failed' && 'text-destructive')}>{t(`ocr.${job.status}`)}</td>
                  <td className={CELL}>{job.page_count == null ? '—' : `${job.pages_done}/${job.page_count}`}</td>
                  <td className={CELL}>{job.finished_at == null ? '—' : `${job.finished_at - job.created_at} s`}</td>
                  <td className={cn(CELL, 'font-mono text-xs')}>{job.error_code ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  )
}
