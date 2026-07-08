/**
 * Shared background effect for setup pages
 */
export function SetupBackground() {
  return (
    <div className="fixed inset-0">
      <div className="absolute inset-0 bg-gradient-to-br from-background via-background to-muted" />
      <div className="absolute inset-0" style={{
        backgroundImage: 'radial-gradient(circle, color-mix(in oklch, var(--foreground) 8%, transparent) 1px, transparent 1px)',
        backgroundSize: '32px 32px'
      }} />
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-muted rounded-full blur-3xl animate-pulse" style={{ animationDuration: '8s' }} />
    </div>
  )
}
