use axum::{
    extract::{State, Extension},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;
use crate::services::graph::{
    GraphService, WikiPage, GraphNode, GraphEdge, CommunityInfo,
    find_surprising_connections, detect_knowledge_gaps, SurprisingConnection, KnowledgeGap,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/build", post(build_graph))
        .route("/insights", post(get_insights))
}

#[derive(Debug, Deserialize)]
pub struct BuildGraphRequest {
    pub pages: Vec<PageInput>,
}

#[derive(Debug, Deserialize)]
pub struct PageInput {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub wikilinks: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct InsightsRequest {
    pub pages: Vec<PageInput>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub communities: Vec<CommunityInfo>,
}

#[derive(Debug, Serialize)]
pub struct InsightsResponse {
    pub surprising_connections: Vec<SurprisingConnection>,
    pub knowledge_gaps: Vec<KnowledgeGap>,
}

async fn build_graph(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<BuildGraphRequest>,
) -> Result<Json<GraphResponse>, AppError> {
    let pages: Vec<WikiPage> = request.pages
        .into_iter()
        .map(|p| WikiPage {
            id: p.id,
            title: p.title,
            content: p.content,
            node_type: p.node_type,
            wikilinks: p.wikilinks,
        })
        .collect();

    let wiki_graph = GraphService::build_graph(pages);

    Ok(Json(GraphResponse {
        nodes: wiki_graph.nodes,
        edges: wiki_graph.edges,
        communities: wiki_graph.communities,
    }))
}

async fn get_insights(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<InsightsRequest>,
) -> Result<Json<InsightsResponse>, AppError> {
    let pages: Vec<WikiPage> = request.pages
        .into_iter()
        .map(|p| WikiPage {
            id: p.id,
            title: p.title,
            content: p.content,
            node_type: p.node_type,
            wikilinks: p.wikilinks,
        })
        .collect();

    let wiki_graph = GraphService::build_graph(pages);

    let limit = request.limit.unwrap_or(5);
    let surprising = find_surprising_connections(
        &wiki_graph.nodes,
        &wiki_graph.edges,
        &wiki_graph.communities,
        limit,
    );

    let gaps = detect_knowledge_gaps(
        &wiki_graph.nodes,
        &wiki_graph.edges,
        &wiki_graph.communities,
        8,
    );

    Ok(Json(InsightsResponse {
        surprising_connections: surprising,
        knowledge_gaps: gaps,
    }))
}
