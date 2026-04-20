import apiClient from "../api-client"

export interface LintIssue {
  type: "structure" | "semantic"
  severity: "error" | "warning"
  message: string
  file_path?: string
  line?: number
}

export interface LintResult {
  issues: LintIssue[]
}

export async function runLint(projectId: number): Promise<LintResult> {
  const response = await apiClient.post(`/lint/run/${projectId}`)
  return response.data
}

export async function getLintResults(projectId: number): Promise<LintResult> {
  const response = await apiClient.get(`/lint/results/${projectId}`)
  return response.data
}

export async function clearLintResults(projectId: number): Promise<void> {
  await apiClient.delete(`/lint/results/${projectId}`)
}
