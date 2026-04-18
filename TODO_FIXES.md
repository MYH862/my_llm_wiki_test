# 待修复问题清单

## 1. MinIO 对象存储集成

**问题描述**：
- 当前 `minio = "0.2"` crate API 与代码不兼容
- MinIO 服务已简化为 stub 实现（所有方法返回空结果）

**影响范围**：
- 文件上传/下载功能不可用
- 项目创建时无法自动创建 bucket
- 文件列表/删除/复制功能不可用

**修复方案**：
1. 使用正确的 MinIO Rust SDK：
   - 选项 A：`minio-rsc` crate（推荐，API 更友好）
   - 选项 B：官方 `minio` crate（需要查找正确版本）
2. 重新实现 `MinIOService` 的所有方法
3. 测试文件上传/下载功能

**相关文件**：
- `server/src/services/file.rs`
- `server/src/api/files.rs`
- `server/Cargo.toml`

**优先级**：高（核心功能）

---

## 2. 文档预处理功能（PDF/DOCX/PPTX/XLSX）

**问题描述**：
- `pdf-extract` crate 没有 `OutputConfig` 和 `extract_text_to_string` API
- 已移除 `pdf-extract`, `docx-rs`, `calamine`, `zip` 等依赖
- 当前文档预处理仅返回原始文本内容

**影响范围**：
- PDF 文件无法提取文本
- DOCX/PPTX/XLSX 文件无法解析

**修复方案**：
1. 查找正确的文档解析库：
   - PDF：`lopdf` 或 `pdf-parser`
   - DOCX：`docx-rs`（需要查找正确版本）
   - XLSX：`calamine`（需要查找正确版本）
   - PPTX：需要查找 Rust PPTX 解析库
2. 重新实现 `preprocess_document` API
3. 添加分页支持（`page_count` 字段）

**相关文件**：
- `server/src/api/files.rs` (preprocess_document 函数)
- `server/Cargo.toml`

**优先级**：中（导入功能依赖）

---

## 3. PostgreSQL 数据库未启动

**问题描述**：
- Docker Desktop 未运行
- 无法启动 PostgreSQL 容器
- SQLx 编译时检查需要数据库连接

**影响范围**：
- 无法运行数据库迁移
- 无法测试 API 功能
- SQLx 查询宏需要 `DATABASE_URL` 环境变量

**修复方案**：
1. 启动 Docker Desktop
2. 运行 `docker-compose up -d postgres`
3. 设置 `DATABASE_URL` 环境变量
4. 运行数据库迁移

**相关文件**：
- `server/docker-compose.yml`
- `server/.env.example`

**优先级**：高（开发环境必需）

---

## 4. MinIO 服务未部署

**问题描述**：
- docker-compose.yml 中没有 MinIO 服务配置
- 无法测试文件存储功能

**影响范围**：
- 无法测试 MinIO 集成
- 文件存储功能无法验证

**修复方案**：
1. 在 `docker-compose.yml` 中添加 MinIO 服务
2. 配置 MinIO 环境变量
3. 创建 MinIO 数据卷

**示例配置**：
```yaml
minio:
  image: minio/minio:latest
  command: server /data --console-address ":9001"
  ports:
    - "9000:9000"
    - "9001:9001"
  environment:
    MINIO_ROOT_USER: minioadmin
    MINIO_ROOT_PASSWORD: minioadmin
  volumes:
    - minio_data:/data

volumes:
  minio_data:
```

**相关文件**：
- `server/docker-compose.yml`

**优先级**：高（文件存储依赖）

---

## 5. SQLx 离线缓存未生成

**问题描述**：
- 没有运行 `cargo sqlx prepare` 生成查询缓存
- 离线编译时需要 `SQLX_OFFLINE=true` 和缓存文件

**影响范围**：
- CI/CD 环境无法编译
- 没有数据库时无法编译

**修复方案**：
1. 启动 PostgreSQL 数据库
2. 运行 `cargo sqlx prepare`
3. 提交生成的 `sqlx-data.json` 到 Git

**相关文件**：
- `server/sqlx-data.json`（需要生成）

**优先级**：中（CI/CD 依赖）

---

## 6. 密码重置功能未实现

**问题描述**：
- 只有密码修改功能（需要当前密码）
- 没有"忘记密码"重置流程

**影响范围**：
- 用户忘记密码无法找回

**修复方案**：
1. 实现邮件发送功能（需要 SMTP 配置）
2. 生成密码重置令牌
3. 实现重置密码 API

**相关文件**：
- `server/src/services/auth.rs`
- `server/src/api/auth.rs`

**优先级**：低（可用密码修改替代）

---

## 7. 向量数据库（Qdrant）未集成

**问题描述**：
- Task 8 需要集成 Qdrant 向量数据库
- 当前没有向量搜索功能

**影响范围**：
- 语义搜索功能不可用
- 知识图谱构建依赖向量搜索

**修复方案**：
1. 在 docker-compose.yml 中添加 Qdrant 服务
2. 添加 `qdrant-client` crate 依赖
3. 实现向量嵌入/搜索 API

**相关文件**：
- `server/Cargo.toml`
- `server/docker-compose.yml`
- `server/src/services/vector.rs`（需要创建）

**优先级**：高（核心功能）

---

## 8. LLM 流式聊天（WebSocket）未实现

**问题描述**：
- Task 7 需要实现 WebSocket 流式聊天
- 当前只有 HTTP API 骨架

**影响范围**：
- 聊天功能不可用
- 深度研究功能依赖 LLM 调用

**修复方案**：
1. 实现 WebSocket 连接处理
2. 集成 LLM API（OpenAI/Anthropic 等）
3. 实现流式响应推送

**相关文件**：
- `server/src/api/chat.rs`
- `server/src/services/llm.rs`（需要创建）

**优先级**：高（核心功能）

---

## 9. 前端改造未开始

**问题描述**：
- Task 11-15 前端改造未开始
- 当前前端仍然使用 Tauri 命令调用

**影响范围**：
- 前端无法调用后端 API
- 用户认证 UI 缺失

**修复方案**：
1. 移除 Tauri 后端依赖
2. 实现 HTTP API 客户端
3. 添加认证 UI（登录/注册）
4. 替换所有 Tauri 命令调用

**相关文件**：
- `src/` 目录下所有文件
- `src-tauri/` 目录（需要移除后端代码）

**优先级**：高（用户界面）

---

## 10. 数据迁移工具未实现

**问题描述**：
- Task 16 需要实现数据迁移工具
- 从 Tauri 本地存储迁移到服务端

**影响范围**：
- 现有用户数据无法迁移

**修复方案**：
1. 实现本地文件上传脚本
2. 实现 Tauri Store 配置导出工具
3. 实现向量数据迁移工具

**相关文件**：
- `server/migrations/`（需要创建数据迁移脚本）

**优先级**：中（上线前必需）

---

## 修复优先级排序

### 高优先级（核心功能）
1. ✅ PostgreSQL 数据库启动
2. ✅ MinIO 服务部署
3. ✅ MinIO 对象存储集成
4. ✅ 向量数据库（Qdrant）集成
5. ✅ LLM 流式聊天（WebSocket）

### 中优先级（重要功能）
6. 文档预处理功能
7. SQLx 离线缓存
8. 数据迁移工具

### 低优先级（可延后）
9. 密码重置功能

---

## 修复时间线建议

**Phase 1**：开发环境搭建
- 启动 Docker Desktop
- 启动 PostgreSQL 和 MinIO
- 生成 SQLx 缓存

**Phase 2**：核心功能完善
- 集成 MinIO SDK
- 集成 Qdrant 向量数据库
- 实现 LLM 流式聊天

**Phase 3**：辅助功能
- 文档预处理
- 密码重置
- 数据迁移工具

---

**最后更新**：2026-04-18
**状态**：待修复
