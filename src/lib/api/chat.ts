import apiClient from "../api-client"

export interface ChatMessage {
  role: "user" | "assistant" | "system"
  content: string
}

export interface ChatRequest {
  project_id: number
  messages: ChatMessage[]
  llm_config_id?: number
}

export interface ChatResponse {
  content: string
}

export async function sendChatMessage(request: ChatRequest): Promise<ChatResponse> {
  const response = await apiClient.post("/chat/send", request)
  return response.data
}

export async function sendChatMessageStream(
  request: ChatRequest,
  onChunk: (chunk: string) => void,
  onComplete: () => void,
  onError: (error: Error) => void
): Promise<void> {
  const token = localStorage.getItem("access_token")
  
  try {
    const response = await fetch(`${import.meta.env.VITE_API_BASE_URL || "http://localhost:3000/api"}/chat/stream`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(request),
    })

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }

    const reader = response.body?.getReader()
    if (!reader) {
      throw new Error("No response body")
    }

    const decoder = new TextDecoder()
    let buffer = ""

    while (true) {
      const { done, value } = await reader.read()
      
      if (done) {
        onComplete()
        break
      }

      buffer += decoder.decode(value, { stream: true })
      const lines = buffer.split("\n")
      buffer = lines.pop() || ""

      for (const line of lines) {
        if (line.startsWith("data: ")) {
          const data = line.slice(6)
          if (data === "[DONE]") {
            onComplete()
            return
          }
          try {
            const parsed = JSON.parse(data)
            if (parsed.content) {
              onChunk(parsed.content)
            }
          } catch {
            // Skip invalid JSON
          }
        }
      }
    }
  } catch (error) {
    onError(error instanceof Error ? error : new Error(String(error)))
  }
}
