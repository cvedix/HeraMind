// ============================================================================
// Dashboard Components from Extensions
// ============================================================================

import type { JSONSchema } from './api'

/**
 * Component category for dashboard widgets
 */
export type ComponentCategory =
  | 'indicators' // Value displays (cards, indicators)
  | 'charts' // Charts and graphs
  | 'controls' // Interactive inputs
  | 'display' // Content display
  | 'spatial' // Maps, video, layers
  | 'business' // Business-specific components
  | 'custom' // Extension-provided custom components

/**
 * Size constraints for dashboard components
 */
export interface SizeConstraints {
  min_w: number
  min_h: number
  default_w: number
  default_h: number
  max_w: number
  max_h: number
  preserve_aspect?: boolean
}

/**
 * Data binding configuration for extension components
 */
export interface DataBindingConfig {
  extension_metric?: string
  extension_command?: string
  required_fields: string[]
}

/**
 * Dashboard component DTO from extension
 */
export interface DashboardComponentDto {
  /** Component type identifier */
  type: string
  /** Display name */
  name: string
  /** Description */
  description: string
  /** Component category */
  category: ComponentCategory
  /** Icon name (lucide-react) */
  icon?: string
  /** Bundle URL (resolved) */
  bundle_url: string
  /** Export name in bundle */
  export_name: string
  /** Size constraints */
  size_constraints: SizeConstraints
  /** Whether this component accepts a data source */
  has_data_source: boolean
  /** Whether this component has display configuration */
  has_display_config: boolean
  /** Whether this component has actions */
  has_actions: boolean
  /** Maximum number of data sources */
  max_data_sources: number
  /** Allowed data source types for binding */
  data_source_allowed_types?: string[]
  /** Whether this component supports device binding (receives deviceContext) */
  has_device_binding?: boolean
  /** JSON Schema for component configuration */
  config_schema?: JSONSchema
  /** JSON Schema for data source binding */
  data_source_schema?: JSONSchema
  /** Default configuration values */
  default_config?: Record<string, unknown>
  /** Component variants */
  variants: string[]
  /** Data binding configuration */
  data_binding: DataBindingConfig
  /** Extension ID */
  extension_id: string
  /** Global variable name for IIFE bundles */
  global_name?: string
}

/**
 * Dashboard components list response
 */
export interface DashboardComponentsResponse {
  extension_id: string
  extension_name: string
  components: DashboardComponentDto[]
}
