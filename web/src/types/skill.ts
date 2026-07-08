export interface SkillSummary {
  id: string
  name: string
  category: string
  origin: string
  priority: number
  token_budget: number
  keywords: string[]
  body_length: number
}

export interface SkillDetail extends SkillSummary {
  tool_targets: ToolTargetInfo[]
  anti_trigger_keywords: string[]
  body: string
}

export interface ToolTargetInfo {
  tool: string
  actions: string[]
}

export interface SkillListResponse {
  skills: SkillSummary[]
  total: number
}

export interface MatchResult {
  skill_id: string
  skill_name: string
  score: number
  body_preview: string
}

export interface MatchTestResponse {
  query: string
  matches: MatchResult[]
}

export interface CreateSkillRequest {
  content: string
}
