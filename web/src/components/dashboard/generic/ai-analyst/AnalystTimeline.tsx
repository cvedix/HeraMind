import { useRef, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { ScrollArea } from '@/components/ui/scroll-area'
import { CircleDot } from 'lucide-react'
import { AnalystMessageBubble } from './AnalystMessageBubble'
import type { AnalystMessage } from './types'

interface AnalystTimelineProps {
  messages: AnalystMessage[]
  streamingContent: string
  streamingMsgId: string | null
  contextWindowSize: number
}

export function AnalystTimeline({ messages, streamingContent, streamingMsgId, contextWindowSize }: AnalystTimelineProps) {
  const { t } = useTranslation('dashboardComponents')
  const viewportRef = useRef<HTMLDivElement>(null)

  // Auto-scroll to bottom on new messages (smart: only if near bottom).
  // IMPORTANT: do NOT use element.scrollIntoView() — it scrolls ALL ancestor
  // scroll containers (including the dashboard page itself), causing the whole
  // page to jump. Manually set scrollTop on the ScrollArea viewport only.
  useEffect(() => {
    const el = viewportRef.current
    if (!el) return
    const isNearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 100
    if (isNearBottom) {
      el.scrollTop = el.scrollHeight
    }
  }, [messages, streamingContent])

  // Apply context window
  const visibleMessages = messages.slice(-contextWindowSize * 2)

  if (messages.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-center h-full">
        <CircleDot className="h-8 w-8 text-muted-foreground opacity-30 mb-2" />
        <p className="text-xs text-muted-foreground">{t('aiAnalyst.noActivity')}</p>
      </div>
    )
  }

  return (
    <ScrollArea className="h-full" viewportRef={viewportRef}>
      <div className="p-3 space-y-2">
        {visibleMessages.map((msg) => (
          <AnalystMessageBubble
            key={msg.id}
            message={msg}
            streamingContent={msg.id === streamingMsgId ? streamingContent : undefined}
          />
        ))}
        {streamingMsgId && !messages.find((m) => m.id === streamingMsgId) && streamingContent && (
          <AnalystMessageBubble
            message={{
              id: streamingMsgId,
              type: 'ai',
              content: '',
              timestamp: Date.now(),
              isStreaming: true,
            }}
            streamingContent={streamingContent}
          />
        )}
      </div>
    </ScrollArea>
  )
}
