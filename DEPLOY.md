# LLM Wiki 部署指南

## 概述

本文档介绍如何部署 LLM Wiki 客户端-服务器架构的完整系统。

## 系统架构

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Frontend  │────▶│   Server    │────▶│  PostgreSQL │
│  (Tauri/    │     │   (Axum)    │     │  (Database) │
│   React)    │     └──────┬──────┘     └─────────────┘
└─────────────┘            │
                           ├─────────────┐
                           ▼             ▼
                    ┌─────────────┐ ┌─────────────┐
                    │    MinIO    │ │   Qdrant    │
                    │   (Files)   │ │  (Vectors)  │
                    └─────────────┘ └─────────────┘
```

## 部署方式

### 方式一：Docker Compose 部署（推荐）

#### 1. 环境要求

- Docker 20.10+
- Docker Compose 2.0+
- 至少 4GB RAM
- 至少 10GB 磁盘空间

#### 2. 配置环境变量

```bash
cd server
cp .env.example .env
```

编辑 `.env` 文件，更新以下关键配置：

```bash
# 必须修改
JWT_SECRET=your-super-secret-jwt-key-change-in-production
POSTGRES_PASSWORD=your-secure-password-here
MINIO_ROOT_PASSWORD=your-minio-password-here

# 生产环境配置
ALLOWED_ORIGINS=https://your-domain.com
VITE_API_BASE_URL=https://api.your-domain.com/api
```

#### 3. 启动服务

```bash
# 开发环境
docker-compose up -d

# 生产环境
docker-compose -f docker-compose.prod.yml up -d
```

#### 4. 验证部署

```bash
# 检查服务状态
docker-compose ps

# 查看日志
docker-compose logs -f server

# 测试健康检查
curl http://localhost:3000/health
```

#### 5. 创建超级管理员账号

首次启动后，系统会自动创建超级管理员账号：

- 用户名: `admin`
- 密码: `admin123`

**重要**: 首次登录后必须修改密码！

### 方式二：手动部署

#### 1. 安装依赖服务

```bash
# PostgreSQL
docker run -d \
  --name llm-wiki-postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=your-password \
  -e POSTGRES_DB=llm_wiki \
  -p 5432:5432 \
  -v postgres_data:/var/lib/postgresql/data \
  postgres:16-alpine

# MinIO
docker run -d \
  --name llm-wiki-minio \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=your-password \
  -p 9000:9000 \
  -p 9001:9001 \
  -v minio_data:/data \
  minio/minio server /data --console-address ":9001"

# Qdrant
docker run -d \
  --name llm-wiki-qdrant \
  -p 6333:6333 \
  -p 6334:6334 \
  -v qdrant_data:/qdrant/storage \
  qdrant/qdrant
```

#### 2. 编译和运行后端

```bash
cd server

# 配置环境变量
cp .env.example .env
# 编辑 .env 文件

# 编译
cargo build --release

# 运行
./target/release/llm-wiki-server
```

#### 3. 编译和运行前端

```bash
# 安装依赖
npm install

# 配置环境变量
echo "VITE_API_BASE_URL=http://localhost:3000/api" > .env

# 开发模式
npm run dev

# 生产构建
npm run build
```

### 方式三：Kubernetes 部署

#### 1. 创建命名空间

```bash
kubectl create namespace llm-wiki
```

#### 2. 创建 ConfigMap 和 Secrets

```bash
kubectl create secret generic llm-wiki-secrets \
  --from-literal=jwt-secret=your-jwt-secret \
  --from-literal=postgres-password=your-password \
  --from-literal=minio-password=your-password \
  -n llm-wiki
```

#### 3. 部署服务

```bash
kubectl apply -f k8s/ -n llm-wiki
```

## Chrome 扩展部署

### 1. 打包扩展

```bash
cd extension
zip -r ../llm-wiki-clipper.zip .
```

### 2. 安装到 Chrome

1. 打开 Chrome 浏览器
2. 访问 `chrome://extensions/`
3. 启用"开发者模式"
4. 点击"加载已解压的扩展程序"
5. 选择 `extension` 目录

### 3. 配置扩展

1. 点击扩展图标
2. 点击"Settings"按钮
3. 输入 API URL: `http://localhost:3000/api`
4. 输入 JWT Access Token（从登录接口获取）
5. 点击"Save"保存

## CI/CD 部署

### GitHub Actions

项目已配置 GitHub Actions 工作流：

#### CI 工作流 (ci.yml)

- 前端构建检查
- 后端编译检查
- Tauri 多平台构建检查

触发条件:
- Push 到 main 分支
- Pull Request 到 main 分支

#### CD 工作流 (build.yml)

- 构建 Docker 镜像并推送到 GHCR
- 打包 Chrome 扩展
- 构建 Tauri 应用（多平台）
- 创建 GitHub Release

触发条件:
- 推送版本标签 (如 `v1.0.0`)
- 手动触发

### 发布新版本

```bash
# 更新版本号
git tag v1.0.0

# 推送标签
git push origin v1.0.0

# GitHub Actions 会自动构建和发布
```

## 数据迁移

### 从 Tauri 本地存储迁移

```bash
# 安装依赖
pip install requests minio qdrant-client lancedb

# 执行迁移
python tools/migrate/migrate.py <项目路径> <项目ID> --token <JWT令牌>
```

详细文档: [tools/migrate/README.md](tools/migrate/README.md)

## 监控和日志

### 查看日志

```bash
# 所有服务
docker-compose logs -f

# 特定服务
docker-compose logs -f server
docker-compose logs -f postgres
docker-compose logs -f minio
docker-compose logs -f qdrant
```

### 健康检查

```bash
# 服务器健康检查
curl http://localhost:3000/health

# PostgreSQL 健康检查
docker exec llm-wiki-postgres pg_isready

# MinIO 健康检查
curl http://localhost:9000/minio/health/live

# Qdrant 健康检查
curl http://localhost:6333/healthz
```

## 备份和恢复

### 备份数据

```bash
# PostgreSQL 备份
docker exec llm-wiki-postgres pg_dump -U postgres llm_wiki > backup.sql

# MinIO 备份
docker run --rm \
  --volumes-from llm-wiki-minio \
  -v $(pwd):/backup \
  alpine tar czf /backup/minio-backup.tar.gz /data

# Qdrant 备份
docker run --rm \
  --volumes-from llm-wiki-qdrant \
  -v $(pwd):/backup \
  alpine tar czf /backup/qdrant-backup.tar.gz /qdrant/storage
```

### 恢复数据

```bash
# PostgreSQL 恢复
docker exec -i llm-wiki-postgres psql -U postgres llm_wiki < backup.sql

# MinIO 恢复
docker run --rm \
  --volumes-from llm-wiki-minio \
  -v $(pwd):/backup \
  alpine tar xzf /backup/minio-backup.tar.gz -C /

# Qdrant 恢复
docker run --rm \
  --volumes-from llm-wiki-qdrant \
  -v $(pwd):/backup \
  alpine tar xzf /backup/qdrant-backup.tar.gz -C /
```

## 安全建议

1. **修改默认密码**: 所有默认密码必须修改
2. **启用 HTTPS**: 生产环境必须使用 HTTPS
3. **配置防火墙**: 只开放必要端口
4. **定期更新**: 保持依赖和服务更新
5. **监控日志**: 定期检查日志发现异常
6. **备份数据**: 定期备份重要数据
7. **限制 CORS**: 配置允许的域名列表

## 故障排除

### 常见问题

#### 1. 服务无法启动

```bash
# 检查端口占用
netstat -tulpn | grep :3000

# 检查 Docker 日志
docker-compose logs server
```

#### 2. 数据库连接失败

```bash
# 检查 PostgreSQL 状态
docker exec llm-wiki-postgres pg_isready

# 检查连接字符串
echo $DATABASE_URL
```

#### 3. MinIO 无法访问

```bash
# 检查 MinIO 状态
docker exec llm-wiki-minio mc admin info local

# 访问控制台
open http://localhost:9001
```

#### 4. 向量搜索失败

```bash
# 检查 Qdrant 状态
curl http://localhost:6333/healthz

# 检查集合
curl http://localhost:6333/collections
```

## 性能优化

### 数据库优化

```sql
-- 创建索引
CREATE INDEX idx_files_project_id ON files(project_id);
CREATE INDEX idx_vectors_project_id ON vectors(project_id);

-- 分析表
ANALYZE files;
ANALYZE vectors;
```

### MinIO 优化

- 使用 SSD 存储
- 配置合适的分片大小
- 启用压缩（如果适用）

### Qdrant 优化

- 调整批量大小
- 使用合适的向量维度
- 配置索引参数

## 技术支持

- GitHub Issues: https://github.com/your-org/llm-wiki/issues
- 文档: https://your-docs-site.com
- 邮件: support@your-domain.com
