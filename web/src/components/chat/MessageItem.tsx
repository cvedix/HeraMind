import React from "react"
import { type Message, type UserInfo } from "@/types"
import { ThinkingBlock } from "./ThinkingBlock"
import { ToolProcessBlock } from "./ToolCallVisualization"
import { MarkdownMessage } from "./MarkdownMessage"
import { Avatar, AvatarFallback } from "@/components/ui/avatar"
import { Bot } from "lucide-react"
import { formatTimestamp } from "@/lib/utils/format"

interface MessageItemProps {
  message: Message
  user: UserInfo | null
  getUserInitials: (username: string) => string
}

/**
 * Memoized message item component.
 * Only re-renders when message.id, content, thinking, tool_calls, or role changes.
 */
export const MessageItem = React.memo<MessageItemProps>(
  ({ message, user, getUserInitials }) => {
    const isAssistant = message.role === "assistant"

    return (
      <div
        className={`flex gap-3 items-start animate-fade-in-up ${
          isAssistant ? "justify-start" : "justify-end"
        }`}
      >
        {isAssistant && (
          <div className="flex-shrink-0 w-8 h-8 rounded-lg bg-accent-orange-bg flex items-center justify-center">
            <Bot className="h-4.5 w-4.5 text-accent-orange" />
          </div>
        )}

        {isAssistant ? (
          <div className="flex-1 min-w-0">
            {/* Thinking block */}
            {message.thinking && (
              <ThinkingBlock thinking={message.thinking} />
            )}

            {/* Tool calls */}
            {message.tool_calls && message.tool_calls.length > 0 && (
              <ToolProcessBlock
                toolCalls={message.tool_calls}
                roundContents={message.round_contents}
                isStreaming={false}
              />
            )}

            {/* Content */}
            {message.content && (
              <MarkdownMessage content={message.content} variant="assistant" className="px-3" />
            )}

            {/* Timestamp */}
            <p className="text-xs text-muted-foreground mt-1.5 px-3">
              {formatTimestamp(message.timestamp, false)}
            </p>
          </div>
        ) : (
          /* User: keep bubble with natural width */
          <div className="max-w-[80%] order-1">
            <div className="rounded-2xl px-4 py-3 overflow-hidden bg-[var(--msg-user-bg)] text-[var(--msg-user-text)]">
              {message.images && message.images.length > 0 && (
                <MessageImages images={message.images} />
              )}
              {message.content && (
                <MarkdownMessage content={message.content} variant="user" />
              )}
            </div>
            <p className="text-xs text-muted-foreground mt-1 px-1">
              {formatTimestamp(message.timestamp, false)}
            </p>
          </div>
        )}

        {message.role === "user" && user && (
          <Avatar className="h-8 w-8 order-2">
            <AvatarFallback className="bg-accent-orange text-white text-xs">
              {getUserInitials(user.username)}
            </AvatarFallback>
          </Avatar>
        )}
      </div>
    )
  },
  (prev, next) => {
    // Custom comparison: only re-render if these specific props change
    return (
      prev.message.id === next.message.id &&
      prev.message.content === next.message.content &&
      prev.message.thinking === next.message.thinking &&
      prev.message.tool_calls === next.message.tool_calls &&
      prev.message.role === next.message.role &&
      prev.message.images === next.message.images
    )
  }
)

MessageItem.displayName = "MessageItem"

/** Image gallery component for user messages */
function MessageImages({ images }: { images: any[] }) {
  if (!images || images.length === 0) return null

  return (
    <div className={images.length === 1 ? "mb-2" : "mb-2 grid grid-cols-2 gap-2"}>
      {images.map((img, idx) => (
        <img
          key={idx}
          src={img.data}
          alt={`Image ${idx + 1}`}
          className="rounded-lg max-w-full max-h-64 object-cover"
          loading="lazy"
        />
      ))}
    </div>
  )
}
