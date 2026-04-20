import apiClient from "../api-client"

export interface FileNode {
  name: string
  path: string
  is_directory: boolean
  children?: FileNode[]
}

export interface DirectoryEntry {
  path: string
  entries: FileNode[]
}

export async function readFile(projectId: string, filePath: string): Promise<string> {
  const response = await apiClient.get(`/files/${projectId}/read`, {
    params: { path: filePath },
  })
  return response.data.content
}

export async function writeFile(
  projectId: string,
  filePath: string,
  contents: string
): Promise<void> {
  await apiClient.post(`/files/${projectId}/write`, {
    path: filePath,
    content: contents,
  })
}

export async function listDirectory(projectId: string, dirPath?: string): Promise<FileNode[]> {
  const response = await apiClient.get(`/files/${projectId}/list`, {
    params: { path: dirPath },
  })
  return response.data.entries || []
}

export async function deleteFile(projectId: string, filePath: string): Promise<void> {
  await apiClient.delete(`/files/${projectId}/delete`, {
    params: { path: filePath },
  })
}

export async function copyFile(
  projectId: string,
  source: string,
  destination: string
): Promise<void> {
  await apiClient.post(`/files/${projectId}/copy`, {
    from_path: source,
    to_path: destination,
  })
}

export async function createDirectory(projectId: string, dirPath: string): Promise<void> {
  await apiClient.post(`/files/${projectId}/write`, {
    path: `${dirPath}/.keep`,
    content: "",
  })
}

export async function preprocessFile(projectId: string, filePath: string): Promise<string> {
  const response = await apiClient.get(`/files/${projectId}/preprocess`, {
    params: { path: filePath },
  })
  return response.data.content
}
