use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TavilySearchRequest {
    pub api_key: String,
    pub query: String,
    pub max_results: usize,
    #[serde(default = "default_search_depth")]
    pub search_depth: String,
    #[serde(default)]
    pub include_answer: bool,
}

fn default_search_depth() -> String {
    "advanced".to_string()
}

#[derive(Debug, Deserialize)]
pub struct TavilySearchResponse {
    pub results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
pub struct TavilyResult {
    pub title: Option<String>,
    pub url: Option<String>,
    pub content: Option<String>,
}

pub struct WebSearchService {
    client: Client,
}

impl WebSearchService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn tavily_search(
        &self,
        api_key: &str,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<WebSearchResult>, String> {
        let request = TavilySearchRequest {
            api_key: api_key.to_string(),
            query: query.to_string(),
            max_results,
            search_depth: "advanced".to_string(),
            include_answer: false,
        };

        let response = self.client
            .post("https://api.tavily.com/search")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Tavily: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Tavily search failed: {}", error_text));
        }

        let data: TavilySearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Tavily response: {}", e))?;

        let results = data.results
            .into_iter()
            .filter_map(|r| {
                let url = r.url.unwrap_or_default();
                if url.is_empty() {
                    return None;
                }

                let source = url::Url::parse(&url)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| h.replace("www.", "")))
                    .unwrap_or_else(|| "unknown".to_string());

                Some(WebSearchResult {
                    title: r.title.unwrap_or_else(|| "Untitled".to_string()),
                    url,
                    snippet: r.content.unwrap_or_default(),
                    source,
                })
            })
            .collect();

        Ok(results)
    }
}
