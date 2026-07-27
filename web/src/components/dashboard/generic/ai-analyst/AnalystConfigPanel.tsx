import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Settings2, Cpu, MessageSquare, Gauge, Eye } from 'lucide-react'
import { UnifiedFormDialog } from '@/components/dialog/UnifiedFormDialog'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { Slider } from '@/components/ui/slider'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useAnalystModels } from './useAnalystModels'
import type { AiAnalystConfig } from './types'
import { DEFAULT_SYSTEM_PROMPT } from './types'
import { cn } from "@/lib/utils"
import { textNano } from "@/design-system/tokens/typography"

interface AnalystConfigPanelProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  config: AiAnalystConfig
  onSave: (config: Partial<AiAnalystConfig>) => void
  dataSource?: string
}

export function AnalystConfigPanel({ open, onOpenChange, config, onSave, dataSource }: AnalystConfigPanelProps) {
  const { t } = useTranslation('dashboardComponents')
  const [systemPrompt, setSystemPrompt] = useState(config.systemPrompt)
  const [modelId, setModelId] = useState(config.modelId || '__auto__')
  const [contextWindowSize, setContextWindowSize] = useState(config.contextWindowSize)
  const { models, loading } = useAnalystModels()

  useEffect(() => {
    setSystemPrompt(config.systemPrompt)
    setModelId(config.modelId || '__auto__')
    setContextWindowSize(config.contextWindowSize)
  }, [config])

  const handleSave = async () => {
    const isAuto = modelId === '__auto__'
    const selectedModel = isAuto ? undefined : models.find((m) => m.id === modelId)
    // Use default prompt if user left it empty
    const effectivePrompt = systemPrompt.trim() || DEFAULT_SYSTEM_PROMPT
    onSave({
      systemPrompt: effectivePrompt,
      modelId: isAuto ? undefined : modelId,
      modelName: selectedModel?.name,
      contextWindowSize,
    })
    onOpenChange(false)
  }

  return (
    <UnifiedFormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={t('aiAnalyst.configTitle')}
      icon={<Settings2 className="h-4 w-4" />}
      width="lg"
      onSubmit={handleSave}
      submitLabel={t('common:save')}
    >
      <div className="space-y-4">
        {/* Data Source */}
        <div>
          <label className="text-xs text-muted-foreground">{t('aiAnalyst.dataSource')}</label>
          <div className="mt-1 px-3 py-2 rounded-lg bg-muted-30 border border-border text-sm">
            {dataSource || t('aiAnalyst.dataSourceNotSet')}
          </div>
        </div>

        {/* Model Selection */}
        <div>
          <label className="text-xs text-muted-foreground flex items-center gap-1">
            <Cpu className="h-4 w-4" />
            {t('aiAnalyst.selectModel')}
          </label>
          {loading ? (
            <div className="mt-1 text-xs text-muted-foreground">{t('aiAnalyst.loadingModels')}</div>
          ) : models.length === 0 ? (
            <div className="mt-1 text-xs text-warning">{t('aiAnalyst.noLlmBackends')}</div>
          ) : (
            <Select value={modelId} onValueChange={setModelId}>
              <SelectTrigger className="mt-1">
                <SelectValue placeholder={t('aiAnalyst.selectModelPlaceholder')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__auto__">{t('aiAnalyst.autoDefault')}</SelectItem>
                {models.map((m) => (
                  <SelectItem key={`${m.backendId}-${m.id}`} value={m.id}>
                    <span className="flex items-center gap-2">
                      {m.name} ({m.backendName})
                      {m.isMultimodal && (
                        <Eye className="h-3 w-3 text-info" />
                      )}
                    </span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
        </div>

        {/* System Prompt */}
        <div>
          <label className="text-xs text-muted-foreground flex items-center gap-1">
            <MessageSquare className="h-4 w-4" />
            {t('aiAnalyst.systemPrompt')}
          </label>
          <Textarea
            value={systemPrompt}
            onChange={(e) => setSystemPrompt(e.target.value)}
            className="mt-1 min-h-[160px] text-xs"
            placeholder={t('aiAnalyst.systemPromptPlaceholder')}
          />
        </div>

        {/* Context Window */}
        <div>
          <label className="text-xs text-muted-foreground flex items-center gap-1">
            <Gauge className="h-4 w-4" />
            {t('aiAnalyst.contextWindow')}
          </label>
          <div className="mt-1 flex items-center gap-3">
            <Slider
              min={1}
              max={20}
              step={1}
              value={[contextWindowSize]}
              onValueChange={(vals) => setContextWindowSize(vals[0])}
              className="flex-1"
            />
            <Badge variant="outline" className="text-xs h-6 px-2">
              {contextWindowSize}
            </Badge>
          </div>
          <p className={cn(textNano, "text-muted-foreground mt-1")}>
            {t('aiAnalyst.contextWindowHelp')}
          </p>
        </div>
      </div>
    </UnifiedFormDialog>
  )
}
