/**
 * QuickActions - AI-generated action suggestions based on conversation context
 * Appears after assistant messages with suggested follow-up actions
 */

import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import {
  Sparkles,
  RefreshCw,
  ChevronRight,
  Settings,
  Cpu,
  AlertTriangle,
  TrendingUp,
  FileText,
  Zap
} from "lucide-react"
import { cn } from "@/lib/utils"
import type { Message } from "@/types"

interface QuickAction {
  id: string
  label: string
  prompt: string
  icon: React.ReactNode
  category: "device" | "automation" | "analytics" | "general"
}

// Analyze message content to generate relevant quick actions
function generateQuickActions(message: Message, t: (key: string) => string): QuickAction[] {
  const content = (message.content || "").toLowerCase()
  const actions: QuickAction[] = []

  // Device-related suggestions
  if (content.includes("设备") || content.includes("sensor") || content.includes("温度") ||
      content.includes("device") || content.includes("temperature")) {
    actions.push({
      id: "refresh-devices",
      label: t("quickActions.refreshDevices"),
      prompt: t("quickActions.refreshPrompt"),
      icon: <RefreshCw className="h-4 w-4" />,
      category: "device"
    })
    actions.push({
      id: "device-details",
      label: t("quickActions.viewDeviceDetails"),
      prompt: t("quickActions.viewDevicePrompt"),
      icon: <Cpu className="h-4 w-4" />,
      category: "device"
    })
  }

  // Automation-related suggestions
  if (content.includes("自动化") || content.includes("规则") || content.includes("工作流") ||
      content.includes("automation") || content.includes("rule") || content.includes("workflow")) {
    actions.push({
      id: "create-automation",
      label: t("quickActions.createAutomation"),
      prompt: t("quickActions.createAutomationPrompt"),
      icon: <Zap className="h-4 w-4" />,
      category: "automation"
    })
    actions.push({
      id: "list-automations",
      label: t("quickActions.viewAllAutomations"),
      prompt: t("quickActions.viewAllAutomationsPrompt"),
      icon: <FileText className="h-4 w-4" />,
      category: "automation"
    })
  }

  // Analytics-related suggestions
  if (content.includes("数据") || content.includes("统计") || content.includes("趋势") ||
      content.includes("data") || content.includes("analytics") || content.includes("trend")) {
    actions.push({
      id: "show-analytics",
      label: t("quickActions.viewAnalytics"),
      prompt: t("quickActions.viewAnalyticsPrompt"),
      icon: <TrendingUp className="h-4 w-4" />,
      category: "analytics"
    })
  }

  // Alert-related suggestions
  if (content.includes("告警") || content.includes("异常") || content.includes("错误") ||
      content.includes("alert") || content.includes("error")) {
    actions.push({
      id: "check-alerts",
      label: t("quickActions.checkAlerts"),
      prompt: t("quickActions.checkAlertsPrompt"),
      icon: <AlertTriangle className="h-4 w-4" />,
      category: "general"
    })
  }

  // Settings-related suggestions
  if (content.includes("配置") || content.includes("设置") || content.includes("config") || content.includes("settings")) {
    actions.push({
      id: "open-settings",
      label: t("quickActions.openSettings"),
      prompt: t("quickActions.openSettingsPrompt"),
      icon: <Settings className="h-4 w-4" />,
      category: "general"
    })
  }

  // Always add some generic actions if none were generated
  if (actions.length === 0) {
    actions.push(
      {
        id: "continue",
        label: t("quickActions.continue"),
        prompt: t("quickActions.continuePrompt"),
        icon: <ChevronRight className="h-4 w-4" />,
        category: "general"
      },
      {
        id: "explain-more",
        label: t("quickActions.elaborate"),
        prompt: t("quickActions.elaboratePrompt"),
        icon: <Sparkles className="h-4 w-4" />,
        category: "general"
      }
    )
  }

  // Limit to 4 actions
  return actions.slice(0, 4)
}

interface QuickActionsProps {
  message: Message
  onActionClick: (prompt: string) => void
}

export function QuickActions({ message, onActionClick }: QuickActionsProps) {
  const { t } = useTranslation("chat")
  const actions = useMemo(() => generateQuickActions(message, t), [message, t])

  if (actions.length === 0) return null

  return (
    <div className="flex flex-wrap gap-2 mt-2">
      {actions.map((action) => (
        <button
          key={action.id}
          onClick={() => onActionClick(action.prompt)}
          onTouchEnd={(e) => {
            e.preventDefault()
            onActionClick(action.prompt)
          }}
          className={cn(
            "flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-medium",
            "transition-all duration-200",
            "bg-[var(--quick-action-bg)]",
            "hover:bg-[var(--quick-action-hover)]",
            "text-[var(--quick-action-text)]",
            "border border-[var(--quick-action-border)]",
            "hover:shadow-sm hover:scale-105"
          )}
          style={{ touchAction: 'manipulation' }}
        >
          {action.icon}
          <span>{action.label}</span>
          <ChevronRight className="h-4 w-4 opacity-50" />
        </button>
      ))}
    </div>
  )
}
