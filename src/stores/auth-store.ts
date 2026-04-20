import { create } from "zustand"
import { login, register, logout, getCurrentUser, setCurrentUser, clearCurrentUser } from "@/lib/api/auth"
import type { UserInfo } from "@/lib/api-client"

interface AuthState {
  user: UserInfo | null
  isAuthenticated: boolean
  isLoading: boolean
  error: string | null
  
  login: (username: string, password: string) => Promise<void>
  register: (username: string, password: string, email?: string, displayName?: string) => Promise<void>
  logout: () => Promise<void>
  initialize: () => void
  clearError: () => void
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  isAuthenticated: false,
  isLoading: false,
  error: null,

  initialize: () => {
    const user = getCurrentUser()
    const token = localStorage.getItem("access_token")
    if (user && token) {
      set({ user, isAuthenticated: true })
    }
  },

  login: async (username: string, password: string) => {
    set({ isLoading: true, error: null })
    try {
      const response = await login({ username, password })
      localStorage.setItem("access_token", response.access_token)
      localStorage.setItem("refresh_token", response.refresh_token)
      setCurrentUser(response.user)
      set({ user: response.user, isAuthenticated: true, isLoading: false })
    } catch (error: any) {
      const message = error.response?.data?.error || "登录失败"
      set({ error: message, isLoading: false })
      throw error
    }
  },

  register: async (username: string, password: string, email?: string, displayName?: string) => {
    set({ isLoading: true, error: null })
    try {
      const response = await register({ username, email, display_name: displayName, password })
      localStorage.setItem("access_token", response.access_token)
      localStorage.setItem("refresh_token", response.refresh_token)
      setCurrentUser(response.user)
      set({ user: response.user, isAuthenticated: true, isLoading: false })
    } catch (error: any) {
      const message = error.response?.data?.error || "注册失败"
      set({ error: message, isLoading: false })
      throw error
    }
  },

  logout: async () => {
    const refreshToken = localStorage.getItem("refresh_token")
    try {
      if (refreshToken) {
        await logout(refreshToken)
      }
    } catch {
      // Ignore logout API errors
    } finally {
      localStorage.removeItem("access_token")
      localStorage.removeItem("refresh_token")
      clearCurrentUser()
      set({ user: null, isAuthenticated: false })
    }
  },

  clearError: () => set({ error: null }),
}))
