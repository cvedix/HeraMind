import { fetchAPI } from './client'

export type OcrLanguage = 'vie' | 'eng' | 'vie_eng'
export type OcrEngine = 'tesseract' | 'paddle_ocr_vl'
export type OcrJobStatus = 'queued' | 'running' | 'completed' | 'failed'

export interface OcrStatus {
  engine: OcrEngine
  engine_ready: boolean
  model_loaded?: boolean
  model_version?: string
  languages: OcrLanguage[]
  max_bytes: number
  max_pages: number
  retention_seconds: number
}

export interface OcrJob {
  id: string
  owner: string
  filename: string
  language: OcrLanguage
  engine: OcrEngine
  status: OcrJobStatus
  created_at: number
  pages_done: number
  page_count: number | null
  error_code: string | null
}

export interface OcrResult {
  job: OcrJob
  pages: { number: number; text: string }[]
}

export interface OcrKey {
  id: string
  name: string
  created_at: number
}

export type OcrRequestRoute = 'status' | 'submit' | 'list_jobs' | 'get_job'

export interface OcrRequestSample {
  sequence: number
  at_ms: number
  duration_ms: number
  route: OcrRequestRoute
  authentication: 'ocr_key' | 'session' | 'none'
  status: number
  job_id: string | null
}

export interface OcrMonitorSnapshot {
  generated_at_ms: number
  observed_since_ms: number
  max_requests: number
  requests: OcrRequestSample[]
  jobs: (Omit<OcrJob, 'owner' | 'filename'> & { finished_at: number | null })[]
}

const localError = { skipErrorToast: true, skipRetry: true }

export const ocrApi = {
  monitor: (signal?: AbortSignal) => fetchAPI<OcrMonitorSnapshot>('/ocr/monitor', { ...localError, signal }),
  status: (engine: OcrEngine, signal?: AbortSignal) => fetchAPI<OcrStatus>(`/ocr/status?engine=${engine}`, { ...localError, signal }),
  jobs: (signal?: AbortSignal) => fetchAPI<OcrJob[]>('/ocr/jobs', { ...localError, signal }),
  result: (id: string, signal?: AbortSignal) => fetchAPI<OcrResult>(`/ocr/jobs/${id}`, { ...localError, signal }),
  submit: (file: File, language: OcrLanguage, engine: OcrEngine) => {
    const query = new URLSearchParams({ filename: file.name, language, engine })
    return fetchAPI<OcrJob>(`/ocr/jobs?${query}`, {
      ...localError,
      method: 'POST',
      headers: { 'Content-Type': 'application/pdf' },
      body: file,
    })
  },
  keys: (signal?: AbortSignal) => fetchAPI<OcrKey[]>('/ocr/keys', { ...localError, signal }),
  issue: (name: string) => fetchAPI<{ key: OcrKey; token: string }>('/ocr/keys', {
    ...localError,
    method: 'POST',
    body: JSON.stringify({ name }),
  }),
  revoke: (id: string) => fetchAPI<boolean>(`/ocr/keys/${id}`, { ...localError, method: 'DELETE' }),
}
