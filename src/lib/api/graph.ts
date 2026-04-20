import apiClient from "../api-client"

export interface GraphNode {
  id: string
  label: string
  community?: number
}

export interface GraphEdge {
  source: string
  target: string
  weight: number
}

export interface GraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

export interface GraphInsight {
  type: "unexpected_connection" | "knowledge_gap"
  description: string
  nodes?: string[]
}

export async function getGraphData(projectId: number): Promise<GraphData> {
  const response = await apiClient.get(`/graph/${projectId}`)
  return response.data
}

export async function runLouvainDetection(projectId: number): Promise<void> {
  await apiClient.post(`/graph/${projectId}/louvain`)
}

export async function getGraphInsights(projectId: number): Promise<GraphInsight[]> {
  const response = await apiClient.get(`/graph/${projectId}/insights`)
  return response.data
}
