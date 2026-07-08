/**
 * Config schema registry.
 *
 * Replaces the giant switch-case in the old configSchemas.tsx with a
 * simple lookup table. Each built-in component type maps to a factory
 * function; unknown types fall through to the dynamic handler which
 * handles extension / community / custom components.
 */

import type { ComponentConfigSchema } from '@/components/dashboard/config/ComponentConfigBuilder'
import { makeUpdaters } from './helpers'
import type { SchemaContext, SchemaFactory } from './types'

// Built-in schemas
import { getValueCardSchema, getSparklineSchema, getProgressBarSchema, getLEDIndicatorSchema } from './builtIn/indicators'
import { getLineChartSchema, getAreaChartSchema, getBarChartSchema, getPieChartSchema } from './builtIn/charts'
import { getToggleSwitchSchema } from './builtIn/controls'
import { getImageDisplaySchema, getImageHistorySchema, getWebDisplaySchema, getMarkdownDisplaySchema, getVideoDisplaySchema } from './builtIn/display'
import { getMapDisplaySchema, getCustomLayerSchema } from './builtIn/spatial'
import { getAgentMonitorSchema, getAIAnalystSchema } from './builtIn/business'

// Dynamic (extension / community / custom)
import { getDynamicSchema } from './dynamic'

// Re-export types for backward compatibility
export type { SchemaContext } from './types'

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

const registry: Record<string, SchemaFactory> = {
  // Indicators
  'value-card': getValueCardSchema,
  'counter': getValueCardSchema,
  'metric-card': getValueCardSchema,
  'sparkline': getSparklineSchema,
  'progress-bar': getProgressBarSchema,
  'led-indicator': getLEDIndicatorSchema,

  // Charts
  'line-chart': getLineChartSchema,
  'area-chart': getAreaChartSchema,
  'bar-chart': getBarChartSchema,
  'pie-chart': getPieChartSchema,

  // Controls
  'toggle-switch': getToggleSwitchSchema,

  // Display
  'image-display': getImageDisplaySchema,
  'image-history': getImageHistorySchema,
  'web-display': getWebDisplaySchema,
  'markdown-display': getMarkdownDisplaySchema,
  'video-display': getVideoDisplaySchema,

  // Spatial
  'map-display': getMapDisplaySchema,
  'custom-layer': getCustomLayerSchema,

  // Business
  'agent-monitor-widget': getAgentMonitorSchema,
  'ai-analyst': getAIAnalystSchema,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export function generateConfigSchema(
  componentType: string,
  currentConfig: any,
  ctx: SchemaContext,
): ComponentConfigSchema | null {
  const config = currentConfig || {}
  const helpers = makeUpdaters(config, ctx)

  const factory = registry[componentType]
  if (factory) {
    return factory(config, ctx, helpers)
  }

  // Fallback: extension / community / custom
  return getDynamicSchema(componentType, config, ctx, helpers)
}
