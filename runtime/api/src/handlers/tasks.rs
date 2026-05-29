use axum::{extract::{Path, State}, http::StatusCode, Json};
use chrono::Utc;
use uuid::Uuid;

use errors::AppError;
use types::{AssignTaskRequest, CreateTaskRequest, Task, TaskStatus};

use crate::errors::ApiResult;
use crate::state::AppState;

pub async fn create(State(s): State<AppState>, Path(pid): Path<Uuid>, Json(b): Json<CreateTaskRequest>) -> ApiResult<(StatusCode, Json<Task>)> {
    s.store.get_project(pid).await?;
    if b.title.trim().is_empty() { return Err(AppError::BadRequest("title ist erforderlich".into()).into()); }
    let mut t = Task::new(pid, b.title, b.description.unwrap_or_default(), b.priority.unwrap_or_default());
    t.required_role = b.required_role;
    let created = s.store.create_task(t).await?;
    s.queue.push(created.clone()).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn list(State(s): State<AppState>, Path(pid): Path<Uuid>) -> ApiResult<Json<serde_json::Value>> {
    let ts = s.store.list_tasks(pid).await?;
    let count = ts.len();
    Ok(Json(serde_json::json!({ "tasks": ts, "count": count })))
}

pub async fn get_by_id(State(s): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<Task>> {
    Ok(Json(s.store.get_task(id).await?))
}

pub async fn update(State(s): State<AppState>, Path(id): Path<Uuid>, Json(b): Json<serde_json::Value>) -> ApiResult<Json<Task>> {
    let mut t = s.store.get_task(id).await?;
    if let Some(st) = b.get("status").and_then(|v| serde_json::from_value(v.clone()).ok()) { t.status = st; }
    if let Some(n)  = b.get("note").and_then(|v| v.as_str()) { t.notes.push(n.to_string()); }
    t.updated_at = Utc::now();
    Ok(Json(s.store.update_task(t).await?))
}

pub async fn assign(State(s): State<AppState>, Path(id): Path<Uuid>, Json(b): Json<AssignTaskRequest>) -> ApiResult<Json<Task>> {
    let mut t    = s.store.get_task(id).await?;
    let agent    = s.store.get_agent(b.agent_id).await?;
    t.claimed_by = Some(agent.id);
    t.status     = TaskStatus::Claimed;
    t.updated_at = Utc::now();
    Ok(Json(s.store.update_task(t).await?))
}

pub async fn vote(State(s): State<AppState>, Path(id): Path<Uuid>, Json(b): Json<serde_json::Value>) -> ApiResult<Json<Task>> {
    let mut t = s.store.get_task(id).await?;
    t.consensus.insert(
        b.get("voter").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        b.get("approved").and_then(|v| v.as_bool()).unwrap_or(false),
    );
    t.updated_at = Utc::now();
    Ok(Json(s.store.update_task(t).await?))
}
