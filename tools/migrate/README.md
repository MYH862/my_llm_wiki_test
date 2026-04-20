# 数据迁移工具

本目录包含从 Tauri 本地存储迁移到客户端-服务器架构的所有工具。

## 快速开始

### 1. 安装依赖

```bash
pip install requests minio qdrant-client lancedb
```

### 2. 一键迁移

```bash
python migrate.py <项目路径> <项目ID> --token <JWT令牌>
```

示例:

```bash
python migrate.py /path/to/my/wiki-project 550e8400-e29b-41d4-a716-446655440000 --token eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## 单独脚本

### 文件迁移

将本地文件批量上传到 MinIO:

```bash
python migrate_files.py <项目路径> <项目ID> [选项]
```

选项:
- `--minio-url`: MinIO 服务地址 (默认: http://localhost:9000)
- `--access-key`: MinIO 访问密钥 (默认: minioadmin)
- `--secret-key`: MinIO 密钥 (默认: minioadmin)
- `--bucket`: MinIO 存储桶名称 (默认: wiki-files)
- `--token`: JWT 认证令牌 (可选)
- `--exclude`: 排除的目录 (默认: .git node_modules .cache)

### 向量数据导出

从 LanceDB 导出向量数据:

```bash
python export_lancedb.py <项目路径> [选项]
```

选项:
- `--output`: 输出文件路径 (默认: vectors_export.json)
- `--batch-size`: 批次大小 (默认: 100)

### 向量数据导入

将导出的向量数据导入到 Qdrant:

```bash
python import_qdrant.py <导出文件> <项目ID> [选项]
```

选项:
- `--qdrant-url`: Qdrant 服务地址 (默认: http://localhost:6333)
- `--api-key`: Qdrant API 密钥 (可选)
- `--collection`: 集合名称 (默认: wiki-vectors-{project_id})
- `--vector-size`: 向量维度 (默认: 384)
- `--batch-size`: 批次大小 (默认: 100)

### 配置迁移

导出或导入 Tauri Store 配置:

```bash
# 导出配置
python migrate_config.py export <store文件路径> --output <输出文件>

# 导入配置
python migrate_config.py import <导出文件> --token <JWT令牌> --project-id <项目ID>
```

选项:
- `--api-url`: API 服务地址 (默认: http://localhost:3000/api)
- `--token`: JWT 认证令牌 (导入时必需)
- `--project-id`: 目标项目 ID (导入时必需)

### 迁移验证

验证迁移是否成功:

```bash
python verify_migration.py <项目ID> --token <JWT令牌> [选项]
```

选项:
- `--api-url`: API 服务地址 (默认: http://localhost:3000/api)
- `--token`: JWT 认证令牌 (必需)
- `--minio-url`: MinIO 服务地址 (默认: http://localhost:9000)
- `--access-key`: MinIO 访问密钥 (默认: minioadmin)
- `--secret-key`: MinIO 密钥 (默认: minioadmin)
- `--qdrant-url`: Qdrant 服务地址 (默认: http://localhost:6333)

## 迁移流程

完整的迁移流程如下:

1. **准备阶段**
   - 确保后端服务已启动 (PostgreSQL, MinIO, Qdrant)
   - 获取 JWT 认证令牌 (通过登录 API)

2. **文件迁移**
   - 运行 `migrate_files.py` 上传所有文件到 MinIO

3. **向量迁移**
   - 运行 `export_lancedb.py` 导出 LanceDB 向量
   - 运行 `import_qdrant.py` 导入到 Qdrant

4. **配置迁移**
   - 运行 `migrate_config.py export` 导出配置
   - 运行 `migrate_config.py import` 导入配置

5. **验证**
   - 运行 `verify_migration.py` 验证迁移结果

## 注意事项

1. **备份数据**: 迁移前请务必备份原始数据
2. **JWT 令牌**: 确保令牌有效且具有足够权限
3. **网络连接**: 确保可以访问 MinIO 和 Qdrant 服务
4. **磁盘空间**: 导出向量数据可能需要较大磁盘空间
5. **错误处理**: 如果某个步骤失败，可以单独重新运行该步骤

## 故障排除

### MinIO 连接失败

检查 MinIO 服务是否运行:

```bash
docker ps | grep minio
```

### Qdrant 连接失败

检查 Qdrant 服务是否运行:

```bash
docker ps | grep qdrant
```

### 权限错误

确保 JWT 令牌有效:

```bash
curl -H "Authorization: Bearer <token>" http://localhost:3000/api/health
```

### 向量维度不匹配

如果导入时出现维度错误，检查 `--vector-size` 参数是否与导出时的向量维度一致。
