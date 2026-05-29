use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    Router,
    routing::{delete, get, post, put},
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use super::handlers::{deployments, projects, sandbox, security, tasks, world};
use super::middleware::auth::issue_token;
use super::middleware::rate_limit::{rate_limit_middleware, RateLimiter};
use super::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new()
        // ── Auth (Dev-Token-Endpoint) ─────────────────────────────────────
        .route("/auth/token", post(issue_token))
        // ── Health ───────────────────────────────────────────────────────
        .route("/health",  get(health))
        .route("/ready",   get(ready))
        // ── Projects ─────────────────────────────────────────────────────
        .route("/projects",           post(projects::create))
        .route("/projects",           get(projects::list))
        .route("/projects/{id}",      get(projects::get_by_id))
        .route("/projects/{id}",      put(projects::update))
        .route("/projects/{id}",      delete(projects::archive))
        // ── Tasks ────────────────────────────────────────────────────────
        .route("/projects/{id}/tasks",        post(tasks::create))
        .route("/projects/{id}/tasks",        get(tasks::list))
        .route("/tasks/{id}",                 get(tasks::get_by_id))
        .route("/tasks/{id}",                 put(tasks::update))
        .route("/tasks/{id}/assign",          post(tasks::assign))
        .route("/tasks/{id}/consensus",       post(tasks::vote))
        // ── World ─────────────────────────────────────────────────────────
        .route("/projects/{id}/world",              get(world::get_state))
        .route("/projects/{id}/world/stream",       get(world::stream_events))
        .route("/projects/{id}/chunks/{cx}/{cz}",   get(world::get_chunk))
        .route("/projects/{id}/structures",         get(world::list_structures))
        .route("/projects/{id}/snapshots",          get(world::list_snapshots))
        .route("/projects/{id}/snapshots",          post(world::take_snapshot))
        // ── Sandbox ───────────────────────────────────────────────────────
        .route("/sandbox/dev",              post(sandbox::create))
        .route("/sandbox/{id}",             get(sandbox::get_by_id))
        .route("/sandbox/{id}",             delete(sandbox::destroy))
        .route("/sandbox/{id}/execute",     post(sandbox::execute))
        .route("/sandbox/{id}/diff",        get(sandbox::diff))
        .route("/sandbox/{id}/snapshot",    post(sandbox::snapshot))
        // ── Security ──────────────────────────────────────────────────────
        .route("/security/scan",            post(security::scan))
        .route("/projects/{id}/reviews",    get(security::list_reviews))
        // ── Deployments ───────────────────────────────────────────────────
        .route("/projects/{id}/deploy",         post(deployments::create))
        .route("/projects/{id}/deployments",    get(deployments::list))
        .route("/deployments/{id}",             get(deployments::get_by_id))
        .route("/deployments/{id}/rollback",    post(deployments::rollback))
        // ── Layers ────────────────────────────────────────────────────────
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(60)))
        .layer(axum_middleware::from_fn_with_state(
            RateLimiter::from_env(),
            rate_limit_middleware,
        ))
        .with_state(state)
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}
async fn ready() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ready", "version": env!("CARGO_PKG_VERSION") }))
}
