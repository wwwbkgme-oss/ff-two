//! Security-Handler — statische Code-Analyse und Review-Verwaltung.

use axum::{extract::{Path, State}, Json};
use uuid::Uuid;

use security::ScanRequest;

use crate::errors::ApiResult;
use crate::state::AppState;

/// `POST /security/scan` — statischer Scan (deterministisch, kein I/O).
pub async fn scan(
    State(s):  State<AppState>,
    Json(req): Json<ScanRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let result = s.scanner.scan(&req.changes);

    // Optional: Projekt-ID aus Kontext für Review-Speicherung
    if let Some(pid) = req.project_id {
        s.scan_reviews
            .entry(pid)
            .or_default()
            .push(result.clone());
        tracing::debug!(scan_id = %result.scan_id, project_id = %pid, "Scan-Ergebnis gespeichert");
    }

    Ok(Json(serde_json::json!({
        "scan_id":     result.scan_id,
        "passed":      result.passed,
        "summary":     result.summary,
        "findings":    result.findings,
        "critical":    result.critical_count(),
        "high":        result.high_count(),
        "duration_ms": result.duration_ms,
    })))
}

/// `GET /projects/{id}/reviews` — alle Scan-Ergebnisse für ein Projekt.
pub async fn list_reviews(
    State(s):  State<AppState>,
    Path(pid): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    s.store.get_project(pid).await?; // 404 wenn Projekt nicht existiert

    let reviews: Vec<_> = s.scan_reviews
        .get(&pid)
        .map(|r| r.clone())
        .unwrap_or_default();

    let count = reviews.len();
    Ok(Json(serde_json::json!({
        "project_id": pid,
        "reviews":    reviews,
        "count":      count,
    })))
}
