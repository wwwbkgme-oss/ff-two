//! Integration-Tests für `runtime/api`.
//!
//! Jeder Test startet einen Axum-In-Process-Server via `axum-test`.
//! Auth: Tests holen sich zuerst einen JWT-Token via `POST /auth/token`.

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::json;

use agents::{AgentRegistry, Orchestrator};
use api::{build_test_app, AppState};
use config::Settings;
use deployment::{DeploymentManager, PipelineConfig};
use queue::MemoryQueue;
use sandbox::LocalSandboxManager;
use security::Scanner;
use store::MemoryStore;
use world::WorldState;

// ── Hilfs-Konstruktor ─────────────────────────────────────────────────────────

fn test_state() -> AppState {
    let settings    = Settings::from_env().expect("Settings laden");
    let registry    = Arc::new(AgentRegistry::new());
    let orchestrator = Orchestrator::new(Arc::clone(&registry));
    let pipeline    = PipelineConfig::default();

    AppState {
        store:        Arc::new(MemoryStore::new()),
        queue:        Arc::new(MemoryQueue::new(64)),
        world:        Arc::new(WorldState::new()),
        sandbox:      Arc::new(LocalSandboxManager::new(settings.sandbox.clone())),
        scanner:      Arc::new(Scanner::new(false, false)),
        deployer:     Arc::new(DeploymentManager::new(pipeline)),
        orchestrator: Arc::new(orchestrator),
        settings:     Arc::new(settings),
    }
}

fn server() -> TestServer {
    TestServer::new(build_test_app(test_state()))
        .expect("TestServer konnte nicht gestartet werden")
}

/// JWT-Token für Tests holen. Nutzt den Standard-Dev-Secret aus Settings.
async fn auth_token(s: &TestServer) -> String {
    let secret = Settings::from_env()
        .map(|s| s.auth.jwt_secret)
        .unwrap_or_else(|_| "change-me-in-production-use-a-strong-random-value".into());

    let r = s.post("/auth/token")
        .json(&json!({ "sub": "test-user", "secret": secret }))
        .await;
    r.assert_status_ok();
    let body: serde_json::Value = r.json();
    body["token"].as_str().expect("token field").to_owned()
}

// ── Health / Ready (public) ───────────────────────────────────────────────────

#[tokio::test]
async fn health_returns_ok() {
    let s = server();
    let r = s.get("/health").await;
    r.assert_status_ok();
    assert_eq!(r.json::<serde_json::Value>()["status"], "ok");
}

#[tokio::test]
async fn ready_returns_ok() {
    let s = server();
    let r = s.get("/ready").await;
    r.assert_status_ok();
    assert_eq!(r.json::<serde_json::Value>()["status"], "ready");
}

// ── Auth ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn token_endpoint_returns_token() {
    let s      = server();
    let token  = auth_token(&s).await;
    assert!(!token.is_empty());
}

#[tokio::test]
async fn protected_route_without_token_returns_401() {
    let s = server();
    let r = s.get("/projects").await;
    r.assert_status_unauthorized();
}

// ── Projects ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_project_returns_201() {
    let s     = server();
    let token = auth_token(&s).await;
    let r = s.post("/projects")
        .add_header("Authorization", format!("Bearer {token}").parse().unwrap())
        .json(&json!({ "name": "Test-Projekt" }))
        .await;
    r.assert_status_success();
    let body: serde_json::Value = r.json();
    assert_eq!(body["name"], "Test-Projekt");
    assert!(body["id"].is_string());
}

#[tokio::test]
async fn create_project_empty_name_returns_400() {
    let s     = server();
    let token = auth_token(&s).await;
    let r = s.post("/projects")
        .add_header("Authorization", format!("Bearer {token}").parse().unwrap())
        .json(&json!({ "name": "   " }))
        .await;
    r.assert_status_bad_request();
}

#[tokio::test]
async fn list_projects_initially_empty() {
    let s     = server();
    let token = auth_token(&s).await;
    let r = s.get("/projects")
        .add_header("Authorization", format!("Bearer {token}").parse().unwrap())
        .await;
    r.assert_status_ok();
    assert_eq!(r.json::<serde_json::Value>()["count"], 0);
}

#[tokio::test]
async fn get_project_not_found() {
    let s     = server();
    let token = auth_token(&s).await;
    let fake  = uuid::Uuid::new_v4();
    let r     = s.get(&format!("/projects/{fake}"))
        .add_header("Authorization", format!("Bearer {token}").parse().unwrap())
        .await;
    r.assert_status_not_found();
    assert_eq!(r.json::<serde_json::Value>()["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn create_and_retrieve_project() {
    let s     = server();
    let token = auth_token(&s).await;
    let auth  = || format!("Bearer {token}").parse::<axum::http::HeaderValue>().unwrap();

    let create = s.post("/projects")
        .add_header("Authorization", auth())
        .json(&json!({ "name": "Mein Projekt", "description": "Test" }))
        .await;
    create.assert_status_success();
    let project: serde_json::Value = create.json();
    let id = project["id"].as_str().unwrap();

    let get = s.get(&format!("/projects/{id}"))
        .add_header("Authorization", auth())
        .await;
    get.assert_status_ok();
    assert_eq!(get.json::<serde_json::Value>()["name"], "Mein Projekt");
}

#[tokio::test]
async fn list_projects_after_create() {
    let s     = server();
    let token = auth_token(&s).await;
    let auth  = || format!("Bearer {token}").parse::<axum::http::HeaderValue>().unwrap();

    s.post("/projects").add_header("Authorization", auth()).json(&json!({ "name": "A" })).await.assert_status_success();
    s.post("/projects").add_header("Authorization", auth()).json(&json!({ "name": "B" })).await.assert_status_success();

    let r = s.get("/projects").add_header("Authorization", auth()).await;
    r.assert_status_ok();
    assert_eq!(r.json::<serde_json::Value>()["count"], 2);
}

// ── Tasks ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_task_returns_201() {
    let s     = server();
    let token = auth_token(&s).await;
    let auth  = || format!("Bearer {token}").parse::<axum::http::HeaderValue>().unwrap();

    let proj: serde_json::Value = s.post("/projects")
        .add_header("Authorization", auth())
        .json(&json!({ "name": "P" }))
        .await.json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.post(&format!("/projects/{pid}/tasks"))
        .add_header("Authorization", auth())
        .json(&json!({ "title": "Feature X" }))
        .await;
    r.assert_status_success();
    assert_eq!(r.json::<serde_json::Value>()["title"], "Feature X");
}

#[tokio::test]
async fn create_task_empty_title_returns_400() {
    let s     = server();
    let token = auth_token(&s).await;
    let auth  = || format!("Bearer {token}").parse::<axum::http::HeaderValue>().unwrap();

    let proj: serde_json::Value = s.post("/projects")
        .add_header("Authorization", auth())
        .json(&json!({ "name": "P" })).await.json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.post(&format!("/projects/{pid}/tasks"))
        .add_header("Authorization", auth())
        .json(&json!({ "title": "" })).await;
    r.assert_status_bad_request();
}

// ── World ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn world_state_returns_empty_initially() {
    let s     = server();
    let token = auth_token(&s).await;
    let auth  = || format!("Bearer {token}").parse::<axum::http::HeaderValue>().unwrap();

    let proj: serde_json::Value = s.post("/projects")
        .add_header("Authorization", auth())
        .json(&json!({ "name": "W" })).await.json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.get(&format!("/projects/{pid}/world"))
        .add_header("Authorization", auth()).await;
    r.assert_status_ok();
    assert_eq!(r.json::<serde_json::Value>()["chunk_count"], 0);
}

// ── Sandbox ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_sandbox_returns_201() {
    let s     = server();
    let token = auth_token(&s).await;
    let auth  = || format!("Bearer {token}").parse::<axum::http::HeaderValue>().unwrap();

    let proj: serde_json::Value = s.post("/projects")
        .add_header("Authorization", auth())
        .json(&json!({ "name": "S" })).await.json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.post("/sandbox/dev")
        .add_header("Authorization", auth())
        .json(&json!({ "project_id": pid })).await;
    r.assert_status_success();
    assert!(r.json::<serde_json::Value>()["id"].is_string());
}

#[tokio::test]
async fn create_sandbox_missing_project_id_returns_400() {
    let s     = server();
    let token = auth_token(&s).await;
    s.post("/sandbox/dev")
        .add_header("Authorization", format!("Bearer {token}").parse().unwrap())
        .json(&json!({})).await
        .assert_status_bad_request();
}

// ── Deployments ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_deployments_initially_empty() {
    let s     = server();
    let token = auth_token(&s).await;
    let auth  = || format!("Bearer {token}").parse::<axum::http::HeaderValue>().unwrap();

    let proj: serde_json::Value = s.post("/projects")
        .add_header("Authorization", auth())
        .json(&json!({ "name": "D" })).await.json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.get(&format!("/projects/{pid}/deployments"))
        .add_header("Authorization", auth()).await;
    r.assert_status_ok();
    assert_eq!(r.json::<serde_json::Value>()["count"], 0);
}
