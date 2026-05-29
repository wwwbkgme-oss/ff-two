use axum::{extract::{Path, State}, http::StatusCode, Json};
use uuid::Uuid;

use types::{CreateDeploymentRequest, Deployment};

use crate::errors::ApiResult;
use crate::state::AppState;

pub async fn create(State(s): State<AppState>, Path(pid): Path<Uuid>, Json(b): Json<CreateDeploymentRequest>) -> ApiResult<(StatusCode, Json<Deployment>)> {
    s.store.get_project(pid).await?;
    let version = b.version.unwrap_or_else(|| chrono::Utc::now().timestamp().to_string());
    let mut d    = Deployment::new(pid, b.env, version);
    d.commit_sha = b.commit_sha;
    let saved    = s.store.create_deployment(d).await?;
    let (deployer, store, clone) = (s.deployer.clone(), s.store.clone(), saved.clone());
    tokio::spawn(async move {
        match deployer.run(clone).await {
            Ok(u)  => { let _ = store.update_deployment(u).await; }
            Err(e) => { tracing::error!(error = %e, "deployment pipeline fehler"); }
        }
    });
    Ok((StatusCode::ACCEPTED, Json(saved)))
}

pub async fn list(State(s): State<AppState>, Path(pid): Path<Uuid>) -> ApiResult<Json<serde_json::Value>> {
    let ds = s.store.list_deployments(pid).await?;
    let count = ds.len();
    Ok(Json(serde_json::json!({ "deployments": ds, "count": count })))
}

pub async fn get_by_id(State(s): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<Deployment>> {
    Ok(Json(s.store.get_deployment(id).await?))
}

pub async fn rollback(State(s): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<Deployment>> {
    let d  = s.store.get_deployment(id).await?;
    let rb = s.deployer.rollback(d)?;
    Ok(Json(s.store.update_deployment(rb).await?))
}
