/**
 * usePageContext - Generates short, neutral page context for AI chat.
 *
 * Produces a factual description of the current page and its data,
 * without any suggestions or bias. Only used by the global FAB chat
 * to give the model awareness of what page the user is on.
 *
 * Updated reactively on route changes, but only read when sending
 * the first message of a new conversation.
 */
import { useMemo } from 'react'
import { useLocation, useParams } from 'react-router-dom'
import { useStore } from '@/store'

export function usePageContext(): string {
  const location = useLocation()
  const params = useParams()

  // Read relevant store slices (lightweight selectors)
  const devices = useStore((s) => s.devices)
  const currentDashboard = useStore((s) => s.currentDashboard)

  return useMemo(() => {
    const path = location.pathname

    // Don't inject on chat pages
    if (path === '/' || path === '/chat' || path.startsWith('/chat/')) {
      return ''
    }

    // Dashboard page
    if (path.startsWith('/visual-dashboard')) {
      const name = currentDashboard?.name
      const count = currentDashboard?.components?.length
      if (name) {
        return `[context] page:dashboard "${name}"${count != null ? `, ${count} components` : ''}`
      }
      return '[context] page:dashboard'
    }

    // Devices page
    if (path.startsWith('/devices')) {
      const total = devices.length
      const online = devices.filter((d) => d.status === 'online').length
      return `[context] page:devices, ${total} devices (${online} online)`
    }

    // Automation page
    if (path.startsWith('/automation')) {
      if (path.includes('transforms')) {
        return '[context] page:data transforms'
      }
      return '[context] page:automation'
    }

    // Agents page
    if (path.startsWith('/agents')) {
      return '[context] page:agents'
    }

    // Extensions page
    if (path.startsWith('/extensions')) {
      return '[context] page:extensions'
    }

    // Messages page
    if (path.startsWith('/messages')) {
      return '[context] page:messages'
    }

    // Data explorer
    if (path.startsWith('/data')) {
      return '[context] page:data explorer'
    }

    return ''
  }, [location.pathname, currentDashboard?.name, currentDashboard?.components?.length, devices])
}
