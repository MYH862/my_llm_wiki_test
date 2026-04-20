import apiClient from "../api-client"

export interface ReviewItem {
  id: number
  project_id: number
  file_path: string
  status: "pending" | "approved" | "rejected"
  comment?: string
  created_at: string
  updated_at: string
}

export interface CreateReviewItemRequest {
  project_id: number
  file_path: string
}

export interface UpdateReviewItemRequest {
  status: "approved" | "rejected"
  comment?: string
}

export async function createReviewItem(request: CreateReviewItemRequest): Promise<ReviewItem> {
  const response = await apiClient.post("/review/create", request)
  return response.data
}

export async function updateReviewItem(
  itemId: number,
  request: UpdateReviewItemRequest
): Promise<ReviewItem> {
  const response = await apiClient.put(`/review/${itemId}`, request)
  return response.data
}

export async function deleteReviewItem(itemId: number): Promise<void> {
  await apiClient.delete(`/review/${itemId}`)
}

export async function listReviewItems(projectId: number): Promise<ReviewItem[]> {
  const response = await apiClient.get(`/review/list/${projectId}`)
  return response.data
}
