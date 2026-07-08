/**
 * Unified Token Manager
 *
 * Centralized JWT token management for authentication.
 */

import type { UserInfo } from '@/types'

// ============================================================================
// Constants
// ============================================================================

const TOKEN_KEY = 'heramind_token'
const TOKEN_KEY_SESSION = 'heramind_token_session'
const USER_KEY = 'heramind_user'
const USER_KEY_SESSION = 'heramind_user_session'

// ============================================================================
// Token Manager
// ============================================================================

class TokenManagerClass {
  /**
   * Decode JWT payload without a library.
   */
  private parseJwtPayload(token: string): { exp?: number; [k: string]: unknown } | null {
    try {
      const part = token.split('.')[1]
      if (!part) return null
      const json = atob(part.replace(/-/g, '+').replace(/_/g, '/'))
      return JSON.parse(json)
    } catch {
      return null
    }
  }

  /**
   * Check if a JWT token is expired.
   * Returns true if the token has no exp claim or is past its expiration.
   */
  private isTokenExpired(token: string): boolean {
    const payload = this.parseJwtPayload(token)
    if (!payload?.exp) return false // no exp claim — trust the server to reject
    // 30s buffer to account for clock skew
    return Date.now() >= (payload.exp - 30) * 1000
  }

  /**
   * Get the current authentication token from storage.
   * Returns null if the token is expired (auto-clears stale tokens).
   */
  getToken(): string | null {
    const token = localStorage.getItem(TOKEN_KEY) || sessionStorage.getItem(TOKEN_KEY_SESSION)
    if (!token) return null
    if (this.isTokenExpired(token)) {
      this.clearToken()
      return null
    }
    return token
  }

  /**
   * Store the authentication token.
   * @param token - The JWT token to store
   * @param remember - If true, uses localStorage (persists across sessions).
   *                   If false, uses sessionStorage (cleared on browser close).
   */
  setToken(token: string, remember: boolean = false): void {
    if (remember) {
      localStorage.setItem(TOKEN_KEY, token)
      sessionStorage.removeItem(TOKEN_KEY_SESSION)
    } else {
      sessionStorage.setItem(TOKEN_KEY_SESSION, token)
      localStorage.removeItem(TOKEN_KEY)
    }
  }

  /**
   * Clear the authentication token from all storage locations.
   */
  clearToken(): void {
    localStorage.removeItem(TOKEN_KEY)
    sessionStorage.removeItem(TOKEN_KEY_SESSION)
  }

  /**
   * Check if a token exists in any storage location.
   */
  hasToken(): boolean {
    return !!(
      localStorage.getItem(TOKEN_KEY) ||
      sessionStorage.getItem(TOKEN_KEY_SESSION)
    )
  }

  // ========================================================================
  // User Info Management
  // ========================================================================

  /**
   * Get the current user information from storage.
   */
  getUser(): UserInfo | null {
    const userStr = localStorage.getItem(USER_KEY) || sessionStorage.getItem(USER_KEY_SESSION)

    if (userStr) {
      try {
        return JSON.parse(userStr)
      } catch {
        return null
      }
    }
    return null
  }

  /**
   * Store user information.
   * @param user - The user info to store
   * @param remember - If true, uses localStorage; otherwise sessionStorage
   */
  setUser(user: UserInfo, remember: boolean = false): void {
    const userStr = JSON.stringify(user)
    if (remember) {
      localStorage.setItem(USER_KEY, userStr)
      sessionStorage.removeItem(USER_KEY_SESSION)
    } else {
      sessionStorage.setItem(USER_KEY_SESSION, userStr)
      localStorage.removeItem(USER_KEY)
    }
  }

  /**
   * Clear user information from all storage locations.
   */
  clearUser(): void {
    localStorage.removeItem(USER_KEY)
    sessionStorage.removeItem(USER_KEY_SESSION)
  }

  /**
   * Clear both token and user information.
   * Convenience method for logout.
   */
  clearAll(): void {
    this.clearToken()
    this.clearUser()
  }
}

// ============================================================================
// Singleton Instance
// ============================================================================

export const tokenManager = new TokenManagerClass()

// ============================================================================
// Convenience Exports
// ============================================================================

export const {
  getToken,
  setToken,
  clearToken,
  hasToken,
  getUser,
  setUser,
  clearUser,
  clearAll,
} = tokenManager
