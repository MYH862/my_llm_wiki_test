-- 添加数据库索引以优化查询性能

-- 用户表索引
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);

-- 项目表索引
CREATE INDEX IF NOT EXISTS idx_projects_owner_id ON projects(owner_id);
CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at);
CREATE INDEX IF NOT EXISTS idx_projects_is_archived ON projects(is_archived);

-- 项目成员表索引
CREATE INDEX IF NOT EXISTS idx_project_members_project_id ON project_members(project_id);
CREATE INDEX IF NOT EXISTS idx_project_members_user_id ON project_members(user_id);
CREATE INDEX IF NOT EXISTS idx_project_members_role ON project_members(role);

-- 刷新令牌表索引
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token ON refresh_tokens(token);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_is_revoked ON refresh_tokens(is_revoked);

-- 文件表索引
CREATE INDEX IF NOT EXISTS idx_files_project_id ON files(project_id);
CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
CREATE INDEX IF NOT EXISTS idx_files_created_at ON files(created_at);

-- LLM 配置表索引
CREATE INDEX IF NOT EXISTS idx_llm_configs_project_id ON llm_configs(project_id);
CREATE INDEX IF NOT EXISTS idx_llm_configs_is_active ON llm_configs(is_active);

-- 导入任务表索引
CREATE INDEX IF NOT EXISTS idx_ingest_tasks_project_id ON ingest_tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_ingest_tasks_status ON ingest_tasks(status);
CREATE INDEX IF NOT EXISTS idx_ingest_tasks_created_at ON ingest_tasks(created_at);

-- 审核表索引
CREATE INDEX IF NOT EXISTS idx_reviews_project_id ON reviews(project_id);
CREATE INDEX IF NOT EXISTS idx_reviews_status ON reviews(status);
CREATE INDEX IF NOT EXISTS idx_reviews_created_at ON reviews(created_at);

-- 研究任务表索引
CREATE INDEX IF NOT EXISTS idx_research_tasks_project_id ON research_tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_research_tasks_status ON research_tasks(status);
