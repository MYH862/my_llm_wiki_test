# Phase 3 迁移指南

## 概述

Phase 3 已完成前端改造，包括 API 客户端层、认证 UI、组件调用方式改造和权限 UI。

## 新增文件

### API 客户端层
- `src/lib/api-client.ts` - 核心 HTTP 客户端（Axios + JWT 拦截器）
- `src/lib/api/` - API 服务模块目录
  - `auth.ts` - 认证相关 API
  - `files.ts` - 文件操作 API
  - `projects.ts` - 项目管理 API
  - `chat.ts` - 聊天 API（支持 SSE 流式）
  - `vector.ts` - 向量搜索 API
  - `graph.ts` - 知识图谱 API
  - `research.ts` - 深度研究 API
  - `review.ts` - 审核 API
  - `lint.ts` - Lint 检查 API
  - `permissions.ts` - 权限管理 API
  - `adapter.ts` - Tauri 命令适配器层
  - `index.ts` - 统一导出

### 认证系统
- `src/stores/auth-store.ts` - 认证状态管理（Zustand）
- `src/components/auth/` - 认证组件目录
  - `login-page.tsx` - 登录页面
  - `register-page.tsx` - 注册页面
  - `auth-router.tsx` - 认证路由容器
  - `protected-route.tsx` - 路由守卫

### 权限管理
- `src/components/settings/project-members.tsx` - 项目成员管理组件

## 如何使用

### 1. 认证流程

应用启动时会自动检查认证状态：

```typescript
// App.tsx 已集成认证流程
const isAuthenticated = useAuthStore((s) => s.isAuthenticated)

if (!isAuthenticated) {
  return <AuthRouter /> // 显示登录/注册页面
}
```

### 2. 调用 API

#### 方式一：直接使用 API 服务模块（推荐）

```typescript
import { login, readFile, listDirectory } from "@/lib/api"

// 登录
const response = await login({ username: "admin", password: "admin123" })

// 读取文件（需要 project ID）
const content = await readFile(projectId, filePath)

// 列出目录
const files = await listDirectory(projectId, dirPath)
```

#### 方式二：使用适配器层（兼容旧代码）

```typescript
import { readFile, writeFile, listDirectory } from "@/lib/api/adapter"

// 打开项目时自动设置当前项目
const project = await openProject(path)

// 后续调用自动使用当前项目
const content = await readFile(filePath)
```

### 3. 权限检查

```typescript
import { usePermission, useProjectPermissions } from "@/lib/api/permissions"

// 全局权限检查
const { hasPermission, hasRole } = usePermission()
if (hasPermission("files", "read")) {
  // 有权限执行操作
}

// 项目成员管理
const { members, loadMembers, addMember, removeMember } = useProjectPermissions(projectId)
```

### 4. 流式聊天

```typescript
import { sendChatMessageStream } from "@/lib/api/chat"

sendChatMessageStream(
  { project_id: projectId, messages: [...] },
  (chunk) => console.log("Received:", chunk),
  () => console.log("Complete"),
  (error) => console.error("Error:", error)
)
```

## 环境变量配置

创建 `.env` 文件配置 API 地址：

```env
VITE_API_BASE_URL=http://localhost:3000/api
```

## 后续工作

### 需要手动替换的组件

以下组件仍使用旧的 Tauri 命令调用，需要逐步替换为新的 API：

1. **文件操作组件**
   - 将 `import { readFile } from "@/commands/fs"` 
   - 替换为 `import { readFile } from "@/lib/api/adapter"`

2. **项目操作组件**
   - 将 `import { openProject } from "@/commands/fs"`
   - 替换为 `import { openProject } from "@/lib/api/adapter"`

3. **设置页面**
   - 使用 `src/lib/api/projects.ts` 中的 API
   - 替换 Tauri Store 为服务端 API 调用

## 测试

### 启动后端服务

```bash
cd server
docker-compose up -d postgres minio qdrant
cargo run
```

### 启动前端

```bash
npm run dev
```

### 验证认证流程

1. 访问 http://localhost:5173
2. 应该看到登录页面
3. 使用 admin/admin123 登录
4. 登录后应该看到项目选择界面

## 注意事项

1. **JWT 令牌自动刷新**：当 access_token 过期时，系统会自动使用 refresh_token 刷新
2. **认证失败重定向**：刷新令牌失败时会自动重定向到登录页
3. **项目 ID 类型**：后端使用 UUID，前端使用 string 类型
4. **CORS 配置**：确保后端 CORS 配置包含前端地址

## 技术栈

- **HTTP 客户端**: Axios
- **状态管理**: Zustand
- **流式响应**: Server-Sent Events (SSE)
- **认证**: JWT (access_token + refresh_token)
- **权限**: RBAC (基于角色的访问控制)
