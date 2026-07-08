// ========== Data Push Types ==========

export type PushTargetType = 'webhook' | 'mqtt'
export type DeliveryStatus = 'pending' | 'success' | 'failed' | 'retrying'
export type PushScheduleType = 'event_driven' | 'interval'

export interface PushSchedule {
  type: PushScheduleType
  event_types?: string[]
  interval_secs?: number
}

export interface DataSourceFilter {
  source_patterns: string[]
  only_changes: boolean
}

export interface RetryConfig {
  max_retries: number
  backoff_secs: number
  max_backoff_secs: number
}

export interface BatchConfig {
  batch_size: number
  batch_interval_ms: number
}

export interface PushTarget {
  id: string
  name: string
  enabled: boolean
  target_type: PushTargetType
  config: Record<string, any>
  schedule: PushSchedule
  data_filter: DataSourceFilter
  template?: string
  retry_config: RetryConfig
  batch_config: BatchConfig
  created_at: number
  updated_at: number
}

export interface DeliveryLog {
  id: string
  target_id: string
  status: DeliveryStatus
  data_source_id: string
  payload_sent: string
  response?: string
  attempts: number
  created_at: number
  completed_at?: number
  error?: string
}

export interface PushStats {
  total_targets: number
  active_targets: number
  total_deliveries: number
  successful_deliveries: number
  failed_deliveries: number
}

export interface CreatePushTargetRequest {
  name: string
  target_type: PushTargetType
  config: Record<string, any>
  schedule: PushSchedule
  data_filter: DataSourceFilter
  template?: string
  enabled?: boolean
  retry_config?: RetryConfig
  batch_config?: BatchConfig
}

export interface UpdatePushTargetRequest {
  name?: string
  target_type?: PushTargetType
  config?: Record<string, any>
  schedule?: PushSchedule
  data_filter?: DataSourceFilter
  template?: string
  enabled?: boolean
  retry_config?: RetryConfig
  batch_config?: BatchConfig
}
