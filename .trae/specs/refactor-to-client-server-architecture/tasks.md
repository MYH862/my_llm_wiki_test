# Tasks

## Phase 1: 后端服务基础架构

- [x] Task 1: 创建独立 Rust Web 服务项目结构
  - [x] SubTask 1.1: 初始化新的 Rust 项目（使用 Axum 框架）
  - [x] SubTask 1.2: 配置项目依赖（Axum, Tokio, Serde, SQLx 等）
  - [x] SubTask 1.3: 设置项目目录结构（src/api, src/models, src/services, src/db 等）
  - [x] SubTask 1.4: 配置 Dockerfile 和 docker-compose.yml

- [x] Task 2: 实现数据库层
  - [x] SubTask 2.1: 设计数据库 Schema（用户、角色、权限、项目表）
  - [x] SubTask 2.2: 实现数据库连接池和迁移系统
  - [x] SubTask 2.3: 创建基础 CRUD 操作

- [x] Task 3: 实现用户认证系统
  - [x] SubTask 3.1: 实现用户注册 API（密码 bcrypt 哈希）
  - [x] SubTask 3.2: 实现用户登录 API（JWT 令牌生成）
  - [x] SubTask 3.3: 实现 JWT 中间件（令牌验证、刷新）
  - [ ] SubTask 3.4: 实现密码重置功能（仅标记，暂未实现）

- [ ] Task 4: 实现权限控制系统（RBAC）
  - [x] SubTask 4.1: 实现角色和权限数据模型
  - [x] SubTask 4.2: 实现权限检查中间件（permission.rs 基础函数）
  - [ ] SubTask 4.3: 实现项目级别权限管理 API（需实现 API 路由和处理函数）
  - [ ] SubTask 4.4: 实现用户管理 API（需实现 users.rs API 路由和处理函数）

## Phase 2: 核心业务 API 迁移

- [ ] Task 5: 迁移文件操作 API
  - [ ] SubTask 5.1: 实现 MinIO 文件服务（已有基础代码，需验证和补全）
  - [ ] SubTask 5.2: 实现文件读取 API（已有实现，需验证）
  - [ ] SubTask 5.3: 实现文件写入 API（已有实现，需验证）
  - [ ] SubTask 5.4: 实现目录列表 API（已有实现，需验证）
  - [ ] SubTask 5.5: 实现文件删除 API（已有实现，需验证）
  - [ ] SubTask 5.6: 实现文件复制 API（已有实现，需验证）
  - [ ] SubTask 5.7: 实现文档预处理 API（PDF, DOCX, PPTX, XLSX）

- [ ] Task 6: 迁移项目管理 API
  - [ ] SubTask 6.1: 实现创建项目 API
  - [ ] SubTask 6.2: 实现打开项目 API
  - [ ] SubTask 6.3: 实现项目列表 API
  - [ ] SubTask 6.4: 实现项目配置 API

- [ ] Task 7: 迁移 LLM 集成服务
  - [ ] SubTask 7.1: 实现 LLM 流式聊天 API（WebSocket/SSE）
  - [ ] SubTask 7.2: 实现两步思维链导入服务
  - [ ] SubTask 7.3: 实现持久化导入队列服务
  - [ ] SubTask 7.4: 实现 LLM 配置管理 API

- [ ] Task 8: 迁移向量搜索服务
  - [ ] SubTask 8.1: 集成 Qdrant 服务（Docker 部署 + Rust 客户端）
  - [ ] SubTask 8.2: 实现向量嵌入 API
  - [ ] SubTask 8.3: 实现向量搜索 API
  - [ ] SubTask 8.4: 实现向量 Upsert/Delete API

- [ ] Task 9: 迁移知识图谱服务
  - [ ] SubTask 9.1: 实现图谱构建 API
  - [ ] SubTask 9.2: 实现 Louvain 社区检测 API
  - [ ] SubTask 9.3: 实现图谱洞察 API

- [ ] Task 10: 迁移深度研究和审核系统
  - [ ] SubTask 10.1: 实现 Web 搜索 API（Tavily 集成）
  - [ ] SubTask 10.2: 实现深度研究任务队列
  - [ ] SubTask 10.3: 实现审核项 CRUD API
  - [ ] SubTask 10.4: 实现 Lint 检查 API

## Phase 3: 前端改造

- [ ] Task 11: 改造 Tauri 容器（保留 WebView，移除后端代码）
  - [ ] SubTask 11.1: 移除 `src-tauri/src/commands/` 目录（Tauri 命令处理代码）
  - [ ] SubTask 11.2: 移除 `src-tauri/src/clip_server.rs` 等后端服务代码
  - [ ] SubTask 11.3: 清理 `src-tauri/Cargo.toml` 移除不必要的后端依赖
  - [ ] SubTask 11.4: 验证 Tauri 构建仍然可用（`cargo tauri dev`）

- [ ] Task 12: 实现 API 客户端层
  - [ ] SubTask 12.1: 创建 HTTP 客户端封装（Axios 实例）
  - [ ] SubTask 12.2: 实现认证拦截器（JWT 自动附加/刷新）
  - [ ] SubTask 12.3: 实现错误处理和重试机制
  - [ ] SubTask 12.4: 创建 WebSocket 客户端（用于流式响应）

- [ ] Task 13: 实现认证 UI
  - [ ] SubTask 13.1: 创建登录页面组件
  - [ ] SubTask 13.2: 创建注册页面组件
  - [ ] SubTask 13.3: 实现认证状态管理（Zustand store）
  - [ ] SubTask 13.4: 实现路由守卫（未登录重定向）

- [ ] Task 14: 改造现有组件调用方式
  - [ ] SubTask 14.1: 替换所有 Tauri 命令调用为 HTTP API 调用
  - [ ] SubTask 14.2: 替换 Tauri Store 为 localStorage/IndexedDB
  - [ ] SubTask 14.3: 更新文件操作组件
  - [ ] SubTask 14.4: 更新聊天组件（使用 WebSocket）
  - [ ] SubTask 14.5: 更新导入队列组件（使用 WebSocket 进度）
  - [ ] SubTask 14.6: 更新设置页面（服务端配置同步）

- [ ] Task 15: 实现权限 UI
  - [ ] SubTask 15.1: 创建权限检查 Hook
  - [ ] SubTask 15.2: 实现功能级权限控制（隐藏/禁用无权限功能）
  - [ ] SubTask 15.3: 创建项目管理页面（成员管理、角色分配）
  - [ ] SubTask 15.4: 创建用户管理页面（仅管理员可见）

## Phase 4: 数据迁移

- [ ] Task 16: 数据迁移工具
  - [ ] SubTask 16.1: 实现本地文件批量上传到 MinIO 脚本
  - [ ] SubTask 16.2: 实现 LanceDB 向量数据导出脚本
  - [ ] SubTask 16.3: 实现 Qdrant 向量导入脚本
  - [ ] SubTask 16.4: 实现 Tauri Store 配置导出/导入工具
  - [ ] SubTask 16.5: 迁移验证（文件完整性、向量可用性）

## Phase 5: 网页剪辑器和部署

- [ ] Task 17: 改造 Chrome 网页剪辑器
  - [ ] SubTask 17.1: 更新扩展 manifest 配置
  - [ ] SubTask 17.2: 修改为调用云端 API
  - [ ] SubTask 17.3: 实现用户认证令牌传递
  - [ ] SubTask 17.4: 实现项目选择器（从服务端获取）

- [ ] Task 18: 配置部署和 CI/CD
  - [ ] SubTask 18.1: 完善 Docker Compose 配置
  - [ ] SubTask 18.2: 配置环境变量管理
  - [ ] SubTask 18.3: 更新 GitHub Actions CI/CD
  - [ ] SubTask 18.4: 编写部署文档

## Phase 6: 测试和优化

- [ ] Task 19: 编写测试
  - [ ] SubTask 19.1: 后端 API 单元测试
  - [ ] SubTask 19.2: 后端集成测试
  - [ ] SubTask 19.3: 前端组件测试
  - [ ] SubTask 19.4: 端到端测试

- [ ] Task 20: 性能优化和安全加固
  - [ ] SubTask 20.1: 实现 API 速率限制
  - [ ] SubTask 20.2: 实现请求日志和监控
  - [ ] SubTask 20.3: 优化数据库查询
  - [ ] SubTask 20.4: 实现 CORS 配置
  - [ ] SubTask 20.5: 安全审计（SQL 注入、XSS 等）

# Task Dependencies

- [Task 2] depends on [Task 1]
- [Task 3] depends on [Task 2]
- [Task 4] depends on [Task 3]
- [Task 5] depends on [Task 4]
- [Task 6] depends on [Task 4]
- [Task 7] depends on [Task 4]
- [Task 8] depends on [Task 4]
- [Task 9] depends on [Task 7]
- [Task 10] depends on [Task 7]
- [Task 11] depends on [Task 5]  # 前端改造只需依赖文件 API，可与其他后端任务并行
- [Task 12] depends on [Task 3, Task 11]  # API 客户端层依赖认证 API 和 Tauri 清理
- [Task 13] depends on [Task 12]
- [Task 14] depends on [Task 12, Task 13]
- [Task 15] depends on [Task 14]
- [Task 16] depends on [Task 5, Task 8]  # 数据迁移依赖文件和向量服务
- [Task 17] depends on [Task 7]
- [Task 18] depends on [Task 15]
- [Task 19] depends on [Task 15, Task 17, Task 16]
- [Task 20] depends on [Task 19]

# 并行执行建议

以下任务可以并行执行：
- Task 5, 6, 7, 8, 9, 10 可以并行（在 Task 4 完成后）
- Task 11, 12, 13 可以并行（后端任务进行时可开始前端改造）
- Task 19 的各个子任务可以并行
- Task 16 可与 Task 17, 18 并行
