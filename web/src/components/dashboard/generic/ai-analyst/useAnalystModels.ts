import { useState, useEffect, useCallback } from 'react'
import { api } from '@/lib/api'
import type { AnalystModel } from './types'

export function useAnalystModels() {
  const [models, setModels] = useState<AnalystModel[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadModels = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const resp = await api.listLlmBackends()
      const backends = resp.backends || []
      const result: AnalystModel[] = []

      for (const backend of backends) {
        result.push({
          id: backend.id,
          name: backend.name || backend.model,
          backendId: backend.id,
          backendName: backend.name || backend.id,
          isMultimodal: backend.capabilities?.supports_multimodal ?? false,
        })
      }

      setModels(result)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load models')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    loadModels()
  }, [loadModels])

  return { models, loading, error, reload: loadModels }
}
