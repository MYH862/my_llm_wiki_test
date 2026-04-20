import { readFile as apiReadFile, writeFile as apiWriteFile, listDirectory as apiListDirectory, deleteFile as apiDeleteFile, copyFile as apiCopyFile, preprocessFile as apiPreprocessFile } from "./files"
import { openProject as apiOpenProject, createProject as apiCreateProject } from "./projects"
import type { FileNode, WikiProject } from "@/types/wiki"

let currentProjectId: string | null = null
let currentProjectPath: string | null = null

export function setCurrentProject(id: string, path: string) {
  currentProjectId = id
  currentProjectPath = path
}

function requireProject(): { id: string; path: string } {
  if (!currentProjectId || !currentProjectPath) {
    throw new Error("No project opened")
  }
  return { id: currentProjectId, path: currentProjectPath }
}

export async function readFile(path: string): Promise<string> {
  const { id } = requireProject()
  return apiReadFile(id, path)
}

export async function writeFile(path: string, contents: string): Promise<void> {
  const { id } = requireProject()
  return apiWriteFile(id, path, contents)
}

export async function listDirectory(path: string): Promise<FileNode[]> {
  const { id } = requireProject()
  return apiListDirectory(id, path)
}

export async function deleteFile(path: string): Promise<void> {
  const { id } = requireProject()
  return apiDeleteFile(id, path)
}

export async function copyFile(source: string, destination: string): Promise<void> {
  const { id } = requireProject()
  return apiCopyFile(id, source, destination)
}

export async function preprocessFile(path: string): Promise<string> {
  const { id } = requireProject()
  return apiPreprocessFile(id, path)
}

export async function createProject(name: string, path: string): Promise<WikiProject> {
  return apiCreateProject({ name, path })
}

export async function openProject(path: string): Promise<WikiProject> {
  const project = await apiOpenProject(path)
  setCurrentProject(project.id.toString(), path)
  return project
}
