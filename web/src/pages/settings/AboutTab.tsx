import { useState, useEffect, useCallback, type ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Skeleton } from "@/components/ui/skeleton"
import { BrandName } from "@/components/shared/BrandName"
import type { LucideIcon } from "lucide-react"
import {
  Server,
  Clock,
  Cpu,
  HardDrive,
  Layers,
  Activity,
  Monitor,
  Download,
  Loader2,
  Terminal,
  ExternalLink,
} from "lucide-react"
import { api, isTauriEnv } from "@/lib/api"
import { useErrorHandler } from "@/hooks/useErrorHandler"
import { useUpdateCheck } from "@/hooks/useUpdateCheck"
import { useAppStore, useStore } from "@/store"
import { InstanceManagerDialog } from "@/components/instances/InstanceManagerDialog"

interface GpuInfo {
  name: string
  vendor: string
  total_memory_mb: number | null
  driver_version: string | null
}

interface SystemInfo {
  version: string
  uptime: number
  platform: string
  arch: string
  cpu_count: number
  total_memory: number
  used_memory: number
  free_memory: number
  available_memory: number
  gpus: GpuInfo[]
}

/* ============================================================================
 * Sub-components
 * ========================================================================== */

function MetricTile({
  icon: Icon,
  label,
  value,
  sub,
  mono,
}: {
  icon: LucideIcon
  label: string
  value: string
  sub?: string
  mono?: boolean
}) {
  return (
    <div className="rounded-lg border bg-muted-30 p-4 space-y-2 transition-colors hover:bg-muted-50">
      <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
        <Icon className="h-3.5 w-3.5" />
        <span className="uppercase tracking-wide truncate">{label}</span>
      </div>
      <div
        className={`text-2xl font-semibold leading-none truncate ${mono ? "font-mono" : ""}`}
      >
        {value}
      </div>
      {/* Unified secondary line: always rendered (preserves vertical rhythm across tiles) */}
      <div className="text-xs text-muted-foreground font-mono uppercase tracking-wide truncate min-h-[1rem]">
        {sub ?? "\u00A0"}
      </div>
    </div>
  )
}

function MemoryGauge({
  used,
  total,
  available,
  formatBytes,
  usedLabel,
  availableLabel,
  memoryLabel,
}: {
  used: number
  total: number
  available: number
  formatBytes: (b: number) => string
  usedLabel: string
  availableLabel: string
  memoryLabel: string
}) {
  const pct = Math.round((used / total) * 100)
  const barColor = pct >= 80 ? "bg-error" : pct >= 60 ? "bg-warning" : "bg-success"
  const textColor = pct >= 80 ? "text-error" : pct >= 60 ? "text-warning" : "text-success"

  return (
    <div className="rounded-lg border bg-muted-30 p-4 space-y-3">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div className="space-y-1">
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <HardDrive className="h-3.5 w-3.5" />
            <span className="uppercase tracking-wide">{memoryLabel}</span>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="font-mono text-2xl font-semibold leading-none">
              {pct}
              <span className="text-lg text-muted-foreground">%</span>
            </span>
            <span className={`text-xs font-mono ${textColor}`}>
              {formatBytes(used)} / {formatBytes(total)}
            </span>
          </div>
        </div>
        <div className="text-right">
          <div className="text-xs text-muted-foreground uppercase tracking-wide">
            {availableLabel}
          </div>
          <div className="font-mono text-sm font-medium">{formatBytes(available)}</div>
        </div>
      </div>
      {/* Segmented gauge with tick marks */}
      <div className="relative h-2.5 w-full rounded-full bg-muted overflow-hidden">
        <div
          className={`h-full ${barColor} rounded-full transition-all duration-700 ease-out`}
          style={{ width: `${pct}%` }}
        />
        {[25, 50, 75].map((p) => (
          <div
            key={p}
            className="absolute top-0 bottom-0 w-px bg-glass-border"
            style={{ left: `${p}%` }}
          />
        ))}
      </div>
      <div className="flex justify-between text-[10px] font-mono uppercase tracking-wide text-muted-foreground">
        <span>{usedLabel}: {formatBytes(used)}</span>
      </div>
    </div>
  )
}

function InfoRow({
  label,
  children,
  last,
}: {
  label: string
  children: ReactNode
  last?: boolean
}) {
  return (
    <div
      className={`flex items-center justify-between gap-4 py-3 ${
        !last ? "border-b border-border" : ""
      }`}
    >
      <span className="text-sm text-muted-foreground">{label}</span>
      <div className="text-sm text-right">{children}</div>
    </div>
  )
}

function ExternalLinkValue({ href, text }: { href: string; text: string }) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className="inline-flex items-center gap-1 font-mono text-info hover:underline"
    >
      <span>{text}</span>
      <ExternalLink className="h-3 w-3 text-muted-foreground/70" />
    </a>
  )
}

function TelemetrySkeleton() {
  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
      {Array.from({ length: 4 }).map((_, i) => (
        <div key={i} className="rounded-lg border bg-muted-30 p-4 space-y-2.5">
          <Skeleton className="h-3 w-16" />
          <Skeleton className="h-7 w-20" />
          <Skeleton className="h-3 w-12" />
        </div>
      ))}
    </div>
  )
}

/* ============================================================================
 * Main component
 * ========================================================================== */

export function AboutTab() {
  const { t } = useTranslation(["common", "settings"])
  const { handleError, showSuccess } = useErrorHandler()
  const { updateInfo, setUpdateDialogOpen } = useAppStore()
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null)
  const [appVersion, setAppVersion] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [checkingUpdate, setCheckingUpdate] = useState(false)
  const [instanceManagerOpen, setInstanceManagerOpen] = useState(false)
  const instances = useStore((s) => s.instances)
  const currentInstanceId = useStore((s) => s.currentInstanceId)
  const isConnected = useStore((s) => s.wsConnected)
  const currentInstance = instances.find((i) => i.id === currentInstanceId)
  const instanceOnline =
    isConnected &&
    (!currentInstance || currentInstance.is_local
      ? currentInstance?.last_status === "online" || !currentInstance
      : true)

  const handleUpToDate = useCallback(() => {
    showSuccess(t("settings:alreadyUpToDate"))
  }, [showSuccess, t])

  const { checkUpdate, getAppVersion } = useUpdateCheck({
    autoCheck: false,
    onUpToDate: handleUpToDate,
  })

  const loadSystemInfo = async () => {
    try {
      const response = await api.getSystemStats()
      setSystemInfo(response)
    } catch (e) {
      handleError(e, { operation: "Load system info", showToast: false })
      if (isTauriEnv() && !appVersion) {
        try {
          const v = await getAppVersion()
          setAppVersion(v)
        } catch {
          /* ignore */
        }
      }
    } finally {
      setLoading(false)
    }
  }

  const handleCheckForUpdates = async () => {
    setCheckingUpdate(true)
    try {
      await checkUpdate()
      const latestInfo = useAppStore.getState().updateInfo
      if (!latestInfo?.available) {
        showSuccess(t("settings:alreadyUpToDate"))
      } else {
        setUpdateDialogOpen(true)
      }
    } catch (error) {
      console.error("[AboutTab] checkUpdate error:", error)
      handleError(error, { operation: "Check for updates" })
    } finally {
      setCheckingUpdate(false)
    }
  }

  useEffect(() => {
    loadSystemInfo()
  }, [])

  const formatBytes = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024)
    return gb.toFixed(2) + " GB"
  }

  const formatUptimeParts = (
    seconds: number
  ): { primary: string; secondary: string } => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    if (days > 0) return { primary: `${days}d`, secondary: `${hours}h ${minutes}m` }
    if (hours > 0) return { primary: `${hours}h`, secondary: `${minutes}m` }
    return { primary: `${minutes}m`, secondary: t("common:runStatus.running") }
  }

  const versionTag = systemInfo?.version || (appVersion ? `v${appVersion}` : "")

  const heroVersion = versionTag || "---"

  return (
    <div className="space-y-6">
      {/* Hero — Brand wordmark + build tag */}
      <div className="relative overflow-hidden rounded-xl border bg-card shadow-sm p-6 md:p-10">
        {/* Grid background */}
        <div
          className="absolute inset-0 opacity-[0.04] pointer-events-none"
          style={{
            backgroundImage: `linear-gradient(var(--foreground) 1px, transparent 1px), linear-gradient(90deg, var(--foreground) 1px, transparent 1px)`,
            backgroundSize: "32px 32px",
          }}
        />
        {/* Brand glow */}
        <div
          className="absolute -top-24 -right-24 w-80 h-80 rounded-full pointer-events-none"
          style={{ background: "var(--brand-bg)", filter: "blur(60px)" }}
        />

        <div className="relative flex flex-col md:flex-row md:items-end md:justify-between gap-6">
          <div className="space-y-3 min-w-0">
            <div className="flex items-center gap-2 text-xs uppercase tracking-wide text-muted-foreground font-mono">
              <span className="relative flex h-2 w-2">
                <span className="absolute inline-flex h-full w-full rounded-full bg-success opacity-75 animate-ping" />
                <span className="relative inline-flex rounded-full h-2 w-2 bg-success" />
              </span>
              <span>{t("settings:aboutDesc")}</span>
            </div>
            <h1 className="text-4xl md:text-5xl font-bold tracking-tight leading-none">
              <BrandName />
            </h1>
            <p className="text-sm text-muted-foreground max-w-md">
              {t("settings:aboutDesc1")}
            </p>
          </div>

          <div className="flex flex-col items-start md:items-end gap-2 shrink-0">
            <div className="font-mono text-[10px] text-muted-foreground uppercase tracking-wide">
              build
            </div>
            <div className="font-mono text-2xl md:text-3xl font-semibold text-brand leading-none">
              [{heroVersion}]
            </div>
          </div>
        </div>
      </div>

      {/* Instance manager entry — moved here from the mobile drawer so the
          drawer stays focused on navigation. Desktop also exposes this via
          TopNav's InstanceSelector, but having it in Settings gives a single
          canonical home on mobile. */}
      <Card>
        <CardContent className="flex items-center justify-between gap-3 py-4">
          <div className="flex min-w-0 items-center gap-3">
            <div
              className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-md ${
                instanceOnline
                  ? "bg-success-light text-success"
                  : "bg-error-light text-error"
              }`}
            >
              <Server className="h-4 w-4" />
            </div>
            <div className="min-w-0">
              <div className="truncate text-sm font-medium">
                {currentInstance?.name || t("instances:local", "Local")}
              </div>
              <div
                className={`text-xs ${
                  instanceOnline ? "text-success" : "text-error"
                }`}
              >
                {instanceOnline
                  ? t("instances:status.online", "Online")
                  : t("instances:status.offline", "Offline")}
              </div>
            </div>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setInstanceManagerOpen(true)}
          >
            {t("instances:manage", "Manage")}
          </Button>
        </CardContent>
      </Card>

      {/* System Information Card */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5 text-info" />
            {t("settings:systemInfo")}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {loading ? (
            <TelemetrySkeleton />
          ) : systemInfo ? (
            (() => {
              const uptime = formatUptimeParts(systemInfo.uptime)
              return (
            <>
              {/* Telemetry tiles */}
              <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
                <MetricTile
                  icon={Server}
                  label={t("settings:platform")}
                  value={systemInfo.platform}
                  sub={systemInfo.arch}
                />
                <MetricTile
                  icon={Clock}
                  label={t("settings:uptime")}
                  value={uptime.primary}
                  sub={uptime.secondary}
                  mono
                />
                <MetricTile
                  icon={Cpu}
                  label={t("settings:cpuCores")}
                  value={String(systemInfo.cpu_count)}
                  sub={t("settings:cores")}
                  mono
                />
                {systemInfo.gpus.length > 0 ? (
                  <MetricTile
                    icon={Monitor}
                    label={t("settings:gpu")}
                    value={String(systemInfo.gpus.length)}
                    sub={systemInfo.gpus[0]?.vendor ?? "GPU"}
                    mono
                  />
                ) : (
                  <MetricTile
                    icon={Layers}
                    label={t("settings:memory")}
                    value={formatBytes(systemInfo.total_memory)}
                    sub="total"
                    mono
                  />
                )}
              </div>

              {/* Memory gauge */}
              <MemoryGauge
                used={systemInfo.used_memory}
                total={systemInfo.total_memory}
                available={systemInfo.available_memory}
                formatBytes={formatBytes}
                usedLabel={t("settings:usedMemory")}
                availableLabel={t("settings:availableMemory")}
                memoryLabel={t("settings:memory")}
              />

              {/* GPU detail rows */}
              {systemInfo.gpus.length > 0 && (
                <div className="space-y-2">
                  {systemInfo.gpus.map((gpu, idx) => (
                    <div
                      key={idx}
                      className="rounded-lg border bg-muted-30 p-3 flex items-center justify-between gap-3"
                    >
                      <div className="flex items-center gap-3 min-w-0">
                        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-brand-bg text-brand">
                          <Monitor className="h-4.5 w-4.5" />
                        </div>
                        <div className="min-w-0">
                          <div className="text-sm font-medium truncate">{gpu.name}</div>
                          <div className="text-xs text-muted-foreground font-mono uppercase tracking-wide">
                            {gpu.vendor}
                          </div>
                        </div>
                      </div>
                      {gpu.total_memory_mb && (
                        <div className="text-right shrink-0">
                          <div className="font-mono text-base font-bold leading-none">
                            {(gpu.total_memory_mb / 1024).toFixed(1)}
                          </div>
                          <div className="text-[10px] text-muted-foreground uppercase tracking-wide mt-1">
                            GB VRAM
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </>
              )
            })()
          ) : (
            <div className="text-center py-8 text-muted-foreground text-sm">
              {t("settings:systemInfoUnavailable")}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Project Information Card */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Terminal className="h-5 w-5 text-info" />
            {t("settings:projectInfo")}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <InfoRow label={t("settings:version")}>
              <div className="flex items-center gap-2">
                <Badge variant="secondary" className="font-mono">
                  {versionTag || "---"}
                </Badge>
                {isTauriEnv() &&
                  updateInfo?.available &&
                  updateInfo.version !== systemInfo?.version && (
                    <Badge variant="default" className="text-xs gap-1">
                      <Download className="h-3 w-3" />
                      v{updateInfo.version} {t("settings:update")}
                    </Badge>
                  )}
              </div>
            </InfoRow>
            <InfoRow label={t("settings:license")}>
              <span className="font-mono">Apache-2.0</span>
            </InfoRow>
            <InfoRow label={t("settings:repository")}>
              <ExternalLinkValue
                href="https://github.com/CVEDIX/HeraMind"
                text="github.com/CVEDIX/HeraMind"
              />
            </InfoRow>
            <InfoRow label={t("settings:website")}>
              <ExternalLinkValue href="https://cvedix.com" text="cvedix.com" />
            </InfoRow>
            <InfoRow label={t("settings:documentation")} last>
              <ExternalLinkValue
                href="https://docs.cvedix.com/heramind"
                text="docs.cvedix.com/heramind"
              />
            </InfoRow>
          </div>

          {isTauriEnv() && (
            <Button
              variant={updateInfo?.available ? "default" : "outline"}
              className="w-full"
              onClick={() =>
                updateInfo?.available
                  ? setUpdateDialogOpen(true)
                  : handleCheckForUpdates()
              }
              disabled={checkingUpdate}
            >
              {checkingUpdate ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  {t("settings:checkingForUpdates")}
                </>
              ) : updateInfo?.available ? (
                <>
                  <Download className="w-4 h-4 mr-2" />
                  {t("settings:updateNow")} · v{updateInfo.version}
                </>
              ) : (
                <>
                  <Download className="w-4 h-4 mr-2" />
                  {t("settings:checkForUpdates")}
                </>
              )}
            </Button>
          )}
        </CardContent>
      </Card>

      {/* Footer */}
      <div className="text-center text-xs text-muted-foreground">
        © CVEDIX AI 2026 · HeraMind
      </div>

      <InstanceManagerDialog
        open={instanceManagerOpen}
        onOpenChange={setInstanceManagerOpen}
      />
    </div>
  )
}
