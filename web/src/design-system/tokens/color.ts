/**
 * Design Tokens - Color
 *
 * Unified color system for all dashboard components.
 * Uses OKLCH for perceptual uniformity.
 */

// Chart colors — blue/white/black product palette with distinct lightness
// and chroma steps for visual hierarchy and accessibility.
export const chartColors = {
  1: 'oklch(0.52 0.23 255)',   // HeraMind blue — primary series
  2: 'oklch(0.61 0.20 245)',   // Royal blue
  3: 'oklch(0.70 0.14 235)',   // Sky blue
  4: 'oklch(0.38 0.08 255)',   // Navy
  5: 'oklch(0.56 0.06 255)',   // Slate blue
  6: 'oklch(0.76 0.09 225)',   // Ice blue
} as const

export type ChartColor = keyof typeof chartColors

// Hex equivalents for SVG rendering (Recharts needs hex, not OKLCH)
// Hex approximations for SVG rendering (Recharts needs hex, not OKLCH)
export const chartColorsHex = [
  '#005bea', // HeraMind blue (chartColors[1])
  '#2563eb', // Royal blue    (chartColors[2])
  '#38bdf8', // Sky blue      (chartColors[3])
  '#0f172a', // Navy          (chartColors[4])
  '#64748b', // Slate blue    (chartColors[5])
  '#93c5fd', // Ice blue      (chartColors[6])
] as const

// Status colors — kept inside the product palette while preserving contrast
export const statusColors = {
  success: 'oklch(0.55 0.21 250)',      // Blue success
  warning: 'oklch(0.36 0.06 255)',      // Navy warning
  error: 'oklch(0.48 0.18 265)',        // Deep blue error
  info: 'oklch(0.52 0.23 255)',         // HeraMind blue
  neutral: 'oklch(0.55 0.02 255)',      // Cool gray
} as const

export type StatusColor = keyof typeof statusColors

// Status colors with opacity for backgrounds
export const statusBgColors = {
  success: 'oklch(0.55 0.21 250 / 0.15)',
  warning: 'oklch(0.36 0.06 255 / 0.12)',
  error: 'oklch(0.48 0.18 265 / 0.14)',
  info: 'oklch(0.52 0.23 255 / 0.15)',
  neutral: 'oklch(0.55 0.02 255 / 0.1)',
} as const

// Semantic color mappings
export const semanticColors = {
  // Device states
  online: statusColors.success,
  offline: statusColors.neutral,
  error: statusColors.error,
  unknown: statusColors.neutral,

  // Agent states
  idle: statusColors.neutral,
  running: statusColors.info,
  paused: statusColors.warning,
  completed: statusColors.success,
  failed: statusColors.error,

  // Trend directions
  up: 'oklch(0.55 0.21 250)',       // Blue positive
  down: 'oklch(0.38 0.08 255)',     // Navy negative
  neutral: statusColors.neutral,
} as const

export type SemanticColor = keyof typeof semanticColors

// CSS custom properties (for global styles)
export const cssVars = {
  // Chart colors
  '--color-chart-1': chartColors[1],
  '--color-chart-2': chartColors[2],
  '--color-chart-3': chartColors[3],
  '--color-chart-4': chartColors[4],
  '--color-chart-5': chartColors[5],
  '--color-chart-6': chartColors[6],

  // Status colors
  '--color-status-success': statusColors.success,
  '--color-status-warning': statusColors.warning,
  '--color-status-error': statusColors.error,
  '--color-status-info': statusColors.info,
  '--color-status-neutral': statusColors.neutral,
} as const

// Helper to get chart color by index
export function getChartColor(index: number): string {
  return chartColors[(index % Object.keys(chartColors).length + 1) as ChartColor]
}

// Helper to get status color with opacity
export function getStatusColor(
  status: keyof typeof statusColors,
  opacity: number = 1
): string {
  const color = statusColors[status]
  if (opacity < 1) {
    return color.replace(')', ` / ${opacity})`)
  }
  return color
}

// Color scale classes for Tailwind — use semantic theme tokens
export const colorScaleClasses = {
  green: {
    text: 'text-success',
    bg: 'bg-success-light',
    border: 'border-success-light',
  },
  yellow: {
    text: 'text-warning',
    bg: 'bg-warning-light',
    border: 'border-warning-light',
  },
  red: {
    text: 'text-error',
    bg: 'bg-error-light',
    border: 'border-error-light',
  },
  blue: {
    text: 'text-info',
    bg: 'bg-info-light',
    border: 'border-info-light',
  },
  gray: {
    text: 'text-muted-foreground',
    bg: 'bg-muted',
    border: 'border-border',
  },
} as const

export type ColorScaleName = keyof typeof colorScaleClasses
