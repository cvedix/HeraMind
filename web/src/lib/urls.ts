/**
 * Dynamic URL management for multi-backend instance switching.
 *
 * Provides API key management and WebSocket URL construction.
 * The main getApiBase() / setApiBase() live in api.ts for backward compatibility.
 */

const API_KEY_SESSION_KEY = 'heramind_api_key'

// Initialize from sessionStorage (survives page refresh, cleared on tab close)
let _currentApiKey = (() => {
  try { return sessionStorage.getItem(API_KEY_SESSION_KEY) || '' } catch { return '' }
})()

/**
 * Set the API key to use for all requests (remote instance auth).
 * Persisted to sessionStorage so it survives page refresh within the same tab.
 */
export function setApiKey(key: string): void {
  _currentApiKey = key
  try {
    if (key) {
      sessionStorage.setItem(API_KEY_SESSION_KEY, key)
    } else {
      sessionStorage.removeItem(API_KEY_SESSION_KEY)
    }
  } catch { /* ignore */ }
}

/**
 * Clear the API key (switch back to local instance).
 */
export function clearApiKey(): void {
  _currentApiKey = ''
  try { sessionStorage.removeItem(API_KEY_SESSION_KEY) } catch { /* ignore */ }
}

/**
 * Get the current API key (empty string if none).
 */
export function getApiKey(): string {
  return _currentApiKey
}

/**
 * Construct a WebSocket URL from a server origin and path.
 * Handles http→ws and https→wss conversion.
 */
export function buildWsUrl(serverOrigin: string, path: string): string {
  const wsOrigin = serverOrigin
    .replace(/^http:/, 'ws:')
    .replace(/^https:/, 'wss:')
  return `${wsOrigin}${path}`
}

/**
 * Reset to default (local) instance: clear API key.
 * URL reset is done via setApiBase('') in api.ts.
 */
export function resetToDefault(): void {
  _currentApiKey = ''
}
