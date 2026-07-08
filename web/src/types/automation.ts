// ========== Unified Automation Types ==========
// Must match backend AutomationDto and related types (crates/api/src/handlers/automations.rs)

import type { ExtensionTransformOperation } from './extension'

/**
 * Automation type enumeration
 */
export type AutomationType = 'transform'

/**
 * Transform scope - determines what data the transform applies to
 * Matches backend Rust enum format with serde(rename_all = "snake_case")
 * - Global: "global" (string)
 * - DeviceType: { device_type: string }
 * - Device: { device: string }
 */
export type TransformScope =
  | 'global'
  | { device_type: string }
  | { device: string }

/**
 * Aggregation function for transforms
 */
export type AggregationFunc =
  | 'mean'
  | 'max'
  | 'min'
  | 'sum'
  | 'count'
  | 'first'
  | 'last'
  | 'median'
  | 'stddev'
  | 'trend'
  | 'delta'
  | 'rate'

/**
 * Time window for time-series aggregation
 */
export interface TimeWindow {
  duration_secs: number
  aggregation: AggregationFunc
}

/**
 * Data type for value conversion
 */
export type DataType = 'string' | 'number' | 'boolean' | 'int' | 'float'

/**
 * Data decode/encode format
 */
export type DecodeFormat = 'hex' | 'base64' | 'bytes' | 'csv' | 'url'

/**
 * Transform operation types
 *
 * Expression-based operations that AI can understand and generate:
 * - Extract: Get data using JSONPath
 * - Map: Transform array elements using template (e.g., convert nested arrays to "cls:fish num:12")
 * - Reduce: Aggregate array to single value
 * - Format: Template string formatting
 * - Compute: Mathematical expressions
 * - Pipeline: Chain operations
 * - Fork: Parallel branches
 * - If: Conditional execution
 * - GroupBy: Group array by key and aggregate (e.g., [{box, cls}] → count by cls)
 * - Decode: Convert encoded data (hex/base64) to JSON
 * - Encode: Convert JSON to encoded format
 */
export type TransformOperation =
  // Legacy operations (for backward compatibility)
  | { op_type: 'single'; json_path: string; output_metric: string }
  | { op_type: 'array_aggregation'; json_path: string; aggregation: AggregationFunc; value_path?: string; output_metric: string }
  | { op_type: 'time_series_aggregation'; source_metric: string; window: TimeWindow; output_metric: string }
  | { op_type: 'reference'; source_device: string; source_metric: string; output_metric: string }
  | { op_type: 'multi_output'; operations: TransformOperation[] }
  // Expression-based operations
  | { op_type: 'extract'; from: string; output: string; as_type?: DataType }
  | { op_type: 'map'; over: string; template: string; output: string; where?: string }
  | { op_type: 'reduce'; over: string; using: AggregationFunc; value?: string; output: string }
  | { op_type: 'format'; template: string; output: string; from?: string }
  | { op_type: 'compute'; expression: string; output: string }
  | { op_type: 'pipeline'; steps: TransformOperation[]; output: string }
  | { op_type: 'fork'; branches: TransformOperation[] }
  | { op_type: 'if'; condition: string; then: TransformOperation; else_?: TransformOperation; output: string }
  // Advanced data processing operations
  | { op_type: 'group_by'; over: string; key: string; using: AggregationFunc; value?: string; output: string }
  | { op_type: 'decode'; from: string; format: DecodeFormat; output: string }
  | { op_type: 'encode'; from: string; format: DecodeFormat; output: string }

/**
 * Transform automation - data processing layer
 *
 * New AI-Native Design:
 * - intent: User's natural language description
 * - js_code: AI-generated JavaScript transformation code
 * - output_prefix: Prefix for generated metric names
 */
export interface TransformAutomation extends BaseAutomation {
  type: 'transform'
  scope: TransformScope
  device_type_filter?: string

  // New AI-Native fields
  intent?: string // User's natural language intent
  js_code?: string // AI-generated JavaScript code
  extension_operation?: ExtensionTransformOperation // Extension-based transform
  output_prefix: string // Prefix for output metrics (default: "transform")
  complexity: number // 1-5, for execution ordering

  // Legacy field (deprecated, use js_code instead)
  operations?: TransformOperation[]
}

/**
 * Unified Automation type - can be Transform or Rule
 */
export type Automation = TransformAutomation

/**
 * Base automation interface with common fields
 */
export interface BaseAutomation {
  id: string
  name: string
  description: string
  enabled: boolean
  type: AutomationType
  created_at: number
  updated_at: number
  execution_count: number
  last_executed: number | null
}

/**
 * Automation list response
 */
export interface AutomationListResponse {
  automations: Automation[]
  count: number
}

/**
 * Intent analysis result from AI
 */
export interface IntentResult {
  recommended_type: AutomationType
  confidence: number // 0-100
  reasoning: string
  suggested_automation: Partial<Automation> | null
  warnings: string[]
}

/**
 * Create automation request
 */
export interface CreateAutomationRequest {
  name: string
  description?: string
  type?: AutomationType
  enabled?: boolean
  definition: Record<string, unknown>
}

/**
 * Update automation request
 */
export interface UpdateAutomationRequest {
  name?: string
  description?: string
  definition?: Record<string, unknown>
  enabled?: boolean
}

/**
 * Set automation status request
 */
export interface SetAutomationStatusRequest {
  enabled: boolean
}

/**
 * Automation execution record
 */
export interface ExecutionRecord {
  id: string
  automation_id: string
  started_at: number
  completed_at: number | null
  status: 'running' | 'completed' | 'failed' | 'cancelled'
  result?: Record<string, unknown>
  error?: string
}

/**
 * Automation template
 */
export interface AutomationTemplate {
  id: string
  name: string
  description: string
  automation_type: AutomationType
  category: string
  parameters: TemplateParameter[]
  definition_template: Record<string, unknown>
}

/**
 * Template parameter for automation templates
 */
export interface TemplateParameter {
  name: string
  label: string
  param_type: 'string' | 'number' | 'boolean' | 'device' | 'metric' | 'enum'
  default?: string | null
  required: boolean
  options: string[]
}

/**
 * Automation filter parameters
 */
export interface AutomationFilter {
  type?: 'transform' | 'all'
  enabled?: boolean
  search?: string
}
