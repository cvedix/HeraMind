// ========== Tool Types ==========

export interface Tool {
  name: string
  description?: string
  parameters?: Record<string, unknown>
  category?: string
}

export interface ToolSchema {
  name: string
  description: string
  parameters: {
    type: string
    properties: Record<string, { type: string; description: string }>
    required?: string[]
  }
}

export interface ToolMetrics {
  total_calls: number
  successful_calls: number
  failed_calls: number
  average_execution_time_ms: number
  by_tool: Record<string, { calls: number; avg_time_ms: number }>
}

export interface ToolExecutionResult {
  success: boolean
  result: unknown
  error?: string
  execution_time_ms: number
}
