// ========== Common API Types ==========

export interface ApiResponse<T> {
  success?: boolean
  data?: T
  error?: string
  message?: string
}

export type JsonSchema = {
  type?: string
  title?: string
  title_zh?: string
  description?: string
  description_zh?: string
  properties?: Record<string, JsonSchema>
  required?: string[]
  default?: unknown
  enum?: unknown[]
  minimum?: number
  maximum?: number
  format?: string
  x_secret?: boolean
  ui_hints?: {
    field_order?: string[]
    display_names?: Record<string, string>
    placeholders?: Record<string, string>
    help_texts?: Record<string, string>
    visibility_rules?: Array<{
      field: string
      condition: string
      value: unknown
      then_show?: string[]
      then_hide?: string[]
      then_disable?: string[]
    }>
  }
  items?: JsonSchema
  additionalProperties?: boolean | JsonSchema
}

// ============================================================================
// JSON Schema (Dashboard / Extension component configuration)
// ============================================================================

/**
 * JSON Schema definition
 */
export interface JSONSchema {
  type?: string
  properties?: Record<string, any>
  required?: string[]
  items?: JSONSchema
  enum?: (string | number | boolean)[]
  enumTitles?: string[]
  default?: unknown
  description?: string
  title?: string
  minimum?: number
  maximum?: number
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

/**
 * Plugin configuration schema (legacy - for backward compatibility)
 */
export interface PluginConfigSchema {
  type: 'object'
  properties: Record<string, {
    type: string
    description?: string
    enum?: string[]
    items?: any
    properties?: any
    required?: string[]
    default?: any
    minimum?: number
    maximum?: number
    format?: string
    secret?: boolean
    [key: string]: any
  }>
  required?: string[]
  ui_hints?: any
  [key: string]: any
}
