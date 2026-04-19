use axum::Router;

use crate::config::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
