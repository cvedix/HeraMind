/**
 * BuildCard - Rich card for AI Build Mode CLI results
 *
 * When a tool call result contains build_meta, this card renders
 * an interactive summary instead of raw JSON.
 *
 * Structure:
 *   ┌─ Type icon + Entity name ──────────────┐
 *   │ Action label · Status badge             │
 *   │ Key-value details from data             │
 *   │ [View] [Undo]                           │
 *   └─────────────────────────────────────────┘
 */

import { useState, useCallback } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import {
  CheckCircle2,
  XCircle,
  Cpu,
  LayoutDashboard,
  GitBranch,
  Puzzle,
  LayoutGrid,
  ArrowLeftRight,
  Bot,
  MessageSquare,
  ExternalLink,
  Undo2,
  ChevronDown,
} from "lucide-react"
import { cn } from "@/lib/utils"
import { textBody, textMini, textNano } from "@/design-system/tokens/typography"
import { getIconForEntity } from "@/design-system/icons"
import type { BuildMeta, BuildEntityType, CliBuildResponse } from "@/types"

// ─── Entity type to icon mapping ───

function getBuildTypeIcon(type: BuildEntityType) {
  const iconMap: Record<BuildEntityType, typeof Cpu> = {
    device: Cpu,
    dashboard: LayoutDashboard,
    rule: GitBranch,
    extension: Puzzle,
    widget: LayoutGrid,
    transform: ArrowLeftRight,
    agent: Bot,
    message: MessageSquare,
  }
  return iconMap[type] || Cpu
}

// ─── Entity type to route path mapping ───

function getEntityRoute(meta: BuildMeta): string | null {
  const routeMap: Record<BuildEntityType, (id: string) => string> = {
    device: (id) => `/devices/${id}`,
    dashboard: (id) => `/visual-dashboard/${id}`,
    rule: (id) => `/automation`,
    extension: (id) => `/extensions`,
    widget: (id) => `/visual-dashboard`,
    transform: () => `/automation/transforms`,
    agent: (id) => `/agents`,
    message: () => `/messages`,
  }
  const builder = routeMap[meta.type]
  return builder ? builder(meta.entity_id) : null
}

// ─── Parse tool call result to extract build_meta ───

export function parseBuildResponse(result: unknown): CliBuildResponse | null {
  if (!result || typeof result !== "object") return null
  // result could be the CliBuildResponse directly, or a string that needs parsing
  const obj = result as Record<string, unknown>
  if (!obj.build_meta) return null
  try {
    // Validate build_meta structure
    const meta = obj.build_meta as Record<string, unknown>
    if (typeof meta.type === "string" && typeof meta.action === "string" && typeof meta.entity_id === "string") {
      return obj as unknown as CliBuildResponse
    }
  } catch {
    return null
  }
  return null
}

// ─── BuildActionButtons ───

interface BuildActionButtonsProps {
  meta: BuildMeta
  success: boolean
}

function BuildActionButtons({ meta, success }: BuildActionButtonsProps) {
  const { t } = useTranslation("chat")
  const navigate = useNavigate()

  const route = success ? getEntityRoute(meta) : null

  return (
    <div className="flex items-center gap-2 mt-2">
      {route && (
        <button
          onClick={(e) => {
            e.stopPropagation()
            navigate(route)
          }}
          className={cn(
            "inline-flex items-center gap-1 px-2 py-1 rounded-md",
            "bg-primary-light text-primary hover:bg-primary-lightHover transition-colors",
            textMini
          )}
        >
          <ExternalLink className="h-3 w-3" />
          {t("build.view")}
        </button>
      )}
      {success && meta.undo_command && (
        <button
          onClick={(e) => {
            e.stopPropagation()
            // TODO: trigger undo via chat send
          }}
          className={cn(
            "inline-flex items-center gap-1 px-2 py-1 rounded-md",
            "bg-muted-50 text-muted-foreground hover:bg-muted-30 transition-colors",
            textMini
          )}
        >
          <Undo2 className="h-3 w-3" />
          {t("build.undo")}
        </button>
      )}
    </div>
  )
}

// ─── BuildCard main component ───

interface BuildCardProps {
  response: CliBuildResponse
}

export function BuildCard({ response }: BuildCardProps) {
  const { t } = useTranslation("chat")
  const meta = response.build_meta!
  const success = response.success
  const IconComponent = getBuildTypeIcon(meta.type)
  const [expanded, setExpanded] = useState(false)

  const actionLabel = t(`build.actions.${meta.action}`, meta.action)
  const typeLabel = t(`build.types.${meta.type}`, meta.type)

  // Extract key details from data for summary display
  const dataEntries: Array<[string, string | number | boolean]> = []
  if (response.data && typeof response.data === "object") {
    for (const [key, value] of Object.entries(response.data)) {
      if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
        // Skip internal-looking fields
        if (["id", "created_at", "updated_at"].includes(key)) continue
        dataEntries.push([key, value])
      }
    }
  }

  return (
    <div className={cn(
      "rounded-lg border overflow-hidden",
      success ? "border-success-light bg-success-light" : "border-error-light bg-error-light"
    )}>
      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-2">
        <div className={cn(
          "h-5 w-5 rounded flex items-center justify-center shrink-0",
          success ? "bg-success-light text-success" : "bg-error-light text-error"
        )}>
          {success ? (
            <CheckCircle2 className="h-3.5 w-3.5" />
          ) : (
            <XCircle className="h-3.5 w-3.5" />
          )}
        </div>
        <IconComponent className={cn("h-4 w-4 shrink-0", success ? "text-success" : "text-error")} />
        <span className={cn(textBody, "font-medium text-foreground truncate")}>
          {meta.entity_name || meta.entity_id}
        </span>
        <span className={cn(
          textNano, "px-1.5 py-0.5 rounded shrink-0",
          success ? "bg-success-light text-success" : "bg-error-light text-error"
        )}>
          {actionLabel}
        </span>
        <div className="flex-1" />
        <span className={cn(textNano, "text-muted-foreground")}>
          {typeLabel}
        </span>
      </div>

      {/* Details */}
      {success && response.message && (
        <div className={cn(textBody, "px-3 pb-1 text-muted-foreground")}>
          {response.message}
        </div>
      )}
      {!success && response.error && (
        <div className={cn(textBody, "px-3 pb-1 text-error")}>
          {response.error}
        </div>
      )}

      {/* Key-value data summary (collapsible) */}
      {dataEntries.length > 0 && (
        <div className="border-t border-glass-border">
          <button
            onClick={() => setExpanded(!expanded)}
            className="w-full flex items-center gap-1.5 px-3 py-1.5 text-left hover:bg-muted-30 transition-colors"
          >
            <ChevronDown className={cn(
              "h-3 w-3 text-muted-foreground transition-transform",
              expanded && "rotate-180"
            )} />
            <span className={cn(textMini, "text-muted-foreground")}>
              {expanded ? t("build.hideDetails") : t("build.showDetails")}
            </span>
          </button>
          {expanded && (
            <div className="px-3 pb-2 space-y-0.5">
              {dataEntries.slice(0, 8).map(([key, value]) => (
                <div key={key} className={cn(textMini, "flex gap-2")}>
                  <span className="text-muted-foreground min-w-[80px]">{key}:</span>
                  <span className="text-foreground font-mono truncate">{String(value)}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Action buttons */}
      <div className="px-3 pb-2">
        <BuildActionButtons meta={meta} success={success} />
      </div>
    </div>
  )
}
