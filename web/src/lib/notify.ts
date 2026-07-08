/**
 * Global notification system for showing toast messages outside React components.
 * This allows API calls to automatically display success/error messages.
 */

import { toast as toastFn } from "@/hooks/use-toast"
import i18n from "@/i18n/config"

export type NotifyType = 'success' | 'error' | 'warning' | 'info'

export interface NotifyOptions {
  title?: string
  message: string
  type?: NotifyType
  duration?: number
}

/**
 * Show a notification toast
 */
export function notify(options: NotifyOptions) {
  const { title, message, type = 'info', duration } = options

  const variant = type === 'error' ? 'destructive' : 'default'

  // Get translated title based on type
  const defaultTitle = {
    success: i18n.t('common:success'),
    error: i18n.t('common:failed'),
    warning: i18n.t('common:warning'),
    info: ''
  }[type] || ''

  toastFn({
    title: title || defaultTitle || undefined,
    description: message,
    variant,
    ...(duration !== undefined && { duration: duration > 0 ? duration : undefined })
  })
}

/**
 * Shorthand methods
 */
export const notifySuccess = (message: string, title?: string) =>
  notify({ type: 'success', message, title })

export const notifyError = (message: string, title?: string) =>
  notify({ type: 'error', message, title })

export const notifyWarning = (message: string, title?: string) =>
  notify({ type: 'warning', message, title })

export const notifyInfo = (message: string, title?: string) =>
  notify({ type: 'info', message, title })

/**
 * Extract user-friendly error message from various error formats
 */
export function extractErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error
  }

  if (error instanceof Error) {
    return error.message
  }

  if (error && typeof error === 'object') {
    // Format: { response: { data: { error: { message: "..." } } } }
    const axiosError = error as any
    if (axiosError?.response?.data?.error?.message) {
      return axiosError.response.data.error.message
    }
    // Format: { error: { message: "..." } }
    if (axiosError?.error?.message) {
      return axiosError.error.message
    }
    // Format: { message: "..." }
    if (axiosError?.message) {
      return axiosError.message
    }
    // Format: { error: "Error message" }
    if (axiosError?.error && typeof axiosError.error === 'string') {
      return axiosError.error
    }
  }

  return String(error)
}

/**
 * Show error notification from any error type
 */
export function notifyFromError(error: unknown, fallbackMessage?: string) {
  const message = extractErrorMessage(error)
  notifyError(message || fallbackMessage || 'An error occurred')
}
