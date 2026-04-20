import apiClient from "../api-client"
import type { LoginResponse, RegisterResponse, UserInfo } from "../api-client"

export interface LoginCredentials {
  username: string
  password: string
}

export interface RegisterCredentials {
  username: string
  email?: string
  display_name?: string
  password: string
}

export interface ChangePasswordRequest {
  current_password: string
  new_password: string
}

export async function login(credentials: LoginCredentials): Promise<LoginResponse> {
  const response = await apiClient.post<LoginResponse>("/auth/login", credentials)
  return response.data
}

export async function register(credentials: RegisterCredentials): Promise<RegisterResponse> {
  const response = await apiClient.post<RegisterResponse>("/auth/register", credentials)
  return response.data
}

export async function logout(refreshToken: string): Promise<void> {
  await apiClient.post("/auth/logout", { refresh_token: refreshToken })
}

export async function changePassword(request: ChangePasswordRequest): Promise<void> {
  await apiClient.put("/auth/password", request)
}

export function getCurrentUser(): UserInfo | null {
  const userStr = localStorage.getItem("user")
  if (!userStr) return null
  try {
    return JSON.parse(userStr)
  } catch {
    return null
  }
}

export function setCurrentUser(user: UserInfo): void {
  localStorage.setItem("user", JSON.stringify(user))
}

export function clearCurrentUser(): void {
  localStorage.removeItem("user")
}
