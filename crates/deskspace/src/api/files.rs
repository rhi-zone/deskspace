use std::path::Path;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::projection::Resource;
use crate::registry::ProjectionInfo;

#[derive(Deserialize)]
pub struct FileQuery {
    pub projection: Option<String>,
}

#[derive(Serialize)]
pub struct FileResponse {
    pub path: String,
    pub is_dir: bool,
    pub projections: Vec<ProjectionInfo>,
    pub active_projection: String,
    pub output: serde_json::Value,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_response(status: StatusCode, msg: impl Into<String>) -> Response {
    let body = Json(ErrorResponse { error: msg.into() });
    (status, body).into_response()
}

async fn project_resource(
    state: &Arc<AppState>,
    path: &str,
    query: &FileQuery,
) -> Result<Response, Response> {
    // Resolve the path to check it exists and stays in workspace
    let resolved = state
        .workspace
        .resolve(path)
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, e.to_string()))?;

    let meta = tokio::fs::metadata(&resolved)
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => error_response(StatusCode::NOT_FOUND, "not found"),
            _ => error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    let resource = Resource::new(path.to_string(), meta.is_dir());
    let projections = state.registry.available_for(&resource);

    // Pick the projection
    let projection = if let Some(ref id) = query.projection {
        state.registry.get(id).ok_or_else(|| {
            error_response(StatusCode::BAD_REQUEST, format!("unknown projection: {id}"))
        })?
    } else {
        state
            .registry
            .best_for(&resource)
            .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "no projection available"))?
    };

    let active_projection = projection.id().to_string();
    let output = projection
        .project(&resource, &state.workspace)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let output_value = serde_json::to_value(&output)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = FileResponse {
        path: path.to_string(),
        is_dir: meta.is_dir(),
        projections,
        active_projection,
        output: output_value,
    };

    Ok(Json(response).into_response())
}

pub async fn get_root(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FileQuery>,
) -> Response {
    match project_resource(&state, "", &query).await {
        Ok(r) => r,
        Err(r) => r,
    }
}

pub async fn get_file(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    Query(query): Query<FileQuery>,
) -> Response {
    match project_resource(&state, &path, &query).await {
        Ok(r) => r,
        Err(r) => r,
    }
}

pub async fn raw_file(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    let data = match state.workspace.read(Path::new(&path)).await {
        Ok(d) => d,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, e.to_string()),
    };

    let mime = mime_guess::from_path(&path)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .body(Body::from(data))
        .unwrap()
}

pub async fn put_file(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    body: axum::body::Bytes,
) -> Response {
    match state.workspace.write(Path::new(&path), &body).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error_response(StatusCode::BAD_REQUEST, e.to_string()),
    }
}
