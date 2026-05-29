use async_trait::async_trait;
use uuid::Uuid;

use errors::AppResult;
use types::{Agent, Deployment, Project, Sandbox, Task, WorldSnapshot};

/// Abstraktes Storage-Interface — alle Backends implementieren diesen Trait.
#[async_trait]
pub trait Store: Send + Sync + 'static {
    // ── Projects ──────────────────────────────────────────────────────────
    async fn create_project(&self, p: Project)         -> AppResult<Project>;
    async fn get_project(&self, id: Uuid)               -> AppResult<Project>;
    async fn list_projects(&self)                        -> AppResult<Vec<Project>>;
    async fn update_project(&self, p: Project)          -> AppResult<Project>;
    async fn delete_project(&self, id: Uuid)            -> AppResult<()>;

    // ── Tasks ─────────────────────────────────────────────────────────────
    async fn create_task(&self, t: Task)                -> AppResult<Task>;
    async fn get_task(&self, id: Uuid)                  -> AppResult<Task>;
    async fn list_tasks(&self, project_id: Uuid)        -> AppResult<Vec<Task>>;
    async fn update_task(&self, t: Task)                -> AppResult<Task>;
    async fn delete_task(&self, id: Uuid)               -> AppResult<()>;

    // ── Agents ────────────────────────────────────────────────────────────
    async fn create_agent(&self, a: Agent)              -> AppResult<Agent>;
    async fn get_agent(&self, id: Uuid)                 -> AppResult<Agent>;
    async fn list_agents(&self)                          -> AppResult<Vec<Agent>>;
    async fn update_agent(&self, a: Agent)              -> AppResult<Agent>;

    // ── Sandboxes ─────────────────────────────────────────────────────────
    async fn create_sandbox(&self, s: Sandbox)          -> AppResult<Sandbox>;
    async fn get_sandbox(&self, id: Uuid)               -> AppResult<Sandbox>;
    async fn list_sandboxes(&self, project_id: Uuid)    -> AppResult<Vec<Sandbox>>;
    async fn update_sandbox(&self, s: Sandbox)          -> AppResult<Sandbox>;

    // ── Deployments ───────────────────────────────────────────────────────
    async fn create_deployment(&self, d: Deployment)    -> AppResult<Deployment>;
    async fn get_deployment(&self, id: Uuid)            -> AppResult<Deployment>;
    async fn list_deployments(&self, project_id: Uuid)  -> AppResult<Vec<Deployment>>;
    async fn update_deployment(&self, d: Deployment)    -> AppResult<Deployment>;

    // ── World Snapshots ───────────────────────────────────────────────────
    async fn save_snapshot(&self, s: WorldSnapshot)     -> AppResult<WorldSnapshot>;
    async fn get_snapshot(&self, id: Uuid)              -> AppResult<WorldSnapshot>;
    async fn list_snapshots(&self, project_id: Uuid)    -> AppResult<Vec<WorldSnapshot>>;
}
