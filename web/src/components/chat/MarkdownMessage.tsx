import React from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import type { Components } from 'react-markdown'
import { cn } from "@/lib/utils"
import { ErrorBoundary } from "@/components/shared/ErrorBoundary"
import { textBody, textCode, textHeading } from "@/design-system/tokens/typography"

interface MarkdownMessageProps {
  content: string
  className?: string
  variant?: 'user' | 'assistant'
}

/**
 * Remove duplicated content when the same text appears twice in one message
 * (e.g. model repetition or backend sending same chunk twice).
 */
function dedupeRepeatedContent(content: string): string {
  const s = (content || '').trim()
  if (s.length < 2) return content
  const half = Math.floor(s.length / 2)
  const first = s.slice(0, half)
  const second = s.slice(half)
  if (first === second) return first
  return content
}

// Static component overrides — hoisted to module scope to avoid re-allocating
// a new object (and new closure functions) on every render / streaming chunk.
const MARKDOWN_COMPONENTS: Components = {
  pre: ({ node, className, children, ...props }) => (
    <pre className={cn("overflow-x-auto", className)} {...(props as any)}>
      {children}
    </pre>
  ),
  code: ({ node, className, children, ...props }) => {
    const isBlock = !!className
    if (!isBlock) {
      return (
        <code className={cn("bg-muted px-1 py-0.5 rounded", textCode, "font-mono text-foreground", className)} {...(props as any)}>
          {children}
        </code>
      )
    }
    return (
      <code className={cn("text-foreground", className)} {...(props as any)}>
        {children}
      </code>
    )
  },
  a: ({ node, className, children, href, ...props }) => (
    <a
      // Inherit color from the surrounding text so links stay readable on
      // every bubble background. In light theme `--primary` is near-white,
      // which collides with the user bubble's `--msg-user-bg` (also
      // near-white) — hardcoding `text-primary` here made user-message URLs
      // invisible. Rely on underline for link affordance instead of color.
      className={cn("text-inherit underline underline-offset-2 hover:opacity-80", className)}
      href={href as string}
      target="_blank"
      rel="noopener noreferrer"
      {...(props as any)}
    >
      {children}
    </a>
  ),
}

/**
 * Markdown message renderer with support for:
 * - GitHub Flavored Markdown (GFM) via remark-gfm
 * - Code blocks with syntax highlighting
 * - Tables, lists, links, images
 * - Styled for chat interface
 * - Auto-scrolls during streaming, fully expands after
 *
 * Memoized: only re-renders when content/className/variant change.
 * Component overrides are module-scope (no per-render allocation).
 * Wrapped in ErrorBoundary so malformed markdown can't crash the chat.
 */
export const MarkdownMessage = React.memo<MarkdownMessageProps>(
  ({ content, className, variant = 'assistant' }) => {
  const displayContent = dedupeRepeatedContent(content)

  return (
    <div className={cn("relative", className)}>
      <div
        className={cn(
          // Base prose classes — use prose for structure, override size to 13px
          "prose max-w-none", textBody,
          // Text wrapping
          "break-words overflow-wrap-anywhere",
          "prose-p:leading-relaxed prose-p:my-1",
          "prose-headings:font-semibold prose-headings:my-2",
          "prose-h1:text-[15px] prose-h2:text-[13px] prose-h3:text-[12px]",
          // Links inherit text color (see MARKDOWN_COMPONENTS.a) so they stay
          // readable on both user and assistant bubble backgrounds. Underline
          // alone provides the link affordance — do NOT set a prose-a color
          // here, it would override the inherit and re-introduce the
          // light-theme invisibility bug on user bubbles.
          "prose-a:text-inherit prose-a:underline prose-a:underline-offset-2",
          "prose-strong:font-semibold",
          "prose-code:rounded prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:text-[12px] prose-code:font-mono",  // text-[12px] kept for Tailwind prose modifier
          "prose-code:break-all prose-code:whitespace-pre-wrap",
          "prose-pre:bg-muted prose-pre:p-3 prose-pre:rounded-lg prose-pre:my-2",
          "prose-pre:overflow-x-auto prose-pre:max-w-full",
          "prose-pre:prose-code:bg-transparent prose-pre:prose-code:p-0 prose-pre:prose-code:text-foreground",
          "prose-blockquote:border-l-2 prose-blockquote:border-muted-foreground prose-blockquote:pl-3 prose-blockquote:italic",
          "prose-ul:my-1 prose-ul:pl-4 prose-ul:list-disc",
          "prose-ol:my-1 prose-ol:pl-4 prose-ol:list-decimal",
          "prose-li:my-0.5",
          "prose-table:my-2 prose-table:text-[12px]",  // text-[12px] kept for Tailwind prose modifier
          "prose-th:px-2 prose-th:py-1 prose-th:border prose-th:border-border prose-th:bg-muted-50",
          "prose-td:px-2 prose-td:py-1 prose-td:border prose-td:border-border",
          "prose-hr:my-2 prose-hr:border-border",
          "text-inherit"
          // Removed max height limit - messages now fully expand
        )}
        data-variant={variant}
      >
        <ErrorBoundary resetKey={displayContent}>
          <ReactMarkdown remarkPlugins={[remarkGfm]} components={MARKDOWN_COMPONENTS}>
            {displayContent}
          </ReactMarkdown>
        </ErrorBoundary>
      </div>
    </div>
  )
},
  (prev, next) =>
    prev.content === next.content &&
    prev.className === next.className &&
    prev.variant === next.variant
)

MarkdownMessage.displayName = "MarkdownMessage"
