// ========== User Authentication Types ==========

export type UserRole = 'admin' | 'user' | 'viewer'

export interface UserInfo {
  id: string
  username: string
  role: UserRole
  created_at: number
}

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  token: string
  user: UserInfo
}

export interface RegisterRequest {
  username: string
  password: string
  role?: UserRole
}

export interface ChangePasswordRequest {
  old_password: string
  new_password: string
}
