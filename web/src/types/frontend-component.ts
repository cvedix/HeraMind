/**
 * Frontend Component Types
 *
 * Types for community marketplace dashboard components.
 * These components are installed from the marketplace and loaded dynamically.
 */

export interface SizeConstraints {
  min_w: number
  min_h: number
  default_w: number
  default_h: number
  max_w: number
  max_h: number
}

/**
 * Metadata for an installed frontend component
 * This is returned by the API and stored in the registry
 */
export interface FrontendComponentMeta {
  id: string
  name: string | Record<string, string>
  description: string | Record<string, string>
  icon: string
  category: string
  version: string
  author?: string
  size_constraints: SizeConstraints
  has_data_source: boolean
  max_data_sources?: number
  data_source_allowed_types?: string[]
  has_display_config: boolean
  has_actions: boolean
  has_device_binding?: boolean
  device_type_filter?: string[]
  config_schema?: {
    type: string
    properties: Record<string, any>
    required?: string[]
    ui_hints?: {
      field_order?: string[]
      visibility_rules?: Array<{
        field: string
        condition: string
        value: any
        then_show?: string[]
        then_hide?: string[]
      }>
    }
    [key: string]: unknown
  }
  default_config?: Record<string, unknown>
  variants?: string[]
  global_name: string
  export_name?: string
  installed_at: number
  /** Origin: `"local"` (CLI/upload) or `"marketplace"`. Undefined for legacy components. */
  source?: 'local' | 'marketplace'
}

/**
 * Entry in the component marketplace
 * This represents a component available for installation
 */
export interface MarketComponentEntry {
  id: string
  name: string | Record<string, string>
  description: string | Record<string, string>
  icon: string
  category: string
  version: string
  author?: string
  size_constraints: SizeConstraints
  has_data_source: boolean
  max_data_sources?: number
  has_display_config: boolean
  has_actions: boolean
  has_device_binding?: boolean
  device_type_filter?: string[]
  manifest_url: string
  bundle_url: string
}

/**
 * Component manifest for manual installation
 * This is uploaded by users when installing components manually
 */
export interface ComponentManifest {
  id: string
  name: string | Record<string, string>
  description: string | Record<string, string>
  icon?: string
  category?: string
  version?: string
  author?: string
  size_constraints: SizeConstraints
  has_data_source?: boolean
  max_data_sources?: number
  data_source_allowed_types?: string[]
  has_display_config?: boolean
  has_actions?: boolean
  has_device_binding?: boolean
  device_type_filter?: string[]
  config_schema?: {
    type: string
    properties: Record<string, any>
    required?: string[]
    ui_hints?: {
      field_order?: string[]
      visibility_rules?: Array<{
        field: string
        condition: string
        value: any
        then_show?: string[]
        then_hide?: string[]
      }>
    }
    [key: string]: unknown
  }
  default_config?: Record<string, unknown>
  variants?: string[]
  global_name: string
  export_name?: string
}
