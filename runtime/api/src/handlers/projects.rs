use axum::{extract::{Path, State}, http::StatusCode, Json};
use chrono::Utc;
use uuid::Uuid;

use errors::AppError;
use types::{CreateProjectRequest, Project, ProjectStatus, ProjectTemplate, UpdateProjectRequest};

use crate::errors::ApiResult;
use crate::state::AppState;

pub async fn create(State(s): State<AppState>, Json(b): Json<CreateProjectRequest>) -> ApiResult<(StatusCode, Json<Project>)> {
    if b.name.trim().is_empty() { return Err(AppError::BadRequest("name ist erforderlich".into()).into()); }
    let p = Project::new(b.name, b.description.unwrap_or_default(), b.template.unwrap_or(ProjectTemplate::Custom));
    Ok((StatusCode::CREATED, Json(s.store.create_project(p).await?)))
}

pub async fn list(State(s): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let ps = s.store.list_projects().await?;
    let count = ps.len();
    Ok(Json(serde_json::json!({ "projects": ps, "count": count })))
}

pub async fn get_by_id(State(s): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<Project>> {
    Ok(Json(s.store.get_project(id).await?))
}

pub async fn update(State(s): State<AppState>, Path(id): Path<Uuid>, Json(b): Json<UpdateProjectRequest>) -> ApiResult<Json<Project>> {
    let mut p = s.store.get_project(id).await?;
    if let Some(d)  = b.description { p.description = d; }
    if let Some(st) = b.status      { p.status      = st; }
    if let Some(br) = b.branch      { p.branch      = br; }
    if let Some(m)  = b.metadata    { p.metadata    = m; }
    p.updated_at = Utc::now();
    Ok(Json(s.store.update_project(p).await?))
}

pub async fn archive(State(s): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<StatusCode> {
    let mut p = s.store.get_project(id).await?;
    p.status     = ProjectStatus::Archived;
    p.updated_at = Utc::now();
    s.store.update_project(p).await?;
    Ok(StatusCode::NO_CONTENT)
}
