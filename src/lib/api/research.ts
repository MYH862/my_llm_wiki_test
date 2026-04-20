import apiClient from "../api-client"

export interface ResearchTask {
  id: number
  project_id: number
  query: string
  status: "pending" | "running" | "completed" | "failed"
  result?: string
  created_at: string
  completed_at?: string
}

export interface CreateResearchTaskRequest {
  project_id: number
  query: string
}

export async function createResearchTask(request: CreateResearchTaskRequest): Promise<ResearchTask> {
  const response = await apiClient.post("/research/create", request)
  return response.data
}

export async function getResearchTask(taskId: number): Promise<ResearchTask> {
  const response = await apiClient.get(`/research/${taskId}`)
  return response.data
}

export async function listResearchTasks(projectId: number): Promise<ResearchTask[]> {
  const response = await apiClient.get(`/research/list/${projectId}`)
  return response.data
}
