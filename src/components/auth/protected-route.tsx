import { useEffect } from "react"
import { useAuthStore } from "@/stores/auth-store"

interface ProtectedRouteProps {
  children: React.ReactNode
}

export function ProtectedRoute({ children }: ProtectedRouteProps) {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated)
  const initialize = useAuthStore((s) => s.initialize)

  useEffect(() => {
    initialize()
  }, [initialize])

  if (!isAuthenticated) {
    window.location.href = "/login"
    return null
  }

  return <>{children}</>
}
