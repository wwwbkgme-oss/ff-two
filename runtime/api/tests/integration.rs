//! Integration-Tests für `runtime/api`.
//!
//! Jeder Test startet einen Axum-In-Process-Server (kein Netzwerk) via `axum-test`
//! und prüft Routing, Status-Codes und Response-Shapes.
//!
//! ## Konventionen
//! * Helper `test_state()` baut einen In-Memory-AppState.
//! * Tests sind `#[tokio::test]` und hängen **nicht** von externen Diensten ab.

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

// ── Hilfs-Konstruktor ──────────────────────────────────────────────────────

/// Erstellt einen vollständigen `AppState` für Tests — rein in-memory, kein I/O.
fn test_state() -> AppState {
    let settings = Settings::from_env().expect("Settings aus ENV nicht ladbar");

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

// ── Health / Ready ─────────────────────────────────────────────────────────

#[tokio::test]
async fn health_returns_ok() {
    let s = server();
    let r = s.get("/health").await;
    r.assert_status_ok();
    let body: serde_json::Value = r.json();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn ready_returns_ok() {
    let s = server();
    let r = s.get("/ready").await;
    r.assert_status_ok();
    let body: serde_json::Value = r.json();
    assert_eq!(body["status"], "ready");
}

// ── Projects ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_project_returns_201() {
    let s = server();
    let r = s.post("/projects")
        .json(&json!({ "name": "Test-Projekt" }))
        .await;
    r.assert_status_success();
    let body: serde_json::Value = r.json();
    assert_eq!(body["name"], "Test-Projekt");
    assert!(body["id"].is_string(), "id muss eine UUID-Zeichenkette sein");
}

#[tokio::test]
async fn create_project_empty_name_returns_400() {
    let s = server();
    let r = s.post("/projects")
        .json(&json!({ "name": "   " }))
        .await;
    r.assert_status_bad_request();
}

#[tokio::test]
async fn list_projects_initially_empty() {
    let s = server();
    let r = s.get("/projects").await;
    r.assert_status_ok();
    let body: serde_json::Value = r.json();
    assert_eq!(body["count"], 0);
    assert!(body["projects"].is_array());
}

#[tokio::test]
async fn get_project_not_found() {
    let s = server();
    let fake_id = uuid::Uuid::new_v4();
    let r = s.get(&format!("/projects/{fake_id}")).await;
    r.assert_status_not_found();
    let body: serde_json::Value = r.json();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn create_and_retrieve_project() {
    let s = server();

    // Anlegen
    let create = s.post("/projects")
        .json(&json!({ "name": "Mein Projekt", "description": "Beschreibung" }))
        .await;
    create.assert_status_success();
    let project: serde_json::Value = create.json();
    let id = project["id"].as_str().expect("id fehlt");

    // Abrufen
    let get = s.get(&format!("/projects/{id}")).await;
    get.assert_status_ok();
    let fetched: serde_json::Value = get.json();
    assert_eq!(fetched["id"], project["id"]);
    assert_eq!(fetched["name"], "Mein Projekt");
}

#[tokio::test]
async fn list_projects_after_create() {
    let s = server();

    s.post("/projects")
        .json(&json!({ "name": "A" }))
        .await
        .assert_status_success();
    s.post("/projects")
        .json(&json!({ "name": "B" }))
        .await
        .assert_status_success();

    let r = s.get("/projects").await;
    r.assert_status_ok();
    let body: serde_json::Value = r.json();
    assert_eq!(body["count"], 2);
}

// ── Tasks ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_task_returns_201() {
    let s = server();

    // Projekt anlegen
    let proj: serde_json::Value = s.post("/projects")
        .json(&json!({ "name": "Task-Test-Projekt" }))
        .await
        .json();
    let pid = proj["id"].as_str().unwrap();

    // Task anlegen
    let r = s.post(&format!("/projects/{pid}/tasks"))
        .json(&json!({ "title": "Feature X implementieren" }))
        .await;
    r.assert_status_success();
    let task: serde_json::Value = r.json();
    assert_eq!(task["title"], "Feature X implementieren");
    assert_eq!(task["project_id"], pid);
}

#[tokio::test]
async fn create_task_empty_title_returns_400() {
    let s = server();

    let proj: serde_json::Value = s.post("/projects")
        .json(&json!({ "name": "P" }))
        .await
        .json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.post(&format!("/projects/{pid}/tasks"))
        .json(&json!({ "title": "" }))
        .await;
    r.assert_status_bad_request();
}

#[tokio::test]
async fn list_tasks_initially_empty() {
    let s = server();

    let proj: serde_json::Value = s.post("/projects")
        .json(&json!({ "name": "P" }))
        .await
        .json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.get(&format!("/projects/{pid}/tasks")).await;
    r.assert_status_ok();
    let body: serde_json::Value = r.json();
    assert_eq!(body["count"], 0);
}

// ── World ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn world_state_returns_empty_initially() {
    let s = server();

    let proj: serde_json::Value = s.post("/projects")
        .json(&json!({ "name": "World-Test" }))
        .await
        .json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.get(&format!("/projects/{pid}/world")).await;
    r.assert_status_ok();
    let body: serde_json::Value = r.json();
    assert_eq!(body["chunk_count"], 0);
    assert_eq!(body["block_count"], 0);
}

// ── Sandbox ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_sandbox_returns_201() {
    let s = server();

    let proj: serde_json::Value = s.post("/projects")
        .json(&json!({ "name": "Sandbox-Test" }))
        .await
        .json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.post("/sandbox/dev")
        .json(&json!({ "project_id": pid }))
        .await;
    r.assert_status_success();
    let sb: serde_json::Value = r.json();
    assert!(sb["id"].is_string());
}

#[tokio::test]
async fn create_sandbox_missing_project_id_returns_400() {
    let s = server();
    let r = s.post("/sandbox/dev")
        .json(&json!({}))
        .await;
    r.assert_status_bad_request();
}

// ── Deployments ───────────────────────────────────────────────────────────

#[tokio::test]
async fn list_deployments_initially_empty() {
    let s = server();

    let proj: serde_json::Value = s.post("/projects")
        .json(&json!({ "name": "Deploy-Test" }))
        .await
        .json();
    let pid = proj["id"].as_str().unwrap();

    let r = s.get(&format!("/projects/{pid}/deployments")).await;
    r.assert_status_ok();
    let body: serde_json::Value = r.json();
    assert_eq!(body["count"], 0);
}
