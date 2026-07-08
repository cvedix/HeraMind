// ========== Memory Types ==========

export interface MemoryEntry {
  id: string
  content: string
  importance: number
  created_at: number
  accessed_count?: number
  last_accessed?: number
  embedding?: number[]
}

export interface MemoryConsolidation {
  consolidated_count: number
  remaining_count: number
  message: string
}

// ========== System Memory Configuration Types ==========

export interface MemorySystemConfig {
  enabled: boolean
  storage_path: string
  user_char_limit: number
  knowledge_char_limit: number
  procedures_char_limit: number
  agent_char_limit: number
  temp_file_ttl_days: number
  system_context_interval_secs: number
  summary_interval_secs: number
  summary_backend_id: string | null
}
