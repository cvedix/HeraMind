import { useEffect, useId, useState, type FormEvent } from 'react'
import { Download, FileScan, Loader2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { MobilePageHeader } from '@/components/layout/MobilePageHeader'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { confirm } from '@/components/ui/use-confirm'
import { getServerOrigin } from '@/lib/api/client'
import { ocrApi, type OcrEngine, type OcrJob, type OcrKey, type OcrLanguage, type OcrResult, type OcrStatus } from '@/lib/api/ocr'
import { cn } from '@/lib/utils'
import { useStore } from '@/store'

const panel = 'min-w-0 rounded-xl border border-border bg-card p-4 sm:p-5'
const active = (job?: OcrJob) => job?.status === 'queued' || job?.status === 'running'
const errorMessage = (error: unknown) => error instanceof Error ? error.message : String(error)
const engines = { tesseract: 'Tesseract', paddle_ocr_vl: 'PaddleOCR-VL' }

export default function OcrServicePage({ initialEngine = 'tesseract' }: { initialEngine?: OcrEngine }) {
  const instanceId = useStore((state) => state.currentInstanceId)
  return <OcrDashboard key={`${instanceId}:${initialEngine}`} initialEngine={initialEngine} />
}

function OcrDashboard({ initialEngine }: { initialEngine: OcrEngine }) {
  const { t, i18n } = useTranslation('common')
  const fieldId = useId()
  const label = (key: string) => t(`ocr.${key}`)
  const isAdmin = useStore((state) => state.user?.role === 'admin')
  const [engine, setEngine] = useState(initialEngine)
  const [engineStatus, setEngineStatus] = useState<OcrStatus>()
  const status = engineStatus?.engine === engine ? engineStatus : undefined
  const [jobs, setJobs] = useState<OcrJob[]>([])
  const [selected, setSelected] = useState<string>()
  const selectedJob = jobs.find((job) => job.id === selected)
  const selectedState = selectedJob?.status
  const [result, setResult] = useState<OcrResult>()
  const [page, setPage] = useState(0)
  const [file, setFile] = useState<File>()
  const [language, setLanguage] = useState<OcrLanguage>('vie_eng')
  const [uploading, setUploading] = useState(false)
  const [loadError, setLoadError] = useState('')
  const [resultError, setResultError] = useState('')
  const [uploadError, setUploadError] = useState('')
  const [statusError, setStatusError] = useState('')
  const [keys, setKeys] = useState<OcrKey[]>([])
  const [keyName, setKeyName] = useState('')
  const [keyBusy, setKeyBusy] = useState(false)
  const [keyError, setKeyError] = useState('')
  const [issuedToken, setIssuedToken] = useState('')
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const controller = new AbortController()
    let timer: ReturnType<typeof setTimeout>
    const refresh = async () => {
      try {
        const entries = await ocrApi.jobs(controller.signal)
        if (!controller.signal.aborted) {
          setJobs(entries)
          setLoadError('')
          setLoading(false)
        }
      } catch (error) {
        if (!controller.signal.aborted) {
          setLoadError(errorMessage(error))
          setLoading(false)
        }
      } finally {
        if (!controller.signal.aborted) timer = setTimeout(refresh, 2500)
      }
    }
    void refresh()
    return () => { controller.abort(); clearTimeout(timer) }
  }, [])

  useEffect(() => {
    const controller = new AbortController()
    let timer: ReturnType<typeof setTimeout>
    setStatusError('')
    const refresh = async () => {
      try {
        const value = await ocrApi.status(engine, controller.signal)
        if (!controller.signal.aborted) {
          setEngineStatus(value)
          setStatusError('')
          setLanguage(current => value.languages.includes(current) ? current : value.languages[0] ?? 'vie_eng')
        }
      } catch (error) {
        if (!controller.signal.aborted) setStatusError(errorMessage(error))
      } finally {
        if (!controller.signal.aborted) timer = setTimeout(refresh, 10000)
      }
    }
    void refresh()
    return () => { controller.abort(); clearTimeout(timer) }
  }, [engine])

  useEffect(() => {
    if (!selected) return
    const controller = new AbortController()
    void ocrApi.result(selected, controller.signal).then((value) => {
      if (!controller.signal.aborted) {
        setResult(value)
        setResultError('')
      }
    }).catch((error) => {
      if (!controller.signal.aborted) setResultError(errorMessage(error))
    })
    return () => controller.abort()
  }, [selected, selectedState])

  useEffect(() => {
    if (!isAdmin) return
    const controller = new AbortController()
    void ocrApi.keys(controller.signal).then(setKeys).catch((error) => {
      if (!controller.signal.aborted) setKeyError(errorMessage(error))
    })
    return () => controller.abort()
  }, [isAdmin])

  function selectJob(id: string) {
    if (id === selected) return
    setResult(undefined)
    setResultError('')
    setPage(0)
    setSelected(id)
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!file || !status?.engine_ready || uploading) return
    if (file.size > status.max_bytes) {
      setUploadError(label('tooLarge'))
      return
    }
    setUploadError('')
    setUploading(true)
    try {
      const job = await ocrApi.submit(file, language, engine)
      setJobs((current) => [job, ...current.filter((item) => item.id !== job.id)])
      selectJob(job.id)
    } catch (error) {
      setUploadError(errorMessage(error))
    } finally {
      setUploading(false)
    }
  }

  async function issueKey(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!keyName.trim() || keyBusy) return
    setKeyBusy(true)
    setKeyError('')
    setIssuedToken('')
    try {
      const issued = await ocrApi.issue(keyName.trim())
      setKeys((current) => [issued.key, ...current])
      setIssuedToken(issued.token)
      setKeyName('')
    } catch (error) {
      setKeyError(errorMessage(error))
    } finally {
      setKeyBusy(false)
    }
  }

  async function revokeKey(key: OcrKey) {
    if (keyBusy || !(await confirm({ title: label('revoke'), description: t('ocr.revokePrompt', { name: key.name }) }))) return
    setKeyBusy(true)
    setKeyError('')
    setIssuedToken('')
    try {
      await ocrApi.revoke(key.id)
      setKeys((current) => current.filter((item) => item.id !== key.id))
    } catch (error) {
      setKeyError(errorMessage(error))
    } finally {
      setKeyBusy(false)
    }
  }

  function download() {
    if (!result?.pages.length) return
    const url = URL.createObjectURL(new Blob([result.pages.map((item) => item.text).join('\n\n\f\n\n')], { type: 'text/plain;charset=utf-8' }))
    const link = document.createElement('a')
    link.href = url
    link.download = result.job.filename.replace(/\.pdf$/i, '') + '.txt'
    link.click()
    setTimeout(() => URL.revokeObjectURL(url), 1000)
  }

  const currentPage = result?.pages[page]
  const job = selectedJob ?? result?.job
  const title = initialEngine === 'paddle_ocr_vl' ? 'OCR Service · PaddleOCR-VL' : label('title')
  const origin = getServerOrigin()
  const keyHeader = '-H "X-OCR-Key: $OCR_KEY"'
  const apiExamples = [
    { step: 'apiSetup', hint: 'apiSetupHint', command: [
      `HERA_BASE='${origin.replace(/'/g, "'\\''")}'`,
      "read -rsp 'OCR_KEY: ' OCR_KEY",
      "printf '\\n'",
    ].join('\n') },
    { step: 'apiStatus', hint: 'apiStatusHint', command: [
      `curl -fsS "$HERA_BASE/api/ocr/status?engine=${engine}" \\`,
      `  ${keyHeader}`,
    ].join('\n') },
    { step: 'apiSubmit', hint: 'apiSubmitHint', command: [
      'curl -sS -i -X POST \\',
      `  "$HERA_BASE/api/ocr/jobs?engine=${engine}&language=${language}&filename=document.pdf" \\`,
      `  ${keyHeader} \\`,
      "  -H 'Content-Type: application/pdf' \\",
      '  --data-binary @document.pdf',
    ].join('\n') },
    { step: 'apiResult', hint: 'apiResultHint', command: [
      "JOB_ID='your-job-id'",
      'curl -fsS "$HERA_BASE/api/ocr/jobs/$JOB_ID" \\',
      `  ${keyHeader}`,
    ].join('\n') },
  ]

  return (
    <div className="h-full overflow-y-auto bg-background">
      <MobilePageHeader title={title} />
      <div className="mx-auto max-w-7xl space-y-5 px-4 pb-28 pt-5 sm:px-6 md:pb-8 md:pt-16">
        <header className="flex flex-wrap items-end justify-between gap-4 border-b border-border pb-5">
          <div className="min-w-0">
            <h1 className="hidden text-3xl font-bold tracking-tight md:block">{title}</h1>
            <p className="mt-1 text-sm text-muted-foreground">{label('description')}</p>
          </div>
          <span className={cn('rounded-full px-3 py-1 text-xs font-medium', status?.engine_ready ? 'bg-success-light text-success' : 'bg-muted text-muted-foreground')}>
            {label(status ? status.engine_ready ? 'ready' : 'unavailable' : statusError ? 'unavailable' : 'checking')}
          </span>
        </header>

        <div className="grid items-start gap-5 lg:grid-cols-[1fr_1.1fr]">
          <section className={panel}>
            <h2 className="mb-4 flex items-center gap-2 font-semibold"><FileScan className="h-4 w-4 text-brand" />{label('upload')}</h2>
            <form onSubmit={submit} className="space-y-4">
              <div>
                <label htmlFor={`${fieldId}-file`} className="mb-1.5 block text-sm font-medium">{label('file')}</label>
                <Input id={`${fieldId}-file`} type="file" accept=".pdf,application/pdf" onChange={(event) => {
                  setFile(event.target.files?.[0])
                  setUploadError('')
                }} />
              </div>
              <div>
                <label htmlFor={`${fieldId}-engine`} className="mb-1.5 block text-sm font-medium">{label('engine')}</label>
                <select id={`${fieldId}-engine`} className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
                  value={engine} disabled={uploading} onChange={(event) => {
                    setEngine(event.target.value as OcrEngine)
                    setUploadError('')
                  }}>
                  <option value="tesseract">{engines.tesseract}</option>
                  <option value="paddle_ocr_vl">{engines.paddle_ocr_vl}</option>
                </select>
                {engine === 'paddle_ocr_vl' && status && <p className="mt-1.5 text-xs text-muted-foreground">
                  {status.engine_ready
                    ? status.model_loaded ? t('ocr.modelLoaded', { version: status.model_version ?? '' }) : label('modelCold')
                    : label('paddleUnavailable')}
                </p>}
              </div>
              <div>
                <label htmlFor={`${fieldId}-language`} className="mb-1.5 block text-sm font-medium">{label('language')}</label>
                <select id={`${fieldId}-language`} className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
                  value={language} onChange={(event) => setLanguage(event.target.value as OcrLanguage)}>
                  {(status?.languages.length ? status.languages : ['vie_eng'] as OcrLanguage[]).map((value) =>
                    <option value={value} key={value}>{label(`lang_${value}`)}</option>)}
                </select>
              </div>
              {status && <p className="text-xs text-muted-foreground">{t('ocr.limits', {
                mb: status.max_bytes / 1024 / 1024, pages: status.max_pages, hours: status.retention_seconds / 3600,
              })}</p>}
              {(uploadError || statusError) && <p role="alert" className="text-sm text-destructive">{uploadError || statusError}</p>}
              <Button type="submit" disabled={!file || !status?.engine_ready || uploading}>
                {uploading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}{label('submit')}
              </Button>
            </form>
          </section>

          <section className={panel}>
            <h2 className="mb-3 font-semibold">{label('integration')}</h2>
            <p className="mb-3 text-sm text-muted-foreground">{label('integrationHint')}</p>
            {isAdmin && (
              <details className="my-4 rounded-md border border-border p-3">
                <summary className="cursor-pointer text-sm font-medium">{label('keys')}</summary>
                <form onSubmit={issueKey} className="mt-3 flex flex-wrap gap-2">
                  <Input className="min-w-32 flex-1" aria-label={label('keyName')} placeholder={label('keyName')}
                    maxLength={80} value={keyName} onChange={(event) => setKeyName(event.target.value)} />
                  <Button type="submit" variant="outline" disabled={!keyName.trim() || keyBusy}>{label('issue')}</Button>
                </form>
                {issuedToken && <div className="mt-3 space-y-2">
                  <p className="text-sm font-medium">{label('keyOnce')}</p>
                  <Input readOnly aria-label={label('token')} value={issuedToken} onFocus={(event) => event.target.select()} />
                  <Button size="sm" variant="ghost" onClick={() => setIssuedToken('')}>{label('dismiss')}</Button>
                </div>}
                {keyError && <p role="alert" className="mt-2 text-sm text-destructive">{keyError}</p>}
                <ul className="mt-3 divide-y divide-border">
                  {keys.map((key) => <li className="flex items-center justify-between gap-2 py-1" key={key.id}>
                    <span className="truncate text-sm">{key.name}</span>
                    <Button variant="ghost" size="sm" disabled={keyBusy} onClick={() => void revokeKey(key)}>{label('revoke')}</Button>
                  </li>)}
                </ul>
              </details>
            )}
            <ol className="list-decimal space-y-4 pl-5 text-sm">
              {apiExamples.map(({ step, hint, command }) => (
                <li key={step}>
                  <h3 className="font-medium">{label(step)}</h3>
                  <p className="mb-2 text-muted-foreground">{label(hint)}</p>
                  <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs"><code>{command}</code></pre>
                </li>
              ))}
            </ol>
            <p className="mt-4 text-sm text-muted-foreground">{label('apiErrors')}</p>
            <a className="mt-2 inline-block text-sm underline underline-offset-4" href={`${origin}/api/docs/openapi.json`} target="_blank" rel="noreferrer">
              {label('apiSchema')}
            </a>
          </section>
        </div>

        <div className="grid gap-5 lg:grid-cols-[minmax(0,0.85fr)_minmax(0,1.15fr)]">
          <section className={panel}>
            <h2 className="font-semibold">{label('history')}</h2>
            {loadError && <p role="alert" className="mt-2 text-sm text-destructive">{loadError}</p>}
            {!jobs.length && !loadError && <p className="mt-3 text-sm text-muted-foreground">{label(loading ? 'loading' : 'noJobs')}</p>}
            <ul className="mt-2 max-h-96 divide-y divide-border overflow-y-auto">
              {jobs.map((entry) => <li key={entry.id}>
                <button type="button" aria-pressed={selected === entry.id} onClick={() => selectJob(entry.id)}
                  className={cn('w-full rounded-lg p-3 text-left hover:bg-muted/50', selected === entry.id && 'bg-brand/10')}>
                  <span className="block truncate text-sm font-medium">{entry.filename}</span>
                  <span className="mt-1 flex flex-wrap gap-x-2 text-xs text-muted-foreground">
                    <span>{label(entry.status)}</span>
                    <span>{engines[entry.engine]}</span>
                    {entry.page_count != null && <span>{entry.pages_done}/{entry.page_count}</span>}
                    <span>{new Date(entry.created_at * 1000).toLocaleString(i18n.language)}</span>
                  </span>
                </button>
              </li>)}
            </ul>
          </section>

          <section className={panel}>
            <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
              <h2 className="font-semibold">{label('result')}</h2>
              {result?.pages.length ? <Button variant="outline" size="sm" onClick={download}><Download className="mr-2 h-4 w-4" />{label('download')}</Button> : null}
            </div>
            {resultError && <p role="alert" className="text-sm text-destructive">{resultError}</p>}
            {!selected && <p className="text-sm text-muted-foreground">{label('selectJob')}</p>}
            {job && (
              <div className="space-y-3">
                <p className="break-all text-sm font-medium">{job.filename}</p>
                <p aria-live="polite" className="text-sm text-muted-foreground">{engines[job.engine]} · {label(job.status)}{job.page_count ? ` · ${job.pages_done}/${job.page_count}` : ''}</p>
                <p className="break-all font-mono text-xs text-muted-foreground">{job.id}</p>
                {job.status === 'failed' && job.error_code && <p role="alert" className="text-sm text-destructive">
                  {t(`ocr.failures.${job.error_code}`, { defaultValue: job.error_code })}
                </p>}
                {active(job) && job.page_count && <progress className="w-full accent-blue-600" aria-label={label('progress')} max={job.page_count} value={job.pages_done} />}
                {currentPage && result && (
                  <>
                    <select aria-label={label('page')} className="h-9 rounded-md border border-input bg-background px-2 text-sm"
                      value={page} onChange={(event) => setPage(Number(event.target.value))}>
                      {result.pages.map((item, index) => <option key={item.number} value={index}>{t('ocr.pageNumber', { number: item.number })}</option>)}
                    </select>
                    <pre className="max-h-96 overflow-y-auto whitespace-pre-wrap break-words rounded-md bg-muted p-3 text-sm">{currentPage.text || label('emptyPage')}</pre>
                  </>
                )}
              </div>
            )}
          </section>
        </div>
      </div>
    </div>
  )
}
