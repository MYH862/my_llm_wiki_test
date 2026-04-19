use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub storage_path: String,
    pub template: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub llm_provider: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub embedding_model: Option<String>,
    pub vector_db_url: Option<String>,
    pub max_context_tokens: Option<i32>,
    pub enable_auto_index: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectSettingsRequest {
    pub llm_provider: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub embedding_model: Option<String>,
    pub vector_db_url: Option<String>,
    pub max_context_tokens: Option<i32>,
    pub enable_auto_index: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub total_files: i64,
    pub total_vectors: i64,
    pub total_graph_nodes: i64,
    pub total_graph_edges: i64,
    pub last_indexed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenProjectResponse {
    pub project: ProjectResponse,
    pub settings: ProjectSettings,
    pub stats: ProjectStats,
    pub user_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub storage_path: String,
    pub template: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            owner_id: project.owner_id,
            storage_path: project.storage_path,
            template: project.template,
            is_active: project.is_active,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddProjectMemberRequest {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectMember {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub role_name: String,
}
