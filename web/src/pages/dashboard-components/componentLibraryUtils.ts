/**
 * Component Library Utilities
 *
 * Extracted from VisualDashboard.tsx — provides types and the factory function
 * used to build the component library sidebar with i18n support.
 */

import type React from 'react'
import { groupComponentsByCategory, getCategoryInfo } from '@/components/dashboard/registry/registry'
import { dynamicIconMap } from '@/lib/dynamicIcons'
import { Box } from 'lucide-react'

// ============================================================================
// Types
// ============================================================================

export type ComponentIconType = React.ComponentType<{ className?: string }>

export interface ComponentItem {
  id: string
  name: string
  description: string
  icon: ComponentIconType
}

export interface ComponentCategory {
  category: string
  categoryLabel: string
  categoryIcon: ComponentIconType
  categoryColor: string
  items: ComponentItem[]
}

// ============================================================================
// i18n key mappings
// ============================================================================

/** Component type → translation key for the component name */
const nameKeys: Record<string, string> = {
  'value-card': 'valueCard',
  'led-indicator': 'ledIndicator',
  'sparkline': 'sparkline',
  'progress-bar': 'progressBar',
  'line-chart': 'lineChart',
  'area-chart': 'areaChart',
  'bar-chart': 'barChart',
  'pie-chart': 'pieChart',
  'image-display': 'imageDisplay',
  'image-history': 'imageHistory',
  'web-display': 'webDisplay',
  'markdown-display': 'markdownDisplay',
  'map-display': 'mapDisplay',
  'video-display': 'videoDisplay',
  'custom-layer': 'customLayer',
  'toggle-switch': 'toggleSwitch',
  'agent-monitor-widget': 'agentMonitor',
  'vlm-vision': 'aiAnalyst',
  'ai-analyst': 'aiAnalyst',
}

/** Component type → translation key for the component description */
const descKeys: Record<string, string> = {
  'value-card': 'valueCardDesc',
  'led-indicator': 'ledIndicatorDesc',
  'sparkline': 'sparklineDesc',
  'progress-bar': 'progressBarDesc',
  'line-chart': 'lineChartDesc',
  'area-chart': 'areaChartDesc',
  'bar-chart': 'barChartDesc',
  'pie-chart': 'pieChartDesc',
  'image-display': 'imageDisplayDesc',
  'image-history': 'imageHistoryDesc',
  'web-display': 'webDisplayDesc',
  'markdown-display': 'markdownDisplayDesc',
  'map-display': 'mapDisplayDesc',
  'video-display': 'videoDisplayDesc',
  'custom-layer': 'customLayerDesc',
  'toggle-switch': 'toggleSwitchDesc',
  'agent-monitor-widget': 'agentMonitorDesc',
  'vlm-vision': 'aiAnalystDesc',
  'ai-analyst': 'aiAnalystDesc',
}

/** Category → translation key for the category label */
const categoryLabelKeys: Record<string, string> = {
  indicators: 'indicators',
  charts: 'charts',
  display: 'display',
  spatial: 'spatial',
  controls: 'controls',
  business: 'business',
  custom: 'custom',
  local: 'localComponents',
  marketplace: 'marketplace',
}

/** Category → accent color (background for icon badge, top strip) */
const categoryColors: Record<string, string> = {
  indicators: 'bg-info-light text-info',
  charts: 'bg-success-light text-success',
  controls: 'bg-warning-light text-warning',
  display: 'bg-info-light text-info',
  spatial: 'bg-warning-light text-warning',
  business: 'bg-success-light text-success',
  custom: 'bg-warning-light text-warning',
  local: 'bg-info-light text-info',
  marketplace: 'bg-success-light text-success',
}

// ============================================================================
// Factory
// ============================================================================

/** Build the full component library structure with translated labels. */
export function getComponentLibrary(t: (key: string) => string): ComponentCategory[] {
  // Get all components grouped by category from the registry
  const grouped = groupComponentsByCategory()

  return grouped.map((group) => {
    const catInfo = getCategoryInfo(group.category as any)
    const labelKey = categoryLabelKeys[group.category]

    return {
      category: group.category,
      categoryLabel: labelKey ? t(`componentLibrary.${labelKey}`) : catInfo.name,
      categoryIcon: catInfo.icon,
      categoryColor: categoryColors[group.category] || 'bg-muted text-muted-foreground',
      items: group.components.map((comp) => {
        const iconName = (comp.icon as any)?.displayName || 'Box'
        const IconComponent = typeof comp.icon === 'function' ? comp.icon : (dynamicIconMap[iconName] || Box)
        const nKey = nameKeys[comp.type]
        const dKey = descKeys[comp.type]

        return {
          id: comp.type,
          name: nKey ? t(`componentLibrary.${nKey}`) : comp.name,
          description: dKey ? t(`componentLibrary.${dKey}`) : comp.description,
          icon: IconComponent,
        }
      }),
    }
  })
}
