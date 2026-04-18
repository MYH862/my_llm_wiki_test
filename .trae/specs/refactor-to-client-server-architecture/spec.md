# 重构为客户端-服务器架构规范

## Why

当前项目是一个基于 Tauri 的前后端一体化桌面应用，所有业务逻辑（文件操作、LLM 调用、向量存储、网页剪辑服务器等）都运行在本地。为了支持多用户协作、集中化管理、权限控制以及更灵活的部署方式，需要将应用拆分为独立的后端服务和前端桌面应用。

## What Changes

- **BREAKING**: 移除 Tauri 桌面应用架构，后端服务独立部署到服务器
- **BREAKING**: 前端从 Tauri WebView 改为纯前端容器（移除所有 Tauri 后端 Rust 代码，保留 WebView 功能）
- 新增用户认证系统（登录/注册/JWT）
- 新增基于角色的权限控制（RBAC）
- 所有后端功能通过 RESTful API 暴露
- 前端通过 HTTP API 调用后端服务
- 保留所有现有功能，但重构为服务端架构

## Impact

- **Affected specs**: 文件管理、LLM 集成、知识图谱、向量搜索、网页剪辑、深度研究、审核系统、聊天系统
- **Affected code**: 
  - 后端: `server/src/` 独立 Rust Web 服务项目，`src-tauri/src/` 中后端逻辑需迁移到 server
  - 前端: `src/` 中所有调用 Tauri API 的代码需要改为 HTTP API 调用
  - 前端: `src-tauri/src/` 仅保留 Tauri WebView 容器代码（移除命令处理逻辑）
  - 存储: 从 Tauri Store 改为 IndexedDB + 服务端同步
  - 部署: 后端从 Tauri 打包改为 Docker 容器化部署

## 现有项目功能点清单

### 核心功能
1. **项目管理**: 创建、打开、切换 Wiki 项目
2. **文件操作**: 读取、写入、删除、复制文件，列出目录
3. **文档预处理**: PDF、DOCX、PPTX、XLSX 等格式解析
4. **两步思维链导入**: LLM 分析源文档 → 生成 Wiki 页面
5. **持久化导入队列**: 串行处理、崩溃恢复、取消、重试
6. **文件夹导入**: 递归导入，保留目录结构
7. **聊天系统**: 多会话、持久化、引用面板、重新生成、保存到 Wiki
8. **知识图谱**: 4 信号相关性模型、Louvain 社区检测、可视化
9. **图谱洞察**: 意外连接检测、知识缺口识别
10. **深度研究**: Web 搜索（Tavily API）、LLM 综合、自动导入
11. **审核系统**: 异步人工审核队列
12. **向量语义搜索**: LanceDB 嵌入存储和检索
13. **Lint 检查**: Wiki 健康检查
14. **Chrome 网页剪辑器**: 浏览器扩展 + 本地 HTTP API（端口 19827）
15. **设置管理**: LLM 配置、语言、上下文窗口大小
16. **i18n**: 中英文支持
17. **Markdown 编辑器**: Milkdown WYSIWYG 编辑器
18. **KaTeX 数学渲染**: LaTeX 公式支持
19. **思维链显示**: 展开/折叠 reasoning 块

### 需要改造的功能

#### 1. 后端服务化（高优先级）
- **改造内容**: 将 Tauri Rust 后端重构为独立的 Web API 服务
- **技术方案**: 使用 Rust + Actix-web/Axum 框架
- **API 设计**: RESTful API + WebSocket（用于流式响应）
- **部署方式**: Docker 容器化

#### 2. 用户认证系统（高优先级）
- **改造内容**: 新增用户注册、登录、JWT 令牌管理
- **技术方案**: JWT + bcrypt 密码哈希
- **功能**: 登录、注册、密码重置、令牌刷新

#### 3. 权限控制系统（高优先级）
- **改造内容**: 基于角色的访问控制（RBAC）
- **角色定义**:
  - **管理员**: 所有权限，包括用户管理、系统配置
  - **编辑者**: 可以创建/编辑 Wiki、导入文档、使用聊天和研究功能
  - **查看者**: 只能查看 Wiki、知识图谱、搜索结果
  - **自定义角色**: 可配置具体权限
- **权限粒度**:
  - 项目级别权限（哪些用户可以访问哪些项目）
  - 功能级别权限（哪些功能可以使用）
  - 操作级别权限（读/写/删除）

#### 4. 前端改造（高优先级）
- **改造内容**: 移除 Tauri 后端代码，保留 WebView 容器，前端改为调用 HTTP API
- **技术方案**: 
  - **推荐方案**: Tauri 作为纯前端容器（仅保留 WebView 功能）— 保持桌面应用体验，改动量最小
  - ~~选项 A: 纯 Web 应用（React SPA）~~ — 不选
  - ~~选项 B: Electron 桌面应用~~ — 不选
- **状态管理**: 从 Tauri Store 改为 IndexedDB + 服务端同步
- **Tauri 改造要点**:
  - 删除 `src-tauri/src/commands/` 下所有命令处理代码
  - 删除 `src-tauri/src/` 下的 `clip_server.rs` 等服务代码
  - 保留 `src-tauri/src/main.rs` 和 `src-tauri/src/lib.rs` 作为 WebView 入口
  - 保留 `tauri.conf.json` 配置（可能需要调整窗口大小、安全策略等）

#### 5. 文件存储改造（中优先级）
- **改造内容**: 文件操作从本地文件系统改为服务端管理
- **技术方案**: 
  - 选项 A: 服务器本地文件系统（简单部署）
  - 选项 B: 对象存储（S3/MinIO，适合大规模）
  - 选项 C: 混合模式（元数据在数据库，文件在对象存储）

#### 6. 向量数据库改造（中优先级）
- **改造内容**: LanceDB 从嵌入式改为服务端部署
- **技术方案**: 
  - 选项 A: LanceDB 服务端模式
  - 选项 B: 其他向量数据库（Milvus、Qdrant、Weaviate）

#### 7. 网页剪辑器改造（中优先级）
- **改造内容**: 从本地 HTTP 服务器改为调用云端 API
- **技术方案**: Chrome 扩展直接调用后端 API

#### 8. 实时通信改造（中优先级）
- **改造内容**: 导入进度、聊天流式响应等需要实时通信
- **技术方案**: WebSocket 或 Server-Sent Events (SSE)

#### 9. 数据库引入（中优先级）
- **改造内容**: 用户信息、权限、项目元数据需要持久化
- **技术方案**: PostgreSQL 或 SQLite（轻量部署）

#### 10. 多租户支持（低优先级）
- **改造内容**: 支持多个用户/组织隔离
- **技术方案**: 项目级别隔离、数据权限控制

## 技术栈选择建议

### 后端技术栈
| 组件 | 推荐方案 | 备选方案 |
|------|---------|---------|
| Web 框架 | Axum (Rust) | Actix-web (Rust), FastAPI (Python) |
| 认证 | JWT + bcrypt | Session-based, OAuth2 |
| 数据库 | PostgreSQL | SQLite, MySQL |
| ORM | SQLx (Rust) | Diesel (Rust), Prisma |
| 向量数据库 | Qdrant | LanceDB Server, Milvus |
| 文件存储 | 本地文件系统 + 对象存储 | S3, MinIO |
| 缓存 | Redis | Memcached |
| 部署 | Docker + Docker Compose | Kubernetes |

### 前端技术栈
| 组件 | 推荐方案 | 备选方案 |
|------|---------|---------|
| 框架 | React 19 (保留现有) | Vue 3, Svelte |
| 桌面容器 | Tauri (仅 WebView) | Electron, 纯 Web |
| 状态管理 | Zustand (保留现有) | Redux, Jotai |
| 本地存储 | IndexedDB + localForage | localStorage |
| HTTP 客户端 | Axios | fetch API, React Query |
| WebSocket | native WebSocket | Socket.IO |

## ADDED Requirements

### Requirement: 用户认证系统
系统 SHALL 提供用户注册、登录、令牌验证功能。

#### Scenario: 用户登录
- **WHEN** 用户输入正确的用户名和密码
- **THEN** 系统返回 JWT 访问令牌和刷新令牌

#### Scenario: 令牌验证
- **WHEN** 用户携带有效 JWT 访问受保护 API
- **THEN** 系统允许访问并识别用户身份

#### Scenario: 令牌刷新
- **WHEN** 访问令牌过期但刷新令牌有效
- **THEN** 系统颁发新的访问令牌

### Requirement: 基于角色的权限控制
系统 SHALL 提供基于角色的访问控制（RBAC），限制用户对功能的访问。

#### Scenario: 权限检查
- **WHEN** 用户尝试访问受保护的功能
- **THEN** 系统检查用户角色权限，允许或拒绝访问

#### Scenario: 功能权限控制
- **WHEN** 查看者角色用户尝试编辑 Wiki 页面
- **THEN** 系统拒绝操作并提示权限不足

### Requirement: RESTful API 服务
系统 SHALL 提供完整的 RESTful API，覆盖所有现有功能。

#### Scenario: 文件操作 API
- **WHEN** 前端调用文件读取 API
- **THEN** 后端验证权限后返回文件内容

#### Scenario: 流式聊天 API
- **WHEN** 前端调用聊天 API
- **THEN** 后端通过 WebSocket/SSE 流式返回 LLM 响应

### Requirement: 项目权限管理
系统 SHALL 支持项目级别的用户权限分配。

#### Scenario: 添加项目成员
- **WHEN** 管理员为用户分配项目角色
- **THEN** 用户获得对应项目的访问权限

## MODIFIED Requirements

### Requirement: 文件管理系统
**原实现**: Tauri 命令直接操作本地文件系统
**改造后**: 通过 HTTP API 调用后端服务，后端验证权限后操作文件系统

### Requirement: LLM 客户端
**原实现**: 前端直接调用 LLM API
**改造后**: 前端通过后端代理调用 LLM API，后端统一管理 API 密钥和配额

### Requirement: 向量搜索
**原实现**: 前端通过 Tauri 命令调用嵌入式 LanceDB
**改造后**: 前端通过 HTTP API 调用后端向量搜索服务

### Requirement: 网页剪辑器
**原实现**: Chrome 扩展调用本地 HTTP 服务器（127.0.0.1:19827）
**改造后**: Chrome 扩展直接调用云端后端 API，携带用户认证令牌

### Requirement: 配置管理
**原实现**: Tauri Store 持久化配置
**改造后**: 前端配置存储在 localStorage，用户/项目配置存储在服务端数据库

## REMOVED Requirements

### Requirement: Tauri 桌面应用架构
**Reason**: 拆分为独立后端和前端，不再需要 Tauri 的后端功能
**Migration**: 前端保留 Tauri 作为 WebView 容器，移除所有 Tauri 后端命令处理代码，后端改为独立 Web 服务

### Requirement: 本地 Clip Server
**Reason**: 网页剪辑器改为调用云端 API
**Migration**: Chrome 扩展更新为调用云端 API 端点

## 数据迁移策略

### 现有 Wiki 文件迁移
- **现状**: 现有 Wiki 文件存储在用户本地文件系统中，通过 `projectStore` 记录项目路径
- **迁移方案**: 在 Milestone 1.5 完成后，提供一次性数据迁移脚本/工具：
  1. 遍历本地项目目录下的所有文件
  2. 通过后端 `/api/files/:projectId/write` API 逐个上传到 MinIO
  3. 验证上传完成后，标记项目为「已迁移」
  4. 保留本地文件副本（用户可选择性删除）
- **LanceDB 向量数据迁移**: 现有向量数据需要重新通过 Qdrant API 导入，因为存储格式不同

### 配置迁移
- **Tauri Store 配置**: 导出为 JSON，通过前端导入到 IndexedDB + 服务端 API 同步
- **LLM API 密钥**: 从前端设置迁移到后端统一管理，前端不再存储密钥

## 前端 Tauri 改造细节

### 需要删除的文件
- `src-tauri/src/commands/` - 所有 Tauri 命令处理
- `src-tauri/src/clip_server.rs` - 本地 HTTP 服务器
- `src-tauri/src/types/` - 与后端重复的类型定义（保留前端需要的类型）

### 需要保留的文件
- `src-tauri/src/main.rs` - WebView 入口
- `src-tauri/src/lib.rs` - Tauri 应用初始化
- `src-tauri/Cargo.toml` - 仅保留 Tauri 相关依赖
- `src-tauri/tauri.conf.json` - WebView 配置

### 需要新增的前端代码
- `src/api/` - HTTP 客户端封装（Axios 实例、拦截器）
- `src/api/auth.ts` - 认证相关 API 调用
- `src/api/files.ts` - 文件操作 API 调用
- `src/api/projects.ts` - 项目管理 API 调用
- `src/api/chat.ts` - 聊天 WebSocket 封装
- `src/components/auth/` - 登录/注册页面组件
- `src/stores/auth-store.ts` - 认证状态管理
- `src/lib/api-client.ts` - 统一的 API 客户端
