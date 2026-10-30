/**
 * CloudAiAddDialog — the add/edit dialog for the Cloud AI card.
 *
 * One card, two protocol paths: the user picks the protocol (OpenAI-
 * compatible / Anthropic) inside this dialog, then fills the standard
 * fields. `backend_type` is mapped from the selection; capabilities come
 * from the concrete type (registry + runtime probe) as usual.
 *
 * Edit mode: pass `editing`. The protocol select doubles as the type
 * switcher — changing it PUTs a different backend_type, and the server
 * re-bases capabilities (the endpoint/model fields are kept, so switching
 * protocols usually also means editing the endpoint). The API key field
 * prefills with the API_KEY_MASK sentinel when a key is stored; both the
 * untouched mask and an empty value mean "keep the existing key".
 */

import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Cloud, Loader2 } from 'lucide-react'
import { UnifiedFormDialog } from '@/components/dialog/UnifiedFormDialog'
import { FormField } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { PasswordInput } from '@/components/ui/password-input'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { API_KEY_MASK } from '@/components/plugins/UniversalPluginConfigDialog'
import type { CreateLlmBackendRequest } from '@/types'
import { api } from '@/lib/api'
import { AgentRouterServiceDialog } from './AgentRouterServiceDialog'

type CloudProtocol = 'openai' | 'anthropic'
type CloudProvider = 'custom' | 'agentrouter'

function isAgentRouterEndpoint(endpoint: string): boolean {
  return /^https:\/\/agentrouter\.org(?::443)?(?:\/v1)?\/?$/.test(endpoint)
}

function isAgentRouterGateway(endpoint: string): boolean {
  return /^http:\/\/(?:127\.0\.0\.1|localhost):19376\/v1\/?$/.test(endpoint)
}

function agentRouterEndpoint(protocol: CloudProtocol): string {
  return protocol === 'anthropic' ? 'https://agentrouter.org' : 'https://agentrouter.org/v1'
}

/** An existing instance loaded into the dialog for editing. */
export interface CloudAiEditTarget {
  id: string
  name: string
  /** Concrete stored backend_type ('openai' | 'anthropic' | legacy vendor). */
  backend_type: string
  endpoint: string
  model: string
  api_key_configured: boolean
  /** Stored context window, if any (prefill the ctx field on edit). */
  max_context?: number
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Receives the payload plus the instance id when editing. */
  onSubmit: (data: CreateLlmBackendRequest, editingId?: string) => Promise<void>
  /** Present → edit mode. */
  editing?: CloudAiEditTarget | null
  onConfigured?: () => Promise<void>
}

const DEFAULTS: Record<'openai' | 'anthropic', { endpoint: string; model: string }> = {
  // Anthropic may be pasted with or without /v1 — the runtime normalizes
  // (auto-appends /v1 before joining "/messages").
  openai: { endpoint: 'https://api.openai.com/v1', model: 'gpt-4.1-mini' },
  anthropic: { endpoint: 'https://api.anthropic.com/v1', model: 'claude-sonnet-4-5' },
}

/** Map a stored backend_type onto a protocol path. Legacy vendor types
 *  (qwen/deepseek/glm/…) are all OpenAI-compatible endpoints. */
function protocolOf(backendType: string): 'openai' | 'anthropic' {
  return backendType === 'anthropic' ? 'anthropic' : 'openai'
}

export function CloudAiAddDialog({ open, onOpenChange, onSubmit, editing, onConfigured }: Props) {
  const { t } = useTranslation(['plugins'])
  const [protocol, setProtocol] = useState<'openai' | 'anthropic'>('openai')
  const [provider, setProvider] = useState<CloudProvider>('custom')
  const [serviceMode, setServiceMode] = useState(false)
  const [name, setName] = useState('')
  const [endpoint, setEndpoint] = useState(DEFAULTS.openai.endpoint)
  const [model, setModel] = useState(DEFAULTS.openai.model)
  const [apiKey, setApiKey] = useState('')
  // Optional real context window for custom backends (e.g. RKLLM3 runs -c 16384
  // but openai-typed backends default to 128000 — over-sized prompts hang the
  // runtime). Sent as capabilities.max_context; the server respects it.
  const [ctx, setCtx] = useState<string>('')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [models, setModels] = useState<Array<{ id: string; name: string }>>([])
  const [modelsLoading, setModelsLoading] = useState(false)
  const [modelsError, setModelsError] = useState<string | null>(null)
  const [modelsLoaded, setModelsLoaded] = useState(false)
  // Invalidate in-flight requests when the key/provider changes or the dialog closes.
  const modelRequest = useRef(0)

  const invalidateModels = useCallback(() => {
    modelRequest.current += 1
    setModels([])
    setModelsLoading(false)
    setModelsError(null)
    setModelsLoaded(false)
  }, [])

  useEffect(() => {
    invalidateModels()
    if (!open) {
      setApiKey('')
      return
    }
    setError(null)
    setCtx(editing?.max_context ? String(editing.max_context) : '')
    if (editing) {
      const p = protocolOf(editing.backend_type)
      setProvider(isAgentRouterEndpoint(editing.endpoint) || isAgentRouterGateway(editing.endpoint) ? 'agentrouter' : 'custom')
      setServiceMode(isAgentRouterGateway(editing.endpoint))
      setProtocol(p)
      setName(editing.name)
      setEndpoint(isAgentRouterEndpoint(editing.endpoint) ? agentRouterEndpoint(p) : editing.endpoint)
      setModel(editing.model)
      // Never returned by the API — sentinel mask says "configured".
      setApiKey(editing.api_key_configured ? API_KEY_MASK : '')
    } else {
      setProvider('custom')
      setServiceMode(false)
      setProtocol('openai')
      setName('')
      setApiKey('')
      setEndpoint(DEFAULTS.openai.endpoint)
      setModel(DEFAULTS.openai.model)
    }
    return () => { modelRequest.current += 1 }
  }, [open, editing, invalidateModels])

  const switchProtocol = (p: 'openai' | 'anthropic') => {
    // Keep custom gateway URLs; named providers use their protocol's base URL.
    setProtocol(p)
    if (provider === 'agentrouter') setEndpoint(agentRouterEndpoint(p))
    setError(null)
  }

  const switchProvider = (next: CloudProvider) => {
    invalidateModels()
    setProvider(next)
    setServiceMode(next === 'agentrouter')
    setApiKey('')
    setError(null)
    if (next === 'agentrouter') {
      setEndpoint(agentRouterEndpoint(protocol))
      setModel('')
      if (!name.trim()) setName('AgentRouter')
    } else {
      setEndpoint(DEFAULTS[protocol].endpoint)
      setModel(DEFAULTS[protocol].model)
    }
  }

  const keyProvided =
    apiKey.trim() && apiKey !== API_KEY_MASK ? apiKey.trim() : undefined
  const editingAgentRouter = !!editing && isAgentRouterEndpoint(editing.endpoint)
  const needsKey = !editing || isAgentRouterGateway(editing.endpoint) || editingAgentRouter !== (provider === 'agentrouter')

  const loadModels = async () => {
    if (provider !== 'agentrouter' || (!keyProvided && !editingAgentRouter)) return
    const request = ++modelRequest.current
    setModelsLoading(true)
    setModelsError(null)
    setModelsLoaded(false)
    setModels([])
    try {
      const response = await api.listAgentRouterModels(
        keyProvided ? { api_key: keyProvided } : { backend_id: editing!.id }
      )
      if (request !== modelRequest.current) return
      setModels(response.models)
      setModelsLoaded(true)
    } catch (e) {
      if (request !== modelRequest.current) return
      setModelsError(e instanceof Error ? e.message : String(e))
    } finally {
      if (request === modelRequest.current) setModelsLoading(false)
    }
  }

  const chooseModel = (id: string) => {
    setModel(id)
    // AgentRouter recommends the Anthropic protocol for Claude models.
    const next = id.toLowerCase().startsWith('claude-') ? 'anthropic' : 'openai'
    setProtocol(next)
    setEndpoint(agentRouterEndpoint(next))
  }
  const matchingModels = models.filter(item =>
    `${item.id} ${item.name}`.toLowerCase().includes(model.trim().toLowerCase())
  )

  const handleSubmit = async () => {
    // Create: cloud protocol paths require an API key (backend validate()
    // enforces it) — catch it client-side. Edit: blank/masked = keep stored.
    if (!name.trim() || !endpoint.trim() || !model.trim() || (needsKey && !keyProvided)) {
      setError(t('plugins:llm.cloudFillRequired'))
      return
    }
    setSaving(true)
    try {
      await onSubmit(
        {
          name: name.trim(),
          backend_type: protocol,
          endpoint: endpoint.trim(),
          model: model.trim(),
          ...(keyProvided ? { api_key: keyProvided } : {}),
          temperature: 0.7,
          ...(protocol === 'openai' ? { top_p: 0.9 } : {}),
          ...(ctx.trim() && !Number.isNaN(Number(ctx))
            ? { max_context: Number(ctx) }
            : {}),
        },
        editing?.id
      )
      onOpenChange(false)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setSaving(false)
    }
  }

  if (provider === 'agentrouter' && serviceMode) {
    return <AgentRouterServiceDialog open={open} onOpenChange={onOpenChange}
      initialName={name} editingId={editing?.id} onConfigured={onConfigured}
      onDirect={() => setServiceMode(false)} />
  }

  return (
    <UnifiedFormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={editing ? t('plugins:llm.cloudEditTitle') : t('plugins:llm.cloudAddTitle')}
      description={t('plugins:llm.cloudAddDesc')}
      icon={<Cloud className="h-5 w-5 text-muted-foreground" />}
      className="z-[110]"
      loading={false}
      isSubmitting={saving}
      onSubmit={handleSubmit}
      submitLabel={editing ? t('common:save', { defaultValue: 'Save' }) : t('common:add', { defaultValue: 'Add' })}
      cancelLabel={t('common:cancel', { defaultValue: 'Cancel' })}
      submitDisabled={!name.trim() || !endpoint.trim() || !model.trim() || (needsKey && !keyProvided) || saving}
      submitError={error ?? undefined}
    >
      <div className="space-y-4">
        {provider === 'agentrouter' && <Button type="button" variant="secondary" onClick={() => setServiceMode(true)}>{t('plugins:llm.serviceTitle')}</Button>}
        <FormField label={t('plugins:llm.cloudProvider')}>
          <Select value={provider} onValueChange={(value) => switchProvider(value as CloudProvider)}>
            <SelectTrigger aria-label={t('plugins:llm.cloudProvider')}><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="custom">{t('plugins:llm.cloudProviderCustom')}</SelectItem>
              <SelectItem value="agentrouter">AgentRouter</SelectItem>
            </SelectContent>
          </Select>
        </FormField>
        {/* Protocol path — OpenAI-compatible / Anthropic. In edit mode this
            switches the instance's backend_type. */}
        <FormField label={t('plugins:llm.cloudProtocol')} helpText={t('plugins:llm.cloudProtocolHelp')}>
          <Select value={protocol} onValueChange={(v) => switchProtocol(v as 'openai' | 'anthropic')}>
            <SelectTrigger className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="openai">{t('plugins:llm.protocol.openai.name')}</SelectItem>
              <SelectItem value="anthropic">{t('plugins:llm.protocol.anthropic.name')}</SelectItem>
            </SelectContent>
          </Select>
        </FormField>

        <FormField label={t('plugins:llm.name')}>
          <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="my-cloud-llm" />
        </FormField>

        <FormField label={t('plugins:llm.cloudCtx')} helpText={t('plugins:llm.cloudCtxHelp')}>
          <Input
            value={ctx}
            onChange={(e) => setCtx(e.target.value)}
            placeholder="16384"
            inputMode="numeric"
          />
        </FormField>
        <FormField label={t('plugins:llm.endpoint')}>
          <Input value={endpoint} onChange={(e) => setEndpoint(e.target.value)} placeholder="https://…" readOnly={provider === 'agentrouter'} />
        </FormField>

        <FormField label={t('plugins:llm.apiKey')} helpText={t('plugins:llm.apiKeyHelp')}>
          <PasswordInput
            value={apiKey}
            aria-label={t('plugins:llm.apiKey')}
            onChange={(e) => { invalidateModels(); setApiKey(e.target.value) }}
            onBlur={() => { if (!modelsLoaded && !modelsLoading) void loadModels() }}
            placeholder={editing && editing.api_key_configured ? t('plugins:llm.apiKeyKeepHint') : undefined}
          />
        </FormField>

        {provider === 'agentrouter' && (
          <div className="space-y-2">
            <Button type="button" variant="secondary" onClick={() => void loadModels()}
              disabled={modelsLoading || (!keyProvided && !editingAgentRouter)}>
              {modelsLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t(modelsLoading ? 'plugins:llm.cloudModelsLoading' : 'plugins:llm.cloudModelsLoad')}
            </Button>
            <p className="text-xs text-muted-foreground">{t('plugins:llm.agentRouterModelsHelp')}</p>
            {modelsError && <p role="alert" className="text-sm text-destructive">{modelsError} {t('plugins:llm.cloudModelsManual')}</p>}
            {modelsLoaded && <p role="status" className="text-xs text-muted-foreground">
              {t(models.length ? 'plugins:llm.cloudModelsCount' : 'plugins:llm.cloudModelsEmpty', { count: models.length })}
            </p>}
          </div>
        )}

        <div className="space-y-2">
          <FormField label={t('plugins:llm.model')}>
            <Input value={model} onChange={(e) => setModel(e.target.value)}
              aria-label={t('plugins:llm.model')}
              placeholder={provider === 'agentrouter' ? t('plugins:llm.cloudModelsSearch') : undefined} />
          </FormField>
          {provider === 'agentrouter' && models.length > 0 && (
            <ul aria-label={t('plugins:llm.cloudModelsAvailable')} className="max-h-40 overflow-y-auto rounded-md border divide-y">
              {matchingModels.map(item => (
                <li key={item.id}>
                  <button type="button" onClick={() => chooseModel(item.id)}
                    className="w-full px-3 py-2 text-left text-sm hover:bg-muted focus-visible:bg-muted break-all"
                    aria-pressed={model === item.id}>
                    {item.name}{item.name !== item.id && <span className="ml-2 text-xs text-muted-foreground">{item.id}</span>}
                  </button>
                </li>
              ))}
              {matchingModels.length === 0 && <li className="px-3 py-2 text-sm text-muted-foreground">{t('plugins:llm.cloudModelsNoMatch')}</li>}
            </ul>
          )}
        </div>
      </div>
    </UnifiedFormDialog>
  )
}
