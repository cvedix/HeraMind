import React from 'react'
import { cn } from '@/lib/utils'
import { chartColorsHex } from '@/design-system/tokens/color'
import { Field } from '@/components/ui/field'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { ColorPicker } from '@/components/ui/color-picker'
import { IconPicker } from '@/components/ui/icon-picker'
import { EntityIconPicker } from '@/components/ui/entity-icon-picker'
import { DataMappingConfig } from '@/components/dashboard/config/UIConfigSections'
import { LEDStateRulesConfig } from '@/components/dashboard/config/LEDStateRulesConfig'
import type { StateRule } from '@/components/dashboard/generic/LEDIndicator'
import type { SingleValueMappingConfig, TimeSeriesMappingConfig, CategoricalMappingConfig } from '@/lib/dataMapping'
import { DualModeSourceField } from '@/components/dashboard/config'
import type { ComponentConfigSchema } from '@/components/dashboard/config/ComponentConfigBuilder'
import { SelectField } from '../../ConfigFieldComponents'
import type { SchemaContext, Updaters } from '../types'
import { useStore } from '@/store'

export function getAgentMonitorSchema(config: any, ctx: SchemaContext, u: Updaters): ComponentConfigSchema {
  const { t, agents, agentsLoading } = ctx
  const { updateConfig, updateDataSource } = u
  return {
          displaySections: [
            {
              type: 'custom' as const,
              render: () => {
                // Read from componentConfig which is kept up-to-date by updateDataSource
                const currentAgentId = (config.dataSource as any)?.agentId || ''
                // Use agents directly from component state (loaded by the agents loading effect)
                const agentsList = agents
                
                return (
                  <div className="space-y-3">
                    <Field>
                      <Label>{t('dashboardComponents:agentMonitorWidget.selectAgent')}</Label>
                      <Select
                        value={currentAgentId}
                        onValueChange={(value) => {
                          
                          updateDataSource({ type: 'agent', agentId: value })
                        }}
                        disabled={agentsLoading}
                      >
                        <SelectTrigger className="h-9">
                          <SelectValue placeholder={agentsLoading ? t('common:loading') : t('dashboardComponents:agentMonitorWidget.selectAgent')} />
                        </SelectTrigger>
                        <SelectContent>
                          {agentsList.map((agent: any) => (
                            <SelectItem key={agent.id} value={agent.id}>
                              {agent.name}
                            </SelectItem>
                          ))}
                          {agentsList.length === 0 && !agentsLoading && (
                            <div className="px-2 py-4 text-center text-sm text-muted-foreground">
                              {t('agents:noAgents')}
                            </div>
                          )}
                        </SelectContent>
                      </Select>
                    </Field>
                  </div>
                )
              },
            },
          ],
        }

}

export function getAIAnalystSchema(config: any, ctx: SchemaContext, u: Updaters): ComponentConfigSchema {
  const { t, agents, visionModels, visionModelsLoading } = ctx
  const { updateConfig, updateDataSource } = u
  return {
          dataSourceSections: [
            {
              type: 'data-source' as const,
              props: {
                dataSource: config.dataSource,
                onChange: updateDataSource,
                allowedTypes: ['device-metric', 'extension', 'command', 'extension-command'],
                multiple: true,
              },
            },
          ],
          displaySections: [
            {
              type: 'custom' as const,
              render: () => {
                const modelsList = visionModels
                return (
                  <div className="space-y-3">
                    <Field>
                      <Label>{t('dashboardComponents:aiAnalyst.selectModel')}</Label>
                      <Select
                        value={config.modelId || ''}
                        onValueChange={(value) => updateConfig('modelId')(value)}
                        disabled={visionModelsLoading}
                      >
                        <SelectTrigger className="h-9">
                          <SelectValue placeholder={visionModelsLoading ? t('common:loading') : t('dashboardComponents:aiAnalyst.selectModelPlaceholder')} />
                        </SelectTrigger>
                        <SelectContent>
                          {modelsList.map((model: any) => (
                            <SelectItem key={model.id} value={model.id}>
                              <div className="flex items-center gap-2">
                                <span>{model.name}</span>
                                <span className="text-xs text-muted-foreground">({model.backendName})</span>
                              </div>
                            </SelectItem>
                          ))}
                          {modelsList.length === 0 && !visionModelsLoading && (
                            <div className="px-2 py-4 text-center text-sm text-muted-foreground">
                              {t('dashboardComponents:aiAnalyst.noModels')}
                            </div>
                          )}
                        </SelectContent>
                      </Select>
                    </Field>
                    <Field>
                      <Label>{t('dashboardComponents:aiAnalyst.systemPrompt')}</Label>
                      <Textarea
                        value={config.systemPrompt || ''}
                        onChange={(e) => updateConfig('systemPrompt')(e.target.value)}
                        placeholder={t('dashboardComponents:aiAnalyst.systemPromptPlaceholder')}
                        className="resize-y"
                      />
                    </Field>
                    <Field>
                      <Label>{t('dashboardComponents:aiAnalyst.contextWindow')}</Label>
                      <Input
                        type="number"
                        min={1}
                        max={100}
                        value={config.contextWindowSize || 10}
                        onChange={(e) => updateConfig('contextWindowSize')(Number(e.target.value) || 10)}
                        className="h-9"
                      />
                    </Field>
                  </div>
                )
              },
            },
          ],
        }
}
