/**
 * Brand configuration hook
 *
 * Provides centralized access to brand information (name, short name, etc.)
 * Makes it easy to rebrand the application by modifying i18n files.
 */

import { useTranslation } from 'react-i18next'

export interface BrandInfo {
  /** Full application name */
  name: string
  /** Short name/acronym for compact display */
  shortName: string
}

/**
 * Hook to access brand information
 *
 * @example
 * ```tsx
 * const { name, shortName } = useBrand()
 * <h1>{name}</h1>
 * ```
 */
export function useBrand(): BrandInfo {
  const { t } = useTranslation('common')
  return {
    name: t('app.name'),
    shortName: t('app.shortName')
  }
}

/**
 * Hook to get formatted brand name with context
 *
 * @example
 * ```tsx
 * const { getWelcomeMessage } = useBrand()
 * <p>{getWelcomeMessage('tagline')}</p>
 * ```
 */
export function useBrandMessages() {
  const { t } = useTranslation('common')
  const appName = t('app.name')

  return {
    appName,
    /** Get a message with appName interpolated */
    getWelcomeMessage: (key: string) => {
      return t(`welcome.${key}`, { appName })
    }
  }
}
