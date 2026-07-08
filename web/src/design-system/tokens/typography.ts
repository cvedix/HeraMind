/**
 * Typography Tokens
 *
 * Semantic font size presets for consistent typography across the app.
 * Use these Tailwind class strings instead of hardcoded pixel values.
 *
 * Standard Tailwind sizes (for reference):
 *   text-xs = 12px, text-sm = 14px, text-base = 16px, text-lg = 18px
 */

// Micro: extreme micro labels, data type labels, execution step indicators
export const textMicro = 'text-[9px]'

// Nano: timestamps, tiny metadata, compact badges
export const textNano = 'text-[10px]'

// Mini: badge text, secondary labels, tab labels
export const textMini = 'text-[11px]'

// Body: chat messages, tool call text, markdown body
export const textBody = 'text-[13px]'

// Code inline: inline code in markdown, code snippets
export const textCode = 'text-[12px]'

// Heading: markdown headings
export const textHeading = 'text-[15px]'

// Font family
export const fontMonoClass = 'font-mono'

// Monospace font stack for inline styles (e.g. CodeMirror, Recharts)
export const fontMonoStack =
  'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace'

/**
 * Badge text size presets
 */
export const badgeSize = {
  micro: 'text-[9px]',   // data type badges in execution details
  small: 'text-[10px]',  // status badges, data type badges
  default: 'text-[11px]', // standard badges
} as const
