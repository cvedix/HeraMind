import { memo } from "react"

// Honeycomb mesh — ported from HeraMind-Landing Capabilities.astro.
// Pointy-top hex tight tiling (odd rows offset = true 6-neighbor honeycomb),
// HeraMind blue, ripple-from-center breathe (delay based on distance to center).
// Shared by the login + setup pages so the whole auth flow reads as one surface.
const HEX_S = 48
const HEX_W = HEX_S * Math.sqrt(3)
const HALF_W = HEX_W / 2
const ROW_STEP = HEX_S * 1.5
const HEX_COLS = 24
const HEX_ROWS = 17
const HEXES: { pts: string; delay: string }[] = (() => {
  const COLS = HEX_COLS, ROWS = HEX_ROWS
  const cCenter = COLS / 2, rCenter = ROWS / 2
  const out: { pts: string; delay: string }[] = []
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const cx = c * HEX_W + (r % 2) * HALF_W
      const cy = r * ROW_STEP
      const pts = [
        [cx, cy - HEX_S], [cx + HALF_W, cy - HEX_S / 2],
        [cx + HALF_W, cy + HEX_S / 2], [cx, cy + HEX_S],
        [cx - HALF_W, cy + HEX_S / 2], [cx - HALF_W, cy - HEX_S / 2],
      ].map(p => p.map(v => v.toFixed(1)).join(',')).join(' ')
      const dist = Math.sqrt((c - cCenter) ** 2 + (r - rCenter) ** 2)
      const jitter = ((r * 31 + c * 17) % 13) * 0.08
      const delay = (((dist * 0.15) + jitter) % 7).toFixed(2)
      out.push({ pts, delay })
    }
  }
  return out
})()

/**
 * Animated HeraMind-blue honeycomb mesh, masked to fade at the edges.
 * Drop inside a `fixed inset-0` / `absolute inset-0` container.
 *
 * The base opacity lives on the `.honeycomb-cell` class (NOT the keyframes)
 * so cells render invisible on first paint — `animation-fill-mode` defaults to
 * `none`, meaning the 0% keyframe does NOT apply during `animation-delay`,
 * which would otherwise flash all cells solid blue before the breathe starts.
 */
export const HoneycombBackground = memo(function HoneycombBackground() {
  return (
    <svg
      className="absolute inset-0 h-full w-full"
      viewBox={`0 0 ${HEX_COLS * HEX_W} ${HEX_ROWS * ROW_STEP}`}
      preserveAspectRatio="xMidYMid slice"
      aria-hidden
    >
      <style>{`
        .honeycomb-cell {
          fill-opacity: 0;
          stroke-opacity: 0.015;
        }
        @keyframes honeycomb-breathe {
          0%, 100% { fill-opacity: 0; stroke-opacity: 0.015; }
          50% { fill-opacity: 0.04; stroke-opacity: 0.025; }
        }
        @media (prefers-reduced-motion: reduce) {
          .honeycomb-cell {
            animation: none !important;
            fill-opacity: 0.02;
            stroke-opacity: 0.02;
          }
        }
      `}</style>
      <g
        style={{
          maskImage: 'radial-gradient(ellipse 90% 80% at 50% 50%, black 30%, transparent 85%)',
          WebkitMaskImage: 'radial-gradient(ellipse 90% 80% at 50% 50%, black 30%, transparent 85%)',
        }}
      >
        {HEXES.map((h, i) => (
          <polygon
            key={i}
            points={h.pts}
            className="honeycomb-cell"
            style={{
              fill: 'var(--accent-blue)',
              stroke: 'var(--accent-blue)',
              strokeWidth: 1,
              vectorEffect: 'non-scaling-stroke',
              animation: 'honeycomb-breathe 7s ease-in-out infinite',
              animationDelay: `${h.delay}s`,
            }}
          />
        ))}
      </g>
    </svg>
  )
})
