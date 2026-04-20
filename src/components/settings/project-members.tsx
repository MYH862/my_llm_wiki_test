import { useEffect, useState } from "react"
import { useProjectPermissions } from "@/lib/api/permissions"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"

interface ProjectMembersProps {
  projectId: number
}

export function ProjectMembers({ projectId }: ProjectMembersProps) {
  const { members, loading, error, loadMembers, removeMember } = useProjectPermissions(projectId)
  const [showAddDialog, setShowAddDialog] = useState(false)

  useEffect(() => {
    loadMembers()
  }, [loadMembers])

  if (loading) {
    return <div className="p-4 text-center text-muted-foreground">加载中...</div>
  }

  if (error) {
    return <div className="p-4 text-center text-destructive">加载失败: {error}</div>
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">项目成员</h3>
        <Button onClick={() => setShowAddDialog(true)}>添加成员</Button>
      </div>

      <ScrollArea className="h-[400px]">
        <div className="space-y-2">
          {members.map((member) => (
            <div
              key={member.user_id}
              className="flex items-center justify-between rounded-md border p-3"
            >
              <div>
                <p className="font-medium">{member.username}</p>
                <p className="text-sm text-muted-foreground">{member.role_name}</p>
              </div>
              <Button
                variant="destructive"
                size="sm"
                onClick={() => removeMember(member.user_id)}
              >
                移除
              </Button>
            </div>
          ))}

          {members.length === 0 && (
            <div className="py-8 text-center text-muted-foreground">
              暂无成员
            </div>
          )}
        </div>
      </ScrollArea>
    </div>
  )
}
