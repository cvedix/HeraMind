import React from "react"
import { MessageSquare } from "lucide-react"
import { cn } from "@/lib/utils"

interface SessionListItemIconProps {
  session: {
    sessionId?: string | null
    title?: string | null
    preview?: string
  }
  isActive: boolean
  onSwitch: (id: string) => void
  getDisplayName: (session: { title?: string | null; preview?: string }) => string
}

/**
 * Memoized icon-mode session list item component.
 * Only re-renders when session.id, title, preview, or isActive changes.
 */
export const SessionListItemIcon = React.memo<SessionListItemIconProps>(
  ({ session, isActive, onSwitch, getDisplayName }) => {
    const handleClick = () => {
      if (session.sessionId) {
        onSwitch(session.sessionId)
      }
    }

    return (
      <button
        onClick={handleClick}
        className={cn(
          "group relative w-full flex items-center justify-center rounded-lg p-2 transition-colors",
          isActive
            ? "bg-primary text-primary-foreground"
            : "hover:bg-muted"
        )}
        title={getDisplayName(session)}
      >
        <MessageSquare className="h-4 w-4 shrink-0" />
      </button>
    )
  },
  (prev, next) => {
    // Custom comparison: only re-render if these specific props change
    return (
      prev.session.sessionId === next.session.sessionId &&
      prev.session.title === next.session.title &&
      prev.session.preview === next.session.preview &&
      prev.isActive === next.isActive
    )
  }
)

SessionListItemIcon.displayName = "SessionListItemIcon"
