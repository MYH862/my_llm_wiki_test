-- Create ingest_tasks table for tracking LLM ingestion tasks
CREATE TABLE IF NOT EXISTS ingest_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    content TEXT NOT NULL,
    metadata TEXT,
    step1_result TEXT,
    result TEXT,
    progress DOUBLE PRECISION DEFAULT 0.0,
    error TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Create llm_configs table for storing LLM configurations
CREATE TABLE IF NOT EXISTS llm_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    provider VARCHAR(50) NOT NULL,
    api_key TEXT,
    model VARCHAR(100) NOT NULL,
    ollama_url TEXT,
    custom_endpoint TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_ingest_tasks_user_id ON ingest_tasks(user_id);
CREATE INDEX IF NOT EXISTS idx_ingest_tasks_status ON ingest_tasks(status);
CREATE INDEX IF NOT EXISTS idx_llm_configs_user_id ON llm_configs(user_id);
