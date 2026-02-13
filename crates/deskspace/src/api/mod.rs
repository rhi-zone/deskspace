pub mod files;

use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

use crate::registry::ProjectionRegistry;
use crate::workspace::Workspace;

pub struct AppState {
    pub workspace: Workspace,
    pub registry: ProjectionRegistry,
}

/// CSRF middleware: reject mutating requests unless Origin is localhost.
pub async fn csrf_check(request: Request, next: Next) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    if method == axum::http::Method::GET || method == axum::http::Method::HEAD {
        return Ok(next.run(request).await);
    }

    let origin = request
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if origin.is_empty() {
        // No origin header — could be same-origin or non-browser. Allow for now
        // but in production this should be tightened.
        return Ok(next.run(request).await);
    }

    let is_localhost = origin.starts_with("http://127.0.0.1")
        || origin.starts_with("http://localhost")
        || origin.starts_with("http://[::1]");

    if is_localhost {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

pub fn router(state: Arc<AppState>) -> axum::Router {
    use axum::middleware;
    use axum::routing::get;

    axum::Router::new()
        .route("/api/files/raw/{*path}", get(files::raw_file))
        .route("/api/files/", get(files::get_root))
        .route(
            "/api/files/{*path}",
            get(files::get_file).put(files::put_file),
        )
        .layer(middleware::from_fn(csrf_check))
        .with_state(state)
}
