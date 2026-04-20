import apiClient from "../api-client"

export interface VectorSearchRequest {
  project_id: number
  query: string
  top_k?: number
}

export interface VectorSearchResult {
  file_path: string
  content: string
  score: number
}

export interface EmbeddingRequest {
  project_id: number
  text: string
}

export interface EmbeddingResponse {
  embedding: number[]
}

export async function searchVectors(request: VectorSearchRequest): Promise<VectorSearchResult[]> {
  const response = await apiClient.post("/vector/search", request)
  return response.data
}

export async function getEmbedding(request: EmbeddingRequest): Promise<number[]> {
  const response = await apiClient.post("/vector/embed", request)
  return response.data.embedding
}

export async function upsertVector(
  projectId: number,
  filePath: string,
  content: string
): Promise<void> {
  await apiClient.post("/vector/upsert", {
    project_id: projectId,
    file_path: filePath,
    content,
  })
}

export async function deleteVector(projectId: number, filePath: string): Promise<void> {
  await apiClient.post("/vector/delete", {
    project_id: projectId,
    file_path: filePath,
  })
}
