# LLM Wiki 企业级应用重构计划

## 一、项目现状分析

### 1.1 当前架构

* **技术栈**: Tauri v2 (Rust 后端) + React 19 + TypeScript + Vite

* **部署方式**: 桌面级应用（macOS/Windows/Linux）

* **数据存储**: 本地文件系统 + Tauri Store（本地配置持久化）

* **通信方式**: Tauri IPC（前端调用 Rust 命令）

* **用户模型**: 无用户概念，单用户本地使用

* **权限模型**: 无权限控制

### 1.2 核心功能模块

| 模块          | 前端组件                               | 后端命令                                                                    |
| ----------- | ---------------------------------- | ----------------------------------------------------------------------- |
| 项目管理        | WelcomeScreen, CreateProjectDialog | `create_project`, `open_project`                                        |
| 文件操作        | FileTree, WikiEditor               | `read_file`, `write_file`, `list_directory`, `copy_file`, `delete_file` |
| 知识图谱        | GraphView                          | 无（前端处理）                                                                 |
| 聊天系统        | ChatPanel, ChatMessage             | 无（前端调用 LLM API）                                                         |
| 向量搜索        | SearchView                         | `vector_upsert`, `vector_search`, `vector_delete`, `vector_count`       |
| Web Clipper | 无                                  | Clip Server (端口 19827)                                                  |
| 深度研究        | ResearchPanel                      | 无（前端处理）                                                                 |
| 设置管理        | SettingsView                       | Tauri Store                                                             |

***

## 二、企业级应用需求梳理

### 2.1 核心需求

1. **用户认证系统** - 支持用户注册、登录、登出
2. **权限控制系统** - 基于角色的访问控制（RBAC）
3. **后端服务独立部署** - 将 Rust 后端改造为可部署在服务器上的独立服务
4. **前端桌面应用** - 前端打包为桌面应用，通过网络与后端通信
5. **多租户支持** - 支持企业内部多用户、多项目协作
6. **数据安全** - API 密钥、用户数据加密存储

### 2.2 衍生需求

* 用户会话管理（JWT Token）

* 项目权限管理（项目所有者、管理员、编辑者、查看者）

* 文件存储方案（本地文件系统 → 对象存储/分布式文件系统）

* 数据库支持（用户信息、权限、项目元数据）

* API 网关/路由层

* 日志与审计

* 配置中心

***

## 三、需要重构的功能点清单

### 3.1 架构层重构（高优先级）

#### 1. 前后端分离

* **现状**: Tauri 架构，前端通过 IPC 调用 Rust 命令

* **目标**: 前端通过 HTTP/WebSocket 与后端 API 通信

* **改动点**:

  * 将 `src-tauri/src/commands/` 改造为 HTTP API 端点

  * 移除 Tauri 特定的 IPC 调用方式

  * 前端统一使用 HTTP Client（如 axios/fetch）调用 API

  * 保留 Tauri 仅作为前端打包工具（或改用 Electron/其他方案）

#### 2. 数据库引入

* **现状**: 无数据库，所有数据存储在本地文件系统

* **目标**: 引入关系型数据库（PostgreSQL/MySQL）

* **改动点**:

  * 设计用户表（users）

  * 设计角色表（roles）

  * 设计用户角色关联表（user\_roles）

  * 设计项目表（projects）

  * 设计项目成员表（project\_members）

  * 设计会话表（sessions）

  * 设计审计日志表（audit\_logs）

  * 迁移 Tauri Store 配置到数据库

#### 3. 文件存储方案

* **现状**: 本地文件系统存储 wiki 项目文件

* **目标**: 支持对象存储（MinIO/S3）或分布式文件系统

* **改动点**:

  * 抽象文件存储接口（FileSystemStorage / S3Storage）

  * 改造所有文件操作命令（read\_file, write\_file, list\_directory 等）

  * 支持文件版本控制

  * 支持文件访问权限控制

### 3.2 用户认证系统（高优先级）

#### 4. 用户注册与登录

* **现状**: 无用户概念

* **目标**: 完整的用户认证流程

* **改动点**:

  * 新增用户注册 API（POST /api/auth/register）

  * 新增用户登录 API（POST /api/auth/login）

  * 新增用户登出 API（POST /api/auth/logout）

  * 新增密码重置 API（POST /api/auth/reset-password）

  * 前端新增登录页面、注册页面

  * 前端新增用户状态管理（UserStore）

  * 实现 JWT Token 生成与验证

  * 实现 Token 刷新机制

#### 5. 会话管理

* **现状**: 无会话概念

* **目标**: 基于 JWT 的会话管理

* **改动点**:

  * 实现 JWT Token 签发（access\_token + refresh\_token）

  * 实现 Token 验证中间件

  * 实现 Token 自动刷新

  * 前端实现 Token 存储与自动携带

  * 实现会话超时处理

  * 实现多设备登录管理

### 3.3 权限控制系统（高优先级）

#### 6. 基于角色的访问控制（RBAC）

* **现状**: 无权限控制

* **目标**: 完整的 RBAC 权限模型

* **改动点**:

  * 定义角色类型：

    * 超级管理员（Super Admin）- 系统级管理

    * 组织管理员（Org Admin）- 组织级管理

    * 项目管理员（Project Admin）- 项目管理

    * 编辑者（Editor）- 项目内容编辑

    * 查看者（Viewer）- 只读访问

  * 定义权限类型：

    * 项目管理（create, read, update, delete）

    * 文件操作（read, write, delete）

    * 用户管理（invite, remove, change\_role）

    * 系统设置（manage\_settings）

  * 实现权限验证中间件

  * 前端实现权限路由守卫

  * 前端实现按钮级权限控制

#### 7. 项目权限管理

* **现状**: 项目本地存储，无共享概念

* **目标**: 项目级权限管理

* **改动点**:

  * 实现项目成员邀请机制

  * 实现项目成员角色分配

  * 实现项目权限验证

  * 前端实现项目成员管理界面

  * 实现项目访问控制列表（ACL）

### 3.4 后端服务改造（高优先级）

#### 8. HTTP API 服务

* **现状**: Tauri 命令（IPC 调用）

* **目标**: 独立 HTTP API 服务

* **改动点**:

  * 选择 Web 框架（Actix-web / Axum / Rocket）

  * 改造所有 Tauri 命令为 HTTP 端点

  * 实现 API 路由组织

  * 实现请求/响应模型

  * 实现错误处理中间件

  * 实现 CORS 配置

  * 实现 API 版本管理

  * 实现 API 文档（OpenAPI/Swagger）

#### 9. 现有命令改造清单

| 原 Tauri 命令                | 新 HTTP 端点                      | 权限要求   |
| ------------------------- | ------------------------------ | ------ |
| `read_file`               | GET /api/files/:path           | 项目读取权限 |
| `write_file`              | PUT /api/files/:path           | 项目写入权限 |
| `list_directory`          | GET /api/files/:path/list      | 项目读取权限 |
| `copy_file`               | POST /api/files/copy           | 项目写入权限 |
| `copy_directory`          | POST /api/files/copy-directory | 项目写入权限 |
| `delete_file`             | DELETE /api/files/:path        | 项目删除权限 |
| `preprocess_file`         | POST /api/files/preprocess     | 项目写入权限 |
| `find_related_wiki_pages` | GET /api/files/related         | 项目读取权限 |
| `create_directory`        | POST /api/files/directory      | 项目写入权限 |
| `create_project`          | POST /api/projects             | 认证用户   |
| `open_project`            | GET /api/projects/:id          | 项目成员   |
| `vector_upsert`           | POST /api/vector/upsert        | 项目写入权限 |
| `vector_search`           | POST /api/vector/search        | 项目读取权限 |
| `vector_delete`           | DELETE /api/vector/:id         | 项目删除权限 |
| `vector_count`            | GET /api/vector/count          | 项目读取权限 |
| `clip_server_status`      | GET /api/clip/status           | 认证用户   |

#### 10. Clip Server 改造

* **现状**: 本地 HTTP 服务（端口 19827），用于 Chrome 扩展通信

* **目标**: 集成到主 API 服务或独立微服务

* **改动点**:

  * 将 Clip Server 路由集成到主 API

  * 改造为 `/api/clip/*` 端点

  * 实现 Clip 权限验证

  * 更新 Chrome 扩展配置

### 3.5 前端改造（高优先级）

#### 11. 前端架构调整

* **现状**: 依赖 Tauri API 进行系统调用

* **目标**: 独立前端应用，通过 HTTP 与后端通信

* **改动点**:

  * 移除 `@tauri-apps/api` 依赖（或仅保留打包相关）

  * 引入 HTTP Client（axios）

  * 实现 API 请求拦截器（自动携带 Token）

  * 实现 API 响应拦截器（统一错误处理）

  * 实现请求重试机制

  * 实现离线状态处理

#### 12. 新增前端页面

* **改动点**:

  * 登录页面（LoginView）

  * 注册页面（RegisterView）

  * 忘记密码页面（ForgotPasswordView）

  * 用户个人资料页面（ProfileView）

  * 项目成员管理页面（ProjectMembersView）

  * 系统管理页面（AdminView）- 仅超级管理员可见

  * 权限管理页面（PermissionsView）

#### 13. 前端状态管理改造

* **现状**: Zustand 存储本地状态

* **目标**: 增加用户、权限、会话状态

* **改动点**:

  * 新增 `user-store.ts` - 用户信息与认证状态

  * 新增 `auth-store.ts` - Token 管理

  * 新增 `permission-store.ts` - 权限状态

  * 改造 `wiki-store.ts` - 增加项目权限检查

  * 改造 `project-store.ts` - 支持从后端加载项目列表

#### 14. 前端路由改造

* **现状**: 无路由概念，单页面应用

* **目标**: 增加路由与权限守卫

* **改动点**:

  * 引入 React Router

  * 实现路由配置

  * 实现认证守卫（未登录跳转登录页）

  * 实现权限守卫（无权限跳转 403 页面）

  * 实现路由懒加载

### 3.6 安全与加密（中优先级）

#### 15. 数据安全

* **现状**: API 密钥明文存储在本地

* **目标**: 加密存储敏感数据

* **改动点**:

  * 实现密码加密存储（bcrypt/argon2）

  * 实现 API 密钥加密存储

  * 实现传输层加密（HTTPS）

  * 实现敏感数据脱敏

  * 实现数据备份与恢复

#### 16. API 安全

* **改动点**:

  * 实现请求频率限制（Rate Limiting）

  * 实现 CSRF 防护

  * 实现 XSS 防护

  * 实现 SQL 注入防护

  * 实现文件上传安全验证

  * 实现 API 密钥轮换机制

### 3.7 日志与审计（中优先级）

#### 17. 操作日志

* **现状**: 无日志系统

* **目标**: 完整的操作审计日志

* **改动点**:

  * 实现操作日志记录（谁、何时、做了什么）

  * 实现错误日志记录

  * 实现性能日志记录

  * 前端实现日志查看界面

  * 实现日志导出功能

### 3.8 配置管理（中优先级）

#### 18. 配置中心

* **现状**: Tauri Store 本地配置

* **目标**: 集中式配置管理

* **改动点**:

  * 实现系统级配置（数据库存储）

  * 实现用户级配置（数据库存储）

  * 实现项目级配置（数据库存储）

  * 实现配置缓存机制

  * 实现配置热更新

### 3.9 部署与运维（中优先级）

#### 19. 后端部署

* **改动点**:

  * 编写 Dockerfile

  * 编写 docker-compose.yml

  * 编写 Kubernetes 部署配置（可选）

  * 实现健康检查端点

  * 实现优雅关闭

  * 编写 CI/CD 流水线

#### 20. 前端打包

* **改动点**:

  * 保留 Tauri 打包为桌面应用

  * 或改用 Electron 打包

  * 或仅打包为 Web 应用

  * 实现前端配置注入（API 地址等）

  * 实现多环境配置（开发/测试/生产）

### 3.10 其他优化（低优先级）

#### 21. 性能优化

* **改动点**:

  * 实现 API 响应缓存

  * 实现数据库查询优化

  * 实现文件操作异步化

  * 实现 WebSocket 支持（实时通知）

  * 实现分页与懒加载

#### 22. 国际化增强

* **现状**: 支持中英文

* **改动点**:

  * 支持多语言切换

  * 支持用户语言偏好存储

  * 新增更多语言支持

***

## 四、重构阶段规划

### 阶段一：基础架构改造（2-3 周）

1. 选择并集成 Web 框架（Axum 推荐）
2. 设计并创建数据库 Schema
3. 实现基础 HTTP API 服务框架
4. 实现 CORS、错误处理等中间件
5. 改造 1-2 个简单命令验证架构可行性

### 阶段二：用户认证系统（2-3 周）

1. 实现用户注册/登录 API
2. 实现 JWT Token 机制
3. 前端实现登录/注册页面
4. 前端实现 Token 管理
5. 实现认证中间件

### 阶段三：权限控制系统（2-3 周）

1. 实现 RBAC 模型
2. 实现权限验证中间件
3. 前端实现权限路由守卫
4. 前端实现按钮级权限控制
5. 实现项目权限管理

### 阶段四：核心功能迁移（3-4 周）

1. 迁移所有文件操作命令
2. 迁移向量搜索命令
3. 迁移项目管理命令
4. 改造 Clip Server
5. 前端全面替换 API 调用方式

### 阶段五：前端完善（2-3 周）

1. 引入 React Router
2. 新增所有管理页面
3. 实现状态管理改造
4. 实现离线处理
5. 实现错误边界优化

### 阶段六：安全与优化（2-3 周）

1. 实现数据安全加密
2. 实现 API 安全防护
3. 实现日志与审计
4. 实现配置中心
5. 性能优化

### 阶段七：部署与测试（1-2 周）

1. 编写 Docker 配置
2. 编写 CI/CD 流水线
3. 编写部署文档
4. 全面测试
5. 灰度发布

***

## 五、技术选型建议

| 组件             | 推荐方案                    | 备选方案                   |
| -------------- | ----------------------- | ---------------------- |
| Web 框架         | Axum                    | Actix-web, Rocket      |
| 数据库            | PostgreSQL              | MySQL, SQLite          |
| ORM            | SQLx                    | Diesel, SeaORM         |
| 对象存储           | MinIO                   | AWS S3, 阿里云 OSS        |
| 缓存             | Redis                   | Memcached              |
| 消息队列           | Redis Streams           | RabbitMQ, Kafka        |
| 前端 HTTP Client | Axios                   | Fetch API, RTK Query   |
| 前端路由           | React Router            | TanStack Router        |
| 桌面打包           | Tauri v2                | Electron, Neutralinojs |
| 容器化            | Docker + docker-compose | Kubernetes             |
| CI/CD          | GitHub Actions          | GitLab CI, Jenkins     |

***

## 六、风险与注意事项

1. **数据迁移风险**: 本地文件系统 → 数据库/对象存储，需要编写迁移脚本
2. **兼容性风险**: 现有 Chrome 扩展需要更新 API 地址
3. **性能风险**: 网络延迟可能影响用户体验，需要优化 API 响应时间
4. **安全风险**: 暴露 API 到网络，需要加强安全防护
5. **用户体验风险**: 从本地应用 → 网络应用，需要处理离线场景
6. **部署复杂度**: 从单文件应用 → 多服务部署，需要运维支持

***

## 七、后续扩展方向

1. **微服务架构**: 将 LLM 调用、向量搜索、文件存储拆分为独立服务
2. **多租户 SaaS**: 支持企业级多租户隔离
3. **SSO 集成**: 支持企业微信、钉钉、LDAP 等单点登录
4. **AI 能力增强**: 支持多模型路由、模型负载均衡
5. **协作功能**: 实时协同编辑、评论、审批流
6. **数据分析**: 用户行为分析、知识库使用统计
7. **移动端**: 开发移动端应用（React Native / Flutter）

