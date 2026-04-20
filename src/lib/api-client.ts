import axios, { type AxiosInstance, type AxiosError } from "axios"

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://localhost:3000/api"

export interface AuthTokens {
  access_token: string
  refresh_token: string
}

export interface UserInfo {
  id: number
  username: string
  email: string | null
  display_name: string | null
}

export interface LoginResponse {
  user: UserInfo
  access_token: string
  refresh_token: string
}

export interface RegisterResponse {
  user: UserInfo
  access_token: string
  refresh_token: string
}

export interface RefreshResponse {
  access_token: string
  refresh_token: string
}

class ApiClient {
  private client: AxiosInstance
  private refreshPromise: Promise<string> | null = null

  constructor() {
    this.client = axios.create({
      baseURL: API_BASE_URL,
      timeout: 30000,
      headers: {
        "Content-Type": "application/json",
      },
    })

    this.setupInterceptors()
  }

  private setupInterceptors() {
    this.client.interceptors.request.use(
      (config) => {
        const token = this.getAccessToken()
        if (token) {
          config.headers.Authorization = `Bearer ${token}`
        }
        return config
      },
      (error) => Promise.reject(error)
    )

    this.client.interceptors.response.use(
      (response) => response,
      async (error: AxiosError) => {
        const originalRequest = error.config as any
        if (error.response?.status === 401 && !originalRequest._retry) {
          originalRequest._retry = true

          try {
            const newToken = await this.refreshAccessToken()
            originalRequest.headers.Authorization = `Bearer ${newToken}`
            return this.client(originalRequest)
          } catch (refreshError) {
            this.clearAuth()
            window.location.href = "/login"
            return Promise.reject(refreshError)
          }
        }
        return Promise.reject(error)
      }
    )
  }

  private getAccessToken(): string | null {
    return localStorage.getItem("access_token")
  }

  private getRefreshToken(): string | null {
    return localStorage.getItem("refresh_token")
  }

  private async refreshAccessToken(): Promise<string> {
    if (this.refreshPromise) {
      return this.refreshPromise
    }

    this.refreshPromise = new Promise(async (resolve, reject) => {
      try {
        const refreshToken = this.getRefreshToken()
        if (!refreshToken) {
          reject(new Error("No refresh token"))
          return
        }

        const response = await axios.post<RefreshResponse>(
          `${API_BASE_URL}/auth/refresh`,
          { refresh_token: refreshToken }
        )

        const { access_token, refresh_token } = response.data
        localStorage.setItem("access_token", access_token)
        localStorage.setItem("refresh_token", refresh_token)
        resolve(access_token)
      } catch (error) {
        reject(error)
      } finally {
        this.refreshPromise = null
      }
    })

    return this.refreshPromise
  }

  setAccessToken(token: string) {
    localStorage.setItem("access_token", token)
  }

  setRefreshToken(token: string) {
    localStorage.setItem("refresh_token", token)
  }

  clearAuth() {
    localStorage.removeItem("access_token")
    localStorage.removeItem("refresh_token")
    localStorage.removeItem("user")
  }

  isAuthenticated(): boolean {
    return !!this.getAccessToken()
  }

  getClient(): AxiosInstance {
    return this.client
  }
}

export const apiClient = new ApiClient()
export default apiClient.getClient()
