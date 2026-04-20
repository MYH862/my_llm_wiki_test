import { useState, useCallback } from "react"
import { useAuthStore } from "@/stores/auth-store"

export interface Permission {
  id: number
  name: string
  resource: string
  action: string
}

export interface Role {
  id: number
  name: string
  permissions: Permission[]
}

export interface ProjectMember {
  user_id: number
  username: string
  role_id: number
  role_name: string
}

export function usePermission() {
  const user = useAuthStore((s) => s.user)

  const hasPermission = useCallback(
    (resource: string, action: string): boolean => {
      if (!user) return false
      const roles = (user as any).roles as Role[] | undefined
      if (!roles) return false

      return roles.some((role) =>
        role.permissions.some(
          (perm) => perm.resource === resource && perm.action === action
        )
      )
    },
    [user]
  )

  const hasRole = useCallback(
    (roleName: string): boolean => {
      if (!user) return false
      const roles = (user as any).roles as Role[] | undefined
      if (!roles) return false

      return roles.some((role) => role.name === roleName)
    },
    [user]
  )

  return { hasPermission, hasRole }
}

export function useProjectPermissions(projectId: number) {
  const [members, setMembers] = useState<ProjectMember[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadMembers = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const response = await fetch(
        `${import.meta.env.VITE_API_BASE_URL || "http://localhost:3000/api"}/projects/${projectId}/members`,
        {
          headers: {
            Authorization: `Bearer ${localStorage.getItem("access_token")}`,
          },
        }
      )
      if (!response.ok) {
        throw new Error("Failed to load members")
      }
      const data = await response.json()
      setMembers(data)
    } catch (err: any) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [projectId])

  const addMember = useCallback(
    async (userId: number, roleId: number) => {
      const response = await fetch(
        `${import.meta.env.VITE_API_BASE_URL || "http://localhost:3000/api"}/projects/${projectId}/members`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${localStorage.getItem("access_token")}`,
          },
          body: JSON.stringify({ user_id: userId, role_id: roleId }),
        }
      )
      if (!response.ok) {
        throw new Error("Failed to add member")
      }
      await loadMembers()
    },
    [projectId, loadMembers]
  )

  const removeMember = useCallback(
    async (userId: number) => {
      const response = await fetch(
        `${import.meta.env.VITE_API_BASE_URL || "http://localhost:3000/api"}/projects/${projectId}/members/${userId}`,
        {
          method: "DELETE",
          headers: {
            Authorization: `Bearer ${localStorage.getItem("access_token")}`,
          },
        }
      )
      if (!response.ok) {
        throw new Error("Failed to remove member")
      }
      await loadMembers()
    },
    [projectId, loadMembers]
  )

  const updateMemberRole = useCallback(
    async (userId: number, roleId: number) => {
      const response = await fetch(
        `${import.meta.env.VITE_API_BASE_URL || "http://localhost:3000/api"}/projects/${projectId}/members/${userId}`,
        {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${localStorage.getItem("access_token")}`,
          },
          body: JSON.stringify({ role_id: roleId }),
        }
      )
      if (!response.ok) {
        throw new Error("Failed to update member role")
      }
      await loadMembers()
    },
    [projectId, loadMembers]
  )

  return {
    members,
    loading,
    error,
    loadMembers,
    addMember,
    removeMember,
    updateMemberRole,
  }
}
