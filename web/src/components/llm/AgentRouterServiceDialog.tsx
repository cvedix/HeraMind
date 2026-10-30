import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Download, Loader2, Play, Square, Server } from 'lucide-react'
import { api } from '@/lib/api'
import type { AgentRouterServiceStatus } from '@/types'
import { UnifiedFormDialog } from '@/components/dialog/UnifiedFormDialog'
import { FormField } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { PasswordInput } from '@/components/ui/password-input'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { BackendConnectionTest } from './BackendConnectionTest'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  onDirect: () => void
  onConfigured?: () => Promise<void>
  initialName?: string
  editingId?: string
}

export function AgentRouterServiceDialog({ open, onOpenChange, onDirect, onConfigured, initialName, editingId }: Props) {
  const { t } = useTranslation('plugins')
  const translations = useRef(t)
  translations.current = t
  const [status, setStatus] = useState<AgentRouterServiceStatus | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [statusError, setStatusError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [name, setName] = useState(initialName || 'AgentRouter · OpenClaw')
  const [model, setModel] = useState('')
  const [protocol, setProtocol] = useState('openai-responses')
  const [key, setKey] = useState('')

  const errorMessage = (error: unknown) => (error as { status?: number })?.status === 403
    ? t('llm.serviceAdminOnly') : error instanceof Error ? error.message : String(error)

  useEffect(() => {
    if (!open) return
    let alive = true
    let timer: ReturnType<typeof setTimeout>
    let initialized = false
    let retry = true
    setStatus(null)
    setError(null)
    setStatusError(null)
    setKey('')
    setName(initialName || 'AgentRouter · OpenClaw')
    const poll = async () => {
      try {
        const next = await api.agentRouterServiceStatus()
        if (!alive) return
        setStatus(next)
        setStatusError(null)
        if (!initialized) {
          setModel(next.model)
          setProtocol(next.protocol)
          initialized = true
        }
      } catch (error) {
        retry = (error as { status?: number })?.status !== 403
        if (alive) setStatusError((error as { status?: number })?.status === 403
          ? translations.current('llm.serviceAdminOnly') : error instanceof Error ? error.message : String(error))
      } finally {
        if (alive && retry) timer = setTimeout(() => void poll(), 2000)
      }
    }
    void poll()
    return () => { alive = false; clearTimeout(timer) }
  }, [open, initialName])

  const action = async (run: () => Promise<unknown>) => {
    setBusy(true)
    setError(null)
    try {
      await run()
      setStatus(await api.agentRouterServiceStatus())
    } catch (error) { setError(errorMessage(error)) }
    finally { setBusy(false) }
  }

  const save = async () => {
    setBusy(true)
    setError(null)
    try {
      await api.configureAgentRouterService({
        name: name.trim(), model: model.trim(), protocol,
        ...(key.trim() ? { api_key: key.trim() } : {}),
        ...((editingId || status?.backend_id) ? { backend_id: editingId || status!.backend_id! } : {}),
      })
      setKey('')
      await onConfigured?.()
      onOpenChange(false)
    } catch (error) { setError(errorMessage(error)) }
    finally { setBusy(false) }
  }

  const installing = status?.progress.busy ?? false
  const external = !!status?.running && !status.managed
  const blocked = busy || installing
  const unchanged = model === status?.model && protocol === status?.protocol && !key.trim()
  return (
    <UnifiedFormDialog open={open} onOpenChange={onOpenChange}
      title={t('llm.serviceTitle')} description={t('llm.serviceDescription')}
      icon={<Server className="h-5 w-5" />} className="z-[110]"
      loading={!status && !error && !statusError} isSubmitting={busy} onSubmit={save}
      submitLabel={t('llm.serviceConnect')}
      submitDisabled={!status?.installed || !status.supported || blocked || external || !name.trim() || !model.trim() || (!key.trim() && !status.configured)}
      submitError={error || statusError || status?.progress.error}>
      <div className="space-y-4">
        <p className="text-sm text-muted-foreground">{t('llm.serviceShared')}</p>
        {status && (
          <div className="space-y-3 rounded-lg border p-3">
            <p role="status" className="text-sm font-medium">
              {installing ? <Loader2 className="mr-2 inline h-4 w-4 animate-spin" /> : null}
              {t(installing ? `llm.servicePhase.${status.progress.phase || 'queued'}` : status.running ? 'llm.serviceRunning' : status.installed ? 'llm.serviceInstalled' : 'llm.serviceMissing')}
            </p>
            <p className="text-xs text-muted-foreground">OpenClaw {status.openclaw_version} · Node.js {status.node_version}</p>
            {!status.supported && <p role="alert">{t('llm.serviceUnsupported')}</p>}
            {external && <p role="alert" className="text-sm text-muted-foreground">{t('llm.serviceExternal')}</p>}
            <div className="flex flex-wrap gap-2">
              <Button type="button" variant="secondary" disabled={blocked || !status.supported || status.installed}
                onClick={() => void action(api.installAgentRouterService)}>
                <Download className="mr-2 h-4 w-4" />{t('llm.serviceInstall')}
              </Button>
              {status.installed && !status.running && <Button type="button" variant="secondary" disabled={blocked || !status.configured}
                onClick={() => void action(api.startAgentRouterService)}><Play className="mr-2 h-4 w-4" />{t('llm.serviceStart')}</Button>}
              {status.running && status.managed && <Button type="button" variant="secondary" disabled={blocked}
                onClick={() => void action(api.stopAgentRouterService)}><Square className="mr-2 h-4 w-4" />{t('llm.serviceStop')}</Button>}
            </div>
            <p className="text-xs text-muted-foreground">{t('llm.serviceInstallHelp')}</p>
          </div>
        )}
        <FormField label={t('llm.name')}><Input value={name} onChange={e => setName(e.target.value)} disabled={blocked} /></FormField>
        <FormField label={t('llm.model')} helpText={t('llm.serviceModelHelp')}>
          <Input value={model} aria-label={t('llm.model')} onChange={e => setModel(e.target.value)} placeholder="gpt-6.1-sol" disabled={blocked} />
        </FormField>
        <FormField label={t('llm.cloudProtocol')}>
          <Select value={protocol} onValueChange={setProtocol} disabled={blocked}>
            <SelectTrigger aria-label={t('llm.cloudProtocol')}><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="openai-responses">OpenAI Responses API</SelectItem>
              <SelectItem value="openai-completions">OpenAI Chat Completions</SelectItem>
              <SelectItem value="anthropic-messages">Anthropic Messages</SelectItem>
            </SelectContent>
          </Select>
        </FormField>
        <FormField label={t('llm.apiKey')} helpText={status?.configured ? t('llm.serviceKeyKeep') : t('llm.apiKeyHelp')}>
          <PasswordInput value={key} aria-label={t('llm.apiKey')} onChange={e => setKey(e.target.value)} disabled={blocked} />
        </FormField>
        {status?.running && status.backend_id && unchanged && !blocked && (
          <BackendConnectionTest onTest={async () => (await api.testLlmBackend(status.backend_id!)).result} />
        )}
        <Button type="button" variant="ghost" disabled={blocked} onClick={onDirect}>{t('llm.serviceDirect')}</Button>
      </div>
    </UnifiedFormDialog>
  )
}
