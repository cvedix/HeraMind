import { useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, TestTube } from 'lucide-react'
import { Button } from '@/components/ui/button'
import type { BackendTestResult } from '@/types'

export function BackendConnectionTest({ onTest }: { onTest: () => Promise<BackendTestResult> }) {
  const { t } = useTranslation('plugins')
  const [pending, setPending] = useState(false)
  const [result, setResult] = useState<BackendTestResult | null>(null)
  const running = useRef(false)

  const test = async () => {
    if (running.current) return
    running.current = true
    setPending(true)
    setResult(null)
    try {
      setResult(await onTest())
    } catch (error) {
      setResult({ success: false, error: error instanceof Error ? error.message : String(error) })
    } finally {
      running.current = false
      setPending(false)
    }
  }

  return (
    <div className="space-y-2">
      <Button type="button" variant="secondary" size="sm" onClick={() => void test()} disabled={pending}>
        {pending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <TestTube className="mr-2 h-4 w-4" />}
        {t(pending ? 'llm.testResponsePending' : 'llm.testResponse')}
      </Button>
      {result && (
        <div role={result.success ? 'status' : 'alert'} className={`rounded p-2 text-xs ${result.success ? 'bg-success-light text-success' : 'bg-error-light text-error'}`}>
          {result.success ? (
            <>
              <p>{t('llm.testResponseSuccess')} · {t('llm.latency')}: {result.latency_ms?.toFixed(0) ?? '—'}ms</p>
              <p className="mt-2 whitespace-pre-wrap break-words text-foreground">{result.response || t('llm.testResponseUnavailable')}</p>
            </>
          ) : <p className="whitespace-pre-wrap break-words">{result.error || t('llm.testResponseFailed')}</p>}
        </div>
      )}
    </div>
  )
}
