import apiClient from "../api-client"
import type { FileNode } from "./files"

export interface WikiProject {
  id: number
  name: string
  path: string
  created_at: string
  updated_at: string
  file_tree?: FileNode[]
}

export interface CreateProjectRequest {
  name: string
  path: string
}

export interface ProjectStats {
  total_files: number
  total_wikilinks: number
  total_vectors: number
  total_nodes: number
  total_edges: number
}

export interface ProjectSettings {
  id: number
  project_id: number
  key: string
  value: string
}

export async function createProject(request: CreateProjectRequest): Promise<WikiProject> {
  const response = await apiClient.post("/projects/create", request)
  return response.data
}

export async function openProject(path: string): Promise<WikiProject> {
  const response = await apiClient.post("/projects/open", { path })
  return response.data
}

export async function listProjects(): Promise<WikiProject[]> {
  const response = await apiClient.get("/projects/list")
  return response.data
}

export async function getProjectStats(projectId: number): Promise<ProjectStats> {
  const response = await apiClient.get(`/projects/${projectId}/stats`)
  return response.data
}

export async function getProjectSettings(projectId: number): Promise<ProjectSettings[]> {
  const response = await apiClient.get(`/projects/${projectId}/settings`)
  return response.data
}

export async function updateProjectSetting(
  projectId: number,
  key: string,
  value: string
): Promise<ProjectSettings> {
  const response = await apiClient.post(`/projects/${projectId}/settings`, {
    key,
    value,
  })
  return response.data
}
