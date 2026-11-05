import { beforeEach, describe, expect, it } from 'vitest'
import { tokenManager } from './tokenManager'

describe('NeoMind dashboard extension authentication', () => {
  beforeEach(() => { localStorage.clear(); sessionStorage.clear() })

  it('supports existing sessions without changing remember-me storage', () => {
    sessionStorage.setItem('heramind_token_session', 'session-token')
    expect(tokenManager.getToken()).toBe('session-token')
    expect(sessionStorage.getItem('neomind_token_session')).toBe('session-token')
    expect(localStorage.getItem('neomind_token')).toBeNull()
  })

  it('updates aliases on login and removes the previous account alias', () => {
    tokenManager.setToken('remembered-token', true)
    expect(localStorage.getItem('neomind_token')).toBe('remembered-token')
    tokenManager.setToken('new-session-token', false)
    expect(localStorage.getItem('neomind_token')).toBeNull()
    expect(sessionStorage.getItem('neomind_token_session')).toBe('new-session-token')
  })

  it('removes legacy credentials after logout or session expiry', () => {
    tokenManager.setToken('token', true)
    tokenManager.clearToken()
    expect(localStorage.getItem('neomind_token')).toBeNull()
    sessionStorage.setItem('neomind_token_session', 'stale-token')
    expect(tokenManager.getToken()).toBeNull()
    expect(sessionStorage.getItem('neomind_token_session')).toBeNull()
    const expired = `header.${btoa(JSON.stringify({ exp: 1 }))}.signature`
    tokenManager.setToken(expired, true)
    expect(tokenManager.getToken()).toBeNull()
    expect(localStorage.getItem('neomind_token')).toBeNull()
  })
})
