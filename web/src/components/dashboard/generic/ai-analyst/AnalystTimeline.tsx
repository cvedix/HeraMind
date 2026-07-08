import { useRef, useEffect } from 'react'
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
  const bottomRef = useRef<HTMLDivElement>(null)
  const viewportRef = useRef<HTMLDivElement>(null)

  // Auto-scroll to bottom on new messages (smart: only if near bottom)
  useEffect(() => {
    const el = viewportRef.current
    if (el) {
      const isNearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 100
      if (isNearBottom) {
        bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
      }
    }
  }, [messages, streamingContent])

  // Apply context window
  const visibleMessages = messages.slice(-contextWindowSize * 2)

  if (messages.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-center h-full">
        <CircleDot className="h-8 w-8 text-muted-foreground opacity-30 mb-2" />
        <p className="text-xs text-muted-foreground">No activity yet</p>
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
        <div ref={bottomRef} />
      </div>
    </ScrollArea>
  )
}
