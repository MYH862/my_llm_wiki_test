use axum::{
    extract::{State, Extension},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use crate::config::AppState;
use crate::middleware::auth::Claims;
use crate::middleware::error::AppError;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/structural", post(structural_lint))
        .route("/semantic", post(semantic_lint))
}

#[derive(Debug, Deserialize)]
pub struct StructuralLintRequest {
    pub project_id: String,
    pub pages: Vec<PageInput>,
}

#[derive(Debug, Deserialize)]
pub struct SemanticLintRequest {
    pub project_id: String,
    pub pages: Vec<PageInput>,
    pub llm_api_key: String,
    pub llm_api_url: String,
    pub llm_model: String,
}

#[derive(Debug, Deserialize)]
pub struct PageInput {
    pub id: String,
    pub title: String,
    pub content: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct LintResponse {
    pub results: Vec<LintResult>,
}

#[derive(Debug, Serialize)]
pub struct LintResult {
    #[serde(rename = "type")]
    pub lint_type: String,
    pub severity: String,
    pub page: String,
    pub detail: String,
    pub affected_pages: Option<Vec<String>>,
}

async fn structural_lint(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<StructuralLintRequest>,
) -> Result<Json<LintResponse>, AppError> {
    let mut results: Vec<LintResult> = Vec::new();

    let slug_map: std::collections::HashMap<String, String> = request.pages
        .iter()
        .map(|p| (p.id.clone(), p.path.clone()))
        .collect();

    let mut inbound_counts: std::collections::HashMap<String, usize> = request.pages
        .iter()
        .map(|p| (p.id.clone(), 0))
        .collect();

    let wikilink_regex = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]").unwrap();

    for page in &request.pages {
        if page.id == "index" || page.id == "log" {
            continue;
        }

        for cap in wikilink_regex.captures_iter(&page.content) {
            let link_target = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            if slug_map.contains_key(link_target) {
                *inbound_counts.entry(link_target.to_string()).or_insert(0) += 1;
            }
        }
    }

    for page in &request.pages {
        if page.id == "index" || page.id == "log" {
            continue;
        }

        let inbound = inbound_counts.get(&page.id).copied().unwrap_or(0);
        if inbound == 0 {
            results.push(LintResult {
                lint_type: "orphan".to_string(),
                severity: "info".to_string(),
                page: page.id.clone(),
                detail: "No other pages link to this page.".to_string(),
                affected_pages: None,
            });
        }

        let outlinks: Vec<_> = wikilink_regex
            .captures_iter(&page.content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
            .collect();

        if outlinks.is_empty() {
            results.push(LintResult {
                lint_type: "no-outlinks".to_string(),
                severity: "info".to_string(),
                page: page.id.clone(),
                detail: "This page has no [[wikilink]] references to other pages.".to_string(),
                affected_pages: None,
            });
        }

        for link in &outlinks {
            if !slug_map.contains_key(link) {
                results.push(LintResult {
                    lint_type: "broken-link".to_string(),
                    severity: "warning".to_string(),
                    page: page.id.clone(),
                    detail: format!("Broken link: [[{}]] — target page not found.", link),
                    affected_pages: None,
                });
            }
        }
    }

    Ok(Json(LintResponse { results }))
}

async fn semantic_lint(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(_request): Json<SemanticLintRequest>,
) -> Result<Json<LintResponse>, AppError> {
    Ok(Json(LintResponse {
        results: vec![],
    }))
}
