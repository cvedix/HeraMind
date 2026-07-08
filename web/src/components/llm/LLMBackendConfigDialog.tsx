import { useEffect, useState, useCallback } from "react"
import { useTranslation } from "react-i18next"
import { Settings } from "lucide-react"
import { Input } from "@/components/ui/input"
import { PasswordInput } from "@/components/ui/password-input"
import { fetchAPI } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import { FormSectionGroup } from "@/components/ui/form-section"
import { FormField } from "@/components/ui/field"
import { UnifiedFormDialog } from "@/components/dialog/UnifiedFormDialog"

interface LLMBackendConfigDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  backend?: LlmBackendInstance | null
  mode: 'create' | 'edit'
  backendType?: string  // Required for create mode
  onRefresh?: () => void  // Callback to refresh the backend list
}

interface LlmBackendInstance {
  id: string
  name: string
  backend_type: string
  config: Record<string, unknown>
  enabled: boolean
}

interface BackendTypeDefinition {
  id: string
  name: string
  description: string
  config_schema?: Record<string, unknown>
}

export function LLMBackendConfigDialog({
  open,
  onOpenChange,
  backend,
  mode,
  backendType,
  onRefresh,
}: LLMBackendConfigDialogProps) {
  const { t } = useTranslation()
  const { toast } = useToast()

  const [loading, setLoading] = useState(false)
  const [config, setConfig] = useState<Record<string, unknown>>({})
  const [name, setName] = useState('')
  const [endpoint, setEndpoint] = useState('')
  const [model, setModel] = useState('')
  const [apiKey, setApiKey] = useState('')

  useEffect(() => {
    if (backend && mode === 'edit') {
      setName(backend.name)
      setConfig(backend.config || {})
      setEndpoint((backend.config as any)?.endpoint || '')
      setModel((backend.config as any)?.model || '')
      setApiKey('')
    } else {
      setName('')
      setConfig({})
      setEndpoint('')
      setModel('')
      setApiKey('')
    }
  }, [backend, mode])

  const handleSubmit = useCallback(async () => {
    setLoading(true)
    try {
      if (mode === 'create') {
        if (!backendType) {
          toast({
            title: t('common:error'),
            description: t('settings:llm.backendTypeRequired', { defaultValue: 'Backend type is required' }),
            variant: 'destructive',
          })
          return
        }

        const createData = {
          name,
          backend_type: backendType,
          endpoint: endpoint || undefined,
          model,
          api_key: apiKey || undefined,
        }

        await fetchAPI('/llm-backends', {
          method: 'POST',
          body: JSON.stringify(createData),
        })

        toast({
          title: t('common:success'),
          description: t('settings:llm.backendCreated', { defaultValue: 'Backend created successfully' }),
        })

        onRefresh?.()
      } else {
        await fetchAPI(`/llm-backends/${backend?.id}`, {
          method: 'PUT',
          body: JSON.stringify({ name, config }),
        })
      }
      onOpenChange(false)
    } catch (error) {
      console.error('Failed to save backend:', error)
      toast({
        title: t('common:error'),
        description: mode === 'create'
          ? t('settings:llm.createFailed', { defaultValue: 'Failed to create backend' })
          : t('settings:llm.updateFailed', { defaultValue: 'Failed to update backend' }),
        variant: 'destructive',
      })
    } finally {
      setLoading(false)
    }
  }, [mode, backendType, name, endpoint, model, apiKey, config, backend, onRefresh, onOpenChange, toast, t])

  return (
    <UnifiedFormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={mode === 'create' ? t('settings:llm.create') : t('settings:llm.edit')}
      icon={<Settings className="h-5 w-5 text-primary" />}
      width="md"
      onSubmit={handleSubmit}
      isSubmitting={loading}
      submitLabel={mode === 'create' ? t('common:create') : t('common:save')}
      preventCloseOnSubmit
    >
      <FormSectionGroup>
        <FormField
          label={t('settings:llm.name')}
          required
        >
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={t('settings:llm.namePlaceholder')}
          />
        </FormField>

        {mode === 'create' && (
          <>
            <FormField
              label={t('settings:llm.endpoint')}
            >
              <Input
                value={endpoint}
                onChange={(e) => setEndpoint(e.target.value)}
                placeholder="http://localhost:11434"
              />
            </FormField>

            <FormField
              label={t('settings:llm.model')}
              required
            >
              <Input
                value={model}
                onChange={(e) => setModel(e.target.value)}
                placeholder="qwen3.5:4b"
              />
            </FormField>

            <FormField
              label={t('settings:llm.apiKey')}
            >
              <PasswordInput
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder={t('settings:llm.apiKeyPlaceholder', { defaultValue: 'Optional API key' })}
              />
            </FormField>
          </>
        )}

        {mode === 'edit' && (
          <div className="text-sm text-muted-foreground p-4 border rounded-lg bg-muted-30">
            {t('settings:llm.configPlaceholder', { defaultValue: 'Configuration fields will be dynamically generated based on backend type' })}
          </div>
        )}
      </FormSectionGroup>
    </UnifiedFormDialog>
  )
}
