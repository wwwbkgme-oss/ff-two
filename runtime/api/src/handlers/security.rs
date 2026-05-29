use axum::{extract::{Path, State}, Json};
use uuid::Uuid;

use security::{ScanRequest, ScanResult};

use crate::errors::ApiResult;
use crate::state::AppState;

pub async fn scan(State(s): State<AppState>, Json(req): Json<ScanRequest>) -> ApiResult<Json<ScanResult>> {
    Ok(Json(s.scanner.scan(&req.changes)))
}

pub async fn list_reviews(State(_s): State<AppState>, Path(pid): Path<Uuid>) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "project_id": pid, "reviews": [], "count": 0 })))
}
