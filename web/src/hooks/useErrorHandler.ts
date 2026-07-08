/**
 * useErrorHandler Hook
 *
 * Provides consistent error handling across components.
 * Replaces scattered console.error and toast notifications.
 */

import { useCallback } from 'react'
import { useToast } from './use-toast'
import { useTranslation } from 'react-i18next'
import { logError, formatErrorForUser, StandardError } from '@/lib/errors'

export interface ErrorHandlerOptions {
  /** Show user-facing toast notification */
  showToast?: boolean
  /** Custom user-facing error message */
  userMessage?: string
  /** Override default error log context */
  operation?: string
}

export interface UseErrorHandlerReturn {
  /** Handle an error with logging and optional toast notification */
  handleError: (error: StandardError, options?: ErrorHandlerOptions) => void
  /** Handle an error and return a formatted message */
  getErrorMessage: (error: StandardError) => string
  /** Execute an async function with automatic error handling */
  withErrorHandling: <T>(fn: () => Promise<T>, options?: ErrorHandlerOptions) => Promise<T | null>
  /** Show success toast notification */
  showSuccess: (message: string) => void
}

/**
 * Hook for consistent error handling across components
 *
 * @example
 * ```tsx
 * const { handleError, showSuccess, withErrorHandling } = useErrorHandler()
 *
 * // Manual error handling
 * try {
 *   await api.deleteDevice(id)
 *   showSuccess('Device deleted')
 * } catch (error) {
 *   handleError(error, { operation: 'Delete device' })
 * }
 *
 * // Automatic error handling
 * const result = await withErrorHandling(
 *   () => api.createDevice(data),
 *   { operation: 'Create device' }
 * )
 * ```
 */
export function useErrorHandler(): UseErrorHandlerReturn {
  const { toast } = useToast()
  const { t } = useTranslation('common')

  const handleError = useCallback((error: StandardError, options: ErrorHandlerOptions = {}) => {
    const { showToast = true, userMessage, operation = 'Operation' } = options

    // Log error with context
    logError(error, { operation, userMessage })

    // Show toast notification
    if (showToast) {
      const message = userMessage || formatErrorForUser(error, { userMessage })
      toast({
        title: t('error'),
        description: message,
        variant: 'destructive',
      })
    }
  }, [toast, t])

  const getErrorMessage = useCallback((error: StandardError): string => {
    return formatErrorForUser(error)
  }, [])

  const withErrorHandling = useCallback(async <T>(
    fn: () => Promise<T>,
    options: ErrorHandlerOptions = {}
  ): Promise<T | null> => {
    try {
      return await fn()
    } catch (error) {
      handleError(error, options)
      return null
    }
  }, [handleError])

  const showSuccess = useCallback((message: string) => {
    toast({
      title: t('success'),
      description: message,
    })
  }, [toast, t])

  return {
    handleError,
    getErrorMessage,
    withErrorHandling,
    showSuccess,
  }
}

/**
 * Simple hook for form submission with loading and error handling
 */
export interface UseFormSubmitOptions {
  onSuccess?: () => void
  onError?: (error: Error) => void
  successMessage?: string
  errorOperation?: string
}

export interface UseFormSubmitReturn {
  /** Whether the form is currently submitting */
  isSubmitting: boolean
  /** Submit handler function */
  handleSubmit: (fn: () => Promise<void>) => () => Promise<void>
  /** Reset submitting state */
  reset: () => void
}

/**
 * Hook for form submissions with built-in loading and error handling
 *
 * @example
 * ```tsx
 * const { isSubmitting, handleSubmit } = useFormSubmit({
 *   successMessage: 'Saved successfully',
 *   errorOperation: 'Save settings',
 * })
 *
 * <form onSubmit={handleSubmit(async () => await api.save(data))}>
 * ```
 */
export function useFormSubmit(options: UseFormSubmitOptions = {}): UseFormSubmitReturn {
  const { onSuccess, onError, successMessage, errorOperation } = options
  const { handleError, showSuccess } = useErrorHandler()
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmit = useCallback((fn: () => Promise<void>) => {
    return async () => {
      setIsSubmitting(true)
      try {
        await fn()
        onSuccess?.()
        if (successMessage) showSuccess(successMessage)
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error))
        handleError(err, { operation: errorOperation })
        onError?.(err)
      } finally {
        setIsSubmitting(false)
      }
    }
  }, [errorOperation, handleError, onSuccess, onError, showSuccess, successMessage])

  const reset = useCallback(() => {
    setIsSubmitting(false)
  }, [])

  return { isSubmitting, handleSubmit, reset }
}

import { useState } from 'react'
