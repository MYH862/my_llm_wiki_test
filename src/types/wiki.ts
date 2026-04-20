export interface WikiProject {
  id: number | string
  name: string
  path: string
  created_at?: string
  updated_at?: string
}

export interface FileNode {
  name: string
  path: string
  is_dir: boolean
  children?: FileNode[]
}

export interface WikiPage {
  path: string
  content: string
  frontmatter: Record<string, unknown>
}
