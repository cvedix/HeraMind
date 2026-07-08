import { useState, useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { Card } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Switch } from '@/components/ui/switch'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Slider } from '@/components/ui/slider'
import {
  Sparkles,
  Loader2,
  CheckCircle2,
  AlertTriangle,
  ChevronRight,
  ChevronLeft,
  Info,
  Database,
  Terminal,
  Eye,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { api } from '@/lib/api'
import { useToast } from '@/hooks/use-toast'
import { showErrorToast } from '@/lib/error-messages'
import { useIsMobile } from '@/hooks/useMobile'
import { FormField } from '@/components/ui/field'
// Unified dialog components
import {
  FullScreenDialog,
  FullScreenDialogHeader,
  FullScreenDialogContent,
  FullScreenDialogFooter,
  FullScreenDialogSidebar,
  FullScreenDialogMain,
  VerticalStepper,
  type Step as StepperStep,
} from '@/components/automation/dialog'

export interface DeviceTypeGeneratorDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onDeviceTypeCreated: () => void
}

interface GeneratedMetric {
  name: string
  path: string
  display_name: string
  description: string
  data_type: string
  semantic_type: string
  unit: string | null
  readable: boolean
  writable: boolean
  confidence: number
}

interface GeneratedCommand {
  name: string
  display_name: string
  description: string
  parameters: Array<{ name: string; type_: string; required: boolean }>
  confidence: number
}

function convertDataType(dataType: string): any {
  const normalized = dataType.toLowerCase()
  if (normalized === 'enum') {
    return 'string'
  }
  return normalized
}

interface GeneratedDeviceType {
  id: string
  name: string
  description: string
  category: string
  manufacturer: string
  metrics: GeneratedMetric[]
  commands: GeneratedCommand[]
  confidence: number
}

interface ValidationResult {
  is_valid: boolean
  issues: string[]
  warnings: string[]
}

type StepValue = 'samples' | 'metrics' | 'commands' | 'preview'

// Steps config for sidebar
const stepsConfig: { key: StepValue; label: string; shortLabel: string; icon: React.ReactNode }[] = [
  { key: 'samples', label: 'Samples', shortLabel: 'Samples', icon: <Database className="h-4 w-4" /> },
  { key: 'metrics', label: 'Metrics', shortLabel: 'Metrics', icon: <Eye className="h-4 w-4" /> },
  { key: 'commands', label: 'Commands', shortLabel: 'Commands', icon: <Terminal className="h-4 w-4" /> },
  { key: 'preview', label: 'Preview', shortLabel: 'Preview', icon: <CheckCircle2 className="h-4 w-4" /> },
]

export function DeviceTypeGeneratorDialog({
  open,
  onOpenChange,
  onDeviceTypeCreated,
}: DeviceTypeGeneratorDialogProps) {
  const { t } = useTranslation(['common', 'devices'])
  const { toast } = useToast()
  const isMobile = useIsMobile()

  // Sample data input
  const [sampleData, setSampleData] = useState('')

  // Analysis state
  const [analyzing, setAnalyzing] = useState(false)
  const [generated, setGenerated] = useState<GeneratedDeviceType | null>(null)
  const [currentStep, setCurrentStep] = useState<StepValue>('samples')

  // Creation state
  const [creating, setCreating] = useState(false)
  const [deviceTypeName, setDeviceTypeName] = useState('')
  const [deviceTypeId, setDeviceTypeId] = useState('')
  const [enableAfterCreate, setEnableAfterCreate] = useState(true)

  // Validation
  const [validation, setValidation] = useState<ValidationResult | null>(null)

  // Filtering configuration
  const [minCoverage, setMinCoverage] = useState(0)
  const [minConfidence, setMinConfidence] = useState(0)

  // Completed steps tracking
  const completedSteps = useMemo(() => {
    const completed = new Set<string>()
    if (generated) {
      completed.add('samples')
    }
    return completed
  }, [generated])

  const parseSamples = useCallback((): Array<{ timestamp: number; data: Record<string, unknown> }> => {
    const trimmed = sampleData.trim()
    if (!trimmed) return []

    const samples: Array<{ timestamp: number; data: Record<string, unknown> }> = []

    try {
      const parsed = JSON.parse(trimmed)
      if (Array.isArray(parsed)) {
        return parsed.filter((item): item is Record<string, unknown> =>
          item && typeof item === 'object'
        ).map(data => ({
          timestamp: Date.now(),
          data,
        }))
      } else if (typeof parsed === 'object' && parsed !== null) {
        return [{
          timestamp: Date.now(),
          data: parsed as Record<string, unknown>,
        }]
      }
    } catch {
      // Fall through to JSONL parsing
    }

    const lines = trimmed.split('\n').filter(Boolean)
    for (const line of lines) {
      const trimmedLine = line.trim()
      if (!trimmedLine) continue

      try {
        const data = JSON.parse(trimmedLine)
        if (typeof data === 'object' && data !== null) {
          samples.push({
            timestamp: Date.now(),
            data: data as Record<string, unknown>,
          })
        }
      } catch {
        // Skip invalid JSON lines
      }
    }

    return samples
  }, [sampleData])

  const tGen = (key: string, params?: Record<string, unknown>) => t(`devices:types.generator.${key}`, params)

  const handleAnalyze = async () => {
    const samples = parseSamples()
    if (samples.length === 0) {
      toast({
        title: tGen('toast.noValidSamples'),
        description: tGen('step1.noValidSamplesDesc'),
        variant: 'destructive',
      })
      return
    }

    setAnalyzing(true)
    try {
      const response = await api.generateDeviceTypeFromSamples({
        device_id: 'device-type',
        manufacturer: undefined,
        samples,
        min_coverage: minCoverage / 100,
        min_confidence: minConfidence / 100,
      })

      setGenerated(response as unknown as GeneratedDeviceType)

      if (response.name) {
        setDeviceTypeName(response.name)
      }
      if (response.id) {
        setDeviceTypeId(response.id.replace('auto-generated-', ''))
      }

      setCurrentStep('metrics')
      toast({
        title: tGen('toast.analysisComplete'),
        description: tGen('toast.analysisCompleteDesc', {
          metrics: response.metrics?.length || 0,
          commands: response.commands?.length || 0,
        }),
      })
    } catch (error) {
      showErrorToast(toast, error, tGen('toast.analysisFailed'))
    } finally {
      setAnalyzing(false)
    }
  }

  const handleValidate = useCallback(() => {
    if (!generated) return false

    const issues: string[] = []
    const warnings: string[] = []

    if (generated.metrics.length === 0) {
      warnings.push(tGen('step4.noMetricsWarning'))
    }

    if (generated.confidence < 0.5) {
      warnings.push(
        tGen('step4.lowConfidenceWarning', {
          percent: Math.round(generated.confidence * 100),
        })
      )
    }

    if (generated.category === 'unknown') {
      warnings.push(tGen('step4.unknownCategoryWarning'))
    }

    const metricNames = new Set<string>()
    for (const metric of generated.metrics) {
      if (!metricNames.add(metric.name)) {
        issues.push(tGen('step4.duplicateMetric', { name: metric.name }))
      }
    }

    setValidation({
      is_valid: issues.length === 0,
      issues,
      warnings,
    })

    return issues.length === 0
  }, [generated, tGen])

  const handleCreate = async () => {
    if (!generated) return
    if (!deviceTypeName.trim()) {
      toast({
        title: tGen('toast.nameRequired'),
        description: tGen('toast.nameRequiredDesc'),
        variant: 'destructive',
      })
      return
    }
    if (!deviceTypeId.trim()) {
      toast({
        title: tGen('toast.idRequired'),
        description: tGen('toast.idRequiredDesc'),
        variant: 'destructive',
      })
      return
    }

    if (!handleValidate()) {
      toast({
        title: tGen('toast.validationFailed'),
        description: tGen('toast.validationFailedDesc'),
        variant: 'destructive',
      })
      return
    }

    setCreating(true)
    try {
      await api.addDeviceType({
        device_type: deviceTypeId,
        name: deviceTypeName,
        description: generated.description,
        categories: [generated.category],
        mode: 'full',
        metrics: generated.metrics.map((m) => ({
          name: m.name,
          display_name: m.display_name,
          data_type: convertDataType(m.data_type),
          unit: m.unit || undefined,
          min: undefined,
          max: undefined,
          required: false,
        })),
        commands: generated.commands.map((c) => ({
          name: c.name,
          display_name: c.display_name,
          payload_template: `{"action": "${c.name}"}`,
          parameters: c.parameters.map((p) => ({
            name: p.name,
            display_name: p.name,
            data_type: convertDataType(p.type_),
            default_value: undefined,
            min: undefined,
            max: undefined,
            unit: undefined,
            allowed_values: [],
          })),
          samples: [],
          llm_hints: c.description,
        })),
        uplink_samples: parseSamples().map((s) => s.data),
      })

      toast({
        title: tGen('toast.created'),
        description: tGen('toast.createdDesc', { name: deviceTypeName }),
      })

      onDeviceTypeCreated()
      onOpenChange(false)

      // Reset state
      setSampleData('')
      setGenerated(null)
      setDeviceTypeName('')
      setDeviceTypeId('')
      setCurrentStep('samples')
      setValidation(null)
    } catch (error) {
      showErrorToast(toast, error, tGen('toast.creationFailed'))
    } finally {
      setCreating(false)
    }
  }

  const loadSample = () => {
    setSampleData(`// JSONL format (one JSON object per line):
{"temp": 25.5, "hum": 60, "battery": 85}
{"temp": 26.0, "hum": 58, "battery": 84}
{"temp": 24.8, "hum": 62, "battery": 83}
{"temp": 25.2, "hum": 59, "battery": 84}

// You can also use JSON array: [{"temp": 25.5, "hum": 60}, {"temp": 26.0, "hum": 58}]
// Or a single multi-line JSON object`)
  }

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'text-success'
    if (confidence >= 0.5) return 'text-warning'
    return 'text-error'
  }

  const getConfidenceLabel = (confidence: number) => {
    if (confidence >= 0.8) return tGen('confidence.high')
    if (confidence >= 0.5) return tGen('confidence.medium')
    return tGen('confidence.low')
  }

  // Navigation handlers
  const handleStepChange = (stepId: string) => {
    // Only allow navigating to completed steps or the next available step
    const stepOrder: StepValue[] = ['samples', 'metrics', 'commands', 'preview']
    const currentIndex = stepOrder.indexOf(currentStep)
    const targetIndex = stepOrder.indexOf(stepId as StepValue)

    // Allow going back
    if (targetIndex <= currentIndex) {
      setCurrentStep(stepId as StepValue)
      return
    }

    // Only allow going forward if we have generated results
    if (generated && targetIndex <= 3) {
      setCurrentStep(stepId as StepValue)
    }
  }

  const handleNext = () => {
    const stepOrder: StepValue[] = ['samples', 'metrics', 'commands', 'preview']
    const currentIndex = stepOrder.indexOf(currentStep)
    if (currentIndex < stepOrder.length - 1 && generated) {
      setCurrentStep(stepOrder[currentIndex + 1])
      return true
    }
    return false
  }

  const handlePrevious = () => {
    const stepOrder: StepValue[] = ['samples', 'metrics', 'commands', 'preview']
    const currentIndex = stepOrder.indexOf(currentStep)
    if (currentIndex > 0) {
      setCurrentStep(stepOrder[currentIndex - 1])
    }
  }

  // Render step content
  const renderStepContent = () => {
    switch (currentStep) {
      case 'samples':
        return (
          <div className="space-y-6">
            {/* Info banner */}
            <div className="flex items-center justify-between flex-wrap gap-2">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Info className="h-4 w-4" />
                <span>{tGen('step1.info')}</span>
              </div>
              <Button variant="ghost" size="sm" onClick={loadSample}>
                {tGen('step1.loadSample')}
              </Button>
            </div>

            {/* Filtering Configuration */}
            <div className="space-y-3">
              <Label className="text-sm font-medium">{tGen('step1.filterConfig')}</Label>
              <Card className="p-4 bg-muted-30">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="min-coverage" className="text-sm">
                      {tGen('step1.minCoverage')}
                    </Label>
                    <span className="text-xs text-muted-foreground font-mono">
                      {minCoverage}%
                    </span>
                  </div>
                  <Slider
                    id="min-coverage"
                    value={[minCoverage]}
                    onValueChange={([v]: number[]) => setMinCoverage(v)}
                    min={0}
                    max={100}
                    step={5}
                    className="w-full"
                  />
                  <p className="text-xs text-muted-foreground">
                    {tGen('step1.minCoverageDesc')}
                  </p>
                </div>

                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="min-confidence" className="text-sm">
                      {tGen('step1.minConfidence')}
                    </Label>
                    <span className="text-xs text-muted-foreground font-mono">
                      {minConfidence}%
                    </span>
                  </div>
                  <Slider
                    id="min-confidence"
                    value={[minConfidence]}
                    onValueChange={([v]: number[]) => setMinConfidence(v)}
                    min={0}
                    max={100}
                    step={5}
                    className="w-full"
                  />
                  <p className="text-xs text-muted-foreground">
                    {tGen('step1.minConfidenceDesc')}
                  </p>
                </div>
              </div>
              </Card>
            </div>

            {/* Sample Data Input */}
            <FormField label={tGen('step1.sampleData')}>
              <Textarea
                placeholder={tGen('step1.sampleDataPlaceholder')}
                value={sampleData}
                onChange={(e) => setSampleData(e.target.value)}
                rows={8}
                className="font-mono text-sm"
              />
              <p className="text-xs text-muted-foreground mt-1">
                {tGen('step1.validSamples', { count: parseSamples().length })}
              </p>
            </FormField>

            {/* Analyze Button */}
            <Button
              onClick={handleAnalyze}
              disabled={parseSamples().length === 0 || analyzing}
              className="w-full"
              size="lg"
            >
              {analyzing ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  {tGen('step1.analyzing')}
                </>
              ) : (
                <>
                  <Sparkles className="mr-2 h-4 w-4" />
                  {tGen('step1.generateButton')}
                </>
              )}
            </Button>
          </div>
        )

      case 'metrics':
        if (!generated) return null
        return (
          <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between flex-wrap gap-2">
              <div>
                <h3 className="text-lg font-semibold">{tGen('step2.title')}</h3>
                <p className="text-sm text-muted-foreground">
                  {tGen('step2.metricsFound', { count: generated.metrics.length })}
                </p>
              </div>
              <Badge variant="outline" className={getConfidenceColor(generated.confidence)}>
                {getConfidenceLabel(generated.confidence)} {tGen('step2.confidence')}
              </Badge>
            </div>

            {/* Metrics Table */}
            <Card>
              <div className="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{tGen('step2.field')}</TableHead>
                      <TableHead>{tGen('step2.semanticType')}</TableHead>
                      <TableHead className="hidden sm:table-cell">{tGen('step2.displayName')}</TableHead>
                      <TableHead>{tGen('step2.unit')}</TableHead>
                      <TableHead className="text-center hidden sm:table-cell">{tGen('step2.readable')}</TableHead>
                      <TableHead className="text-center hidden sm:table-cell">{tGen('step2.writable')}</TableHead>
                      <TableHead>{tGen('step2.confidence')}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {generated.metrics.map((metric) => (
                      <TableRow key={metric.name}>
                        <TableCell className="font-mono text-xs">{metric.path}</TableCell>
                        <TableCell>
                          <Badge variant="secondary">{metric.semantic_type}</Badge>
                        </TableCell>
                        <TableCell className="hidden sm:table-cell">{metric.display_name}</TableCell>
                        <TableCell>{metric.unit || '-'}</TableCell>
                        <TableCell className="text-center hidden sm:table-cell">
                          {metric.readable ? (
                            <CheckCircle2 className="h-4 w-4 text-success mx-auto" />
                          ) : (
                            <span className="text-muted-foreground">-</span>
                          )}
                        </TableCell>
                        <TableCell className="text-center hidden sm:table-cell">
                          {metric.writable ? (
                            <CheckCircle2 className="h-4 w-4 text-success mx-auto" />
                          ) : (
                            <span className="text-muted-foreground">-</span>
                          )}
                        </TableCell>
                        <TableCell>
                          <Badge
                            variant="outline"
                            className={getConfidenceColor(metric.confidence)}
                          >
                            {Math.round(metric.confidence * 100)}%
                          </Badge>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </Card>
          </div>
        )

      case 'commands':
        if (!generated) return null
        return (
          <div className="space-y-6">
            {/* Header */}
            <div>
              <h3 className="text-lg font-semibold">{tGen('step3.title')}</h3>
              <p className="text-sm text-muted-foreground">
                {tGen('step3.commandsFound', { count: generated.commands.length })}
              </p>
            </div>

            {/* Commands Table or Empty State */}
            {generated.commands.length > 0 ? (
              <Card>
                <div className="overflow-x-auto">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>{tGen('step3.command')}</TableHead>
                        <TableHead>{tGen('step2.displayName')}</TableHead>
                        <TableHead className="hidden sm:table-cell">{tGen('step3.description')}</TableHead>
                        <TableHead>{tGen('step3.parameters')}</TableHead>
                        <TableHead>{tGen('step2.confidence')}</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {generated.commands.map((cmd) => (
                        <TableRow key={cmd.name}>
                          <TableCell className="font-mono text-xs">{cmd.name}</TableCell>
                          <TableCell>{cmd.display_name}</TableCell>
                          <TableCell className="text-sm text-muted-foreground hidden sm:table-cell">
                            {cmd.description}
                          </TableCell>
                          <TableCell>
                            {cmd.parameters.length > 0 ? (
                              cmd.parameters.map((p) => p.name).join(', ')
                            ) : (
                              '-'
                            )}
                          </TableCell>
                          <TableCell>
                            <Badge
                              variant="outline"
                              className={getConfidenceColor(cmd.confidence)}
                            >
                              {Math.round(cmd.confidence * 100)}%
                            </Badge>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              </Card>
            ) : (
              <Card className="p-8 text-center">
                <p className="text-muted-foreground">
                  {tGen('step3.noCommands')}
                  <br />
                  {tGen('step3.noCommandsDesc')}
                </p>
              </Card>
            )}
          </div>
        )

      case 'preview':
        if (!generated) return null
        return (
          <div className="space-y-6">
            {/* Header */}
            <div>
              <h3 className="text-lg font-semibold">{tGen('step4.title')}</h3>
              <p className="text-sm text-muted-foreground">
                {tGen('step4.description')}
              </p>
            </div>

            {/* Name and ID Inputs */}
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <FormField label={tGen('step4.typeName')} required>
                <Input
                  placeholder={tGen('step4.typeNamePlaceholder')}
                  value={deviceTypeName}
                  onChange={(e) => setDeviceTypeName(e.target.value)}
                />
              </FormField>
              <FormField label={tGen('step4.typeId')} required>
                <Input
                  placeholder={tGen('step4.typeIdPlaceholder')}
                  value={deviceTypeId}
                  onChange={(e) => setDeviceTypeId(e.target.value)}
                />
              </FormField>
            </div>

            {/* JSON Preview */}
            <div className="space-y-3">
              <Label className="text-sm font-medium">{tGen('step4.jsonPreview')}</Label>
              <Card className="p-4 border">
                <ScrollArea className="h-48">
                <pre className="text-xs font-mono">
                  {JSON.stringify(
                    {
                      device_type: deviceTypeId || 'example-id',
                      name: deviceTypeName || 'Example Device',
                      description: generated.description,
                      categories: [generated.category],
                      mode: 'full',
                      metrics: generated.metrics.map((m) => ({
                        name: m.name,
                        display_name: m.display_name,
                        data_type: convertDataType(m.data_type),
                        unit: m.unit || undefined,
                      })),
                      commands: generated.commands.map((c) => ({
                        name: c.name,
                        display_name: c.display_name,
                        payload_template: `{"action": "${c.name}"}`,
                      })),
                    },
                    null,
                    2
                  )}
                </pre>
              </ScrollArea>
              </Card>
            </div>

            {/* Validation Results */}
            {validation && (
              <Card className={cn('p-4', validation.is_valid ? 'border-success' : 'border-warning')}>
                <h4 className="font-semibold flex items-center gap-2 mb-2">
                  {validation.is_valid ? (
                    <>
                      <CheckCircle2 className="h-4 w-4 text-success" />
                    </>
                  ) : (
                    <>
                      <AlertTriangle className="h-4 w-4 text-warning" />
                      {tGen('step4.validationWarnings')}
                    </>
                  )}
                </h4>
                {validation.issues.length > 0 && (
                  <ul className="list-disc list-inside text-sm text-error mb-2">
                    {validation.issues.map((issue, i) => (
                      <li key={i}>{issue}</li>
                    ))}
                  </ul>
                )}
                {validation.warnings.length > 0 && (
                  <ul className="list-disc list-inside text-sm text-warning">
                    {validation.warnings.map((warning, i) => (
                      <li key={i}>{warning}</li>
                    ))}
                  </ul>
                )}
              </Card>
            )}

            {/* Enable Switch */}
            <div className="flex items-center space-x-2">
              <Switch
                id="enable"
                checked={enableAfterCreate}
                onCheckedChange={setEnableAfterCreate}
              />
              <Label htmlFor="enable">{tGen('step4.enableImmediately')}</Label>
            </div>
          </div>
        )

      default:
        return null
    }
  }

  // Calculate step states
  const stepOrder: StepValue[] = ['samples', 'metrics', 'commands', 'preview']
  const currentIndex = stepOrder.indexOf(currentStep)
  const isFirstStep = currentIndex === 0
  const isLastStep = currentIndex === stepOrder.length - 1

  // Handle dialog close with reset
  const handleOpenChange = (open: boolean) => {
    if (!open && !analyzing && !creating) {
      // Reset state on close
      setSampleData('')
      setGenerated(null)
      setDeviceTypeName('')
      setDeviceTypeId('')
      setCurrentStep('samples')
      setValidation(null)
    }
    onOpenChange(open)
  }

  // Convert steps to VerticalStepper format
  const stepperSteps: StepperStep[] = stepsConfig.map(step => ({
    id: step.key,
    label: step.label,
    shortLabel: step.shortLabel,
    icon: step.icon,
  }))

  return (
    <FullScreenDialog
      open={open}
      onOpenChange={handleOpenChange}
    >
      {/* Header */}
      <FullScreenDialogHeader
        icon={<Sparkles className="h-5 w-5" />}
        iconBg="bg-accent-purple-light"
        iconColor="text-accent-purple"
        title={tGen('title')}
        subtitle={tGen('description')}
        onClose={() => handleOpenChange(false)}
      />

      {/* Content with Sidebar */}
      <FullScreenDialogContent>
        {/* Left Sidebar - Vertical Steps - Hide on mobile */}
        <FullScreenDialogSidebar>
          <VerticalStepper
            steps={stepperSteps}
            currentStep={currentStep}
            completedSteps={Array.from(completedSteps)}
            onStepClick={(stepId) => {
              // Allow navigating to completed steps or previous steps
              const clickedIndex = stepsConfig.findIndex(s => s.key === stepId)
              if (completedSteps.has(stepId) || clickedIndex < currentIndex) {
                setCurrentStep(stepId as StepValue)
              }
            }}
          />
        </FullScreenDialogSidebar>

        {/* Main Content */}
        <FullScreenDialogMain>
          <div className={cn(
            "max-w-4xl mx-auto",
            isMobile ? "px-4 py-4" : "px-4 py-6"
          )}>
            {renderStepContent()}
          </div>
        </FullScreenDialogMain>
      </FullScreenDialogContent>

      {/* Footer */}
      <FullScreenDialogFooter>
        {!isFirstStep && (
          <Button
            variant="outline"
            size={isMobile ? "default" : "sm"}
            onClick={handlePrevious}
            disabled={analyzing || creating}
            className={isMobile ? "h-12 min-w-[100px]" : ""}
          >
            <ChevronLeft className={cn(isMobile ? "h-4 w-4" : "h-4 w-4", "mr-1")} />
            {t('common:previous')}
          </Button>
        )}

        <div className="flex-1" />

        {!isLastStep ? (
          <Button
            size={isMobile ? "default" : "sm"}
            onClick={handleNext}
            disabled={!generated || analyzing}
            className={isMobile ? "h-12 min-w-[100px]" : ""}
          >
            {t('common:next')}
            <ChevronRight className={cn(isMobile ? "h-4 w-4" : "h-4 w-4", "ml-1")} />
          </Button>
        ) : (
          <Button
            size={isMobile ? "default" : "sm"}
            onClick={handleCreate}
            disabled={!deviceTypeName || !deviceTypeId || creating}
            className={isMobile ? "h-12 min-w-[100px]" : ""}
          >
            {creating ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : null}
            {creating ? t('common:creating') : tGen('step4.create')}
          </Button>
        )}
      </FullScreenDialogFooter>
    </FullScreenDialog>
  )
}
