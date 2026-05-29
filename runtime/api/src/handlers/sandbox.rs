use axum::{extract::{Path, State}, http::StatusCode, Json};
use uuid::Uuid;

use errors::AppError;
use types::{Sandbox, SandboxExecRequest, SandboxExecResult};

use crate::errors::ApiResult;
use crate::state::AppState;

pub async fn create(State(s): State<AppState>, Json(b): Json<serde_json::Value>) -> ApiResult<(StatusCode, Json<Sandbox>)> {
    let pid = b.get("project_id").and_then(|v| v.as_str()).and_then(|v| v.parse::<Uuid>().ok())
        .ok_or_else(|| AppError::BadRequest("project_id ist erforderlich".into()))?;
    let sb = s.sandbox.create(pid).await?;
    Ok((StatusCode::CREATED, Json(s.store.create_sandbox(sb).await?)))
}

pub async fn get_by_id(State(s): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<Sandbox>> {
    Ok(Json(s.store.get_sandbox(id).await?))
}

pub async fn destroy(State(s): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<StatusCode> {
    s.sandbox.destroy(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn execute(State(s): State<AppState>, Path(id): Path<Uuid>, Json(req): Json<SandboxExecRequest>) -> ApiResult<Json<SandboxExecResult>> {
    Ok(Json(s.sandbox.execute(id, req).await?))
}

pub async fn diff(State(s): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "sandbox_id": id, "diff": s.sandbox.diff(id).await? })))
}

pub async fn snapshot(State(s): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "sandbox_id": id, "snapshot_id": s.sandbox.snapshot(id).await? })))
}
