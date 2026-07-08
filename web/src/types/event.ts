// ========== Events Types ==========

export interface Event {
  id: string
  event_type: string
  source: string
  data: Record<string, unknown>
  timestamp: number
  processed: boolean
}

// ========== Search Types ==========

export interface SearchResult {
  type: 'device' | 'rule' | 'alert'
  id: string
  title: string
  description?: string
  relevance_score?: number
}

export interface SearchSuggestion {
  text: string
  type: string
  count: number
}

// ========== Extended Events Types ==========

export interface EventSubscription {
  id: string
  event_types: string[]
  source?: string
  created_at: number
}
