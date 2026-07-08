import { useState, useEffect } from 'react'
import { Settings2, Cpu, MessageSquare, Gauge, Eye } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
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
  const [systemPrompt, setSystemPrompt] = useState(config.systemPrompt)
  const [modelId, setModelId] = useState(config.modelId || '__auto__')
  const [contextWindowSize, setContextWindowSize] = useState(config.contextWindowSize)
  const { models, loading } = useAnalystModels()

  useEffect(() => {
    setSystemPrompt(config.systemPrompt)
    setModelId(config.modelId || '__auto__')
    setContextWindowSize(config.contextWindowSize)
  }, [config])

  const handleSave = () => {
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
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg z-[110]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-base">
            <Settings2 className="h-4 w-4" />
            AI Analyst Configuration
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          {/* Data Source */}
          <div>
            <label className="text-xs text-muted-foreground">Data Source</label>
            <div className="mt-1 px-3 py-2 rounded-lg bg-muted-30 border border-border text-sm">
              {dataSource || 'Not configured (use component settings)'}
            </div>
          </div>

          {/* Model Selection */}
          <div>
            <label className="text-xs text-muted-foreground flex items-center gap-1">
              <Cpu className="h-4 w-4" />
              Analysis Model
            </label>
            {loading ? (
              <div className="mt-1 text-xs text-muted-foreground">Loading models...</div>
            ) : models.length === 0 ? (
              <div className="mt-1 text-xs text-warning">No LLM backends found. Configure an LLM backend first.</div>
            ) : (
              <Select value={modelId} onValueChange={setModelId}>
                <SelectTrigger className="mt-1">
                  <SelectValue placeholder="Select a model" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="__auto__">Auto (default)</SelectItem>
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
              System Prompt
            </label>
            <Textarea
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              className="mt-1 min-h-[80px] text-xs"
              placeholder="Describe how the AI should analyze the data..."
            />
          </div>

          {/* Context Window */}
          <div>
            <label className="text-xs text-muted-foreground flex items-center gap-1">
              <Gauge className="h-4 w-4" />
              Context Window
            </label>
            <div className="mt-1 flex items-center gap-3">
              <input
                type="range"
                min={1}
                max={20}
                step={1}
                value={contextWindowSize}
                onChange={(e) => setContextWindowSize(Number(e.target.value))}
                className="flex-1"
              />
              <Badge variant="outline" className="text-xs h-6 px-2">
                {contextWindowSize}
              </Badge>
            </div>
            <p className={cn(textNano, "text-muted-foreground mt-1")}>
              Number of recent data+response pairs to display in timeline
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSave}>Save</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
