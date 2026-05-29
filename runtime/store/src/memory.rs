use async_trait::async_trait;
use dashmap::DashMap;
use uuid::Uuid;

use errors::{AppError, AppResult};
use types::{Agent, Deployment, Project, Sandbox, Task, WorldSnapshot};

use super::store::Store;

macro_rules! nf {
    ($kind:literal, $id:expr) => { AppError::not_found($kind, $id) };
}

/// Thread-sicherer In-Memory-Store auf Basis von DashMap.
/// Geeignet für Entwicklung und Tests; für Produktion gegen PostgresStore tauschen.
#[derive(Debug, Default)]
pub struct MemoryStore {
    projects:    DashMap<Uuid, Project>,
    tasks:       DashMap<Uuid, Task>,
    agents:      DashMap<Uuid, Agent>,
    sandboxes:   DashMap<Uuid, Sandbox>,
    deployments: DashMap<Uuid, Deployment>,
    snapshots:   DashMap<Uuid, WorldSnapshot>,
}

impl MemoryStore {
    pub fn new() -> Self { Self::default() }
}

#[async_trait]
impl Store for MemoryStore {
    async fn create_project(&self, p: Project)   -> AppResult<Project>       { self.projects.insert(p.id, p.clone()); Ok(p) }
    async fn get_project(&self, id: Uuid)         -> AppResult<Project>       { self.projects.get(&id).map(|r| r.clone()).ok_or_else(|| nf!("project", id)) }
    async fn list_projects(&self)                  -> AppResult<Vec<Project>>  { Ok(self.projects.iter().map(|r| r.clone()).collect()) }
    async fn update_project(&self, p: Project)    -> AppResult<Project>       { if !self.projects.contains_key(&p.id) { return Err(nf!("project", p.id)); } self.projects.insert(p.id, p.clone()); Ok(p) }
    async fn delete_project(&self, id: Uuid)      -> AppResult<()>            { self.projects.remove(&id).ok_or_else(|| nf!("project", id))?; Ok(()) }

    async fn create_task(&self, t: Task)          -> AppResult<Task>          { self.tasks.insert(t.id, t.clone()); Ok(t) }
    async fn get_task(&self, id: Uuid)            -> AppResult<Task>          { self.tasks.get(&id).map(|r| r.clone()).ok_or_else(|| nf!("task", id)) }
    async fn list_tasks(&self, pid: Uuid)         -> AppResult<Vec<Task>>     { Ok(self.tasks.iter().filter(|r| r.project_id == pid).map(|r| r.clone()).collect()) }
    async fn update_task(&self, t: Task)          -> AppResult<Task>          { if !self.tasks.contains_key(&t.id) { return Err(nf!("task", t.id)); } self.tasks.insert(t.id, t.clone()); Ok(t) }
    async fn delete_task(&self, id: Uuid)         -> AppResult<()>            { self.tasks.remove(&id).ok_or_else(|| nf!("task", id))?; Ok(()) }

    async fn create_agent(&self, a: Agent)        -> AppResult<Agent>         { self.agents.insert(a.id, a.clone()); Ok(a) }
    async fn get_agent(&self, id: Uuid)           -> AppResult<Agent>         { self.agents.get(&id).map(|r| r.clone()).ok_or_else(|| nf!("agent", id)) }
    async fn list_agents(&self)                    -> AppResult<Vec<Agent>>    { Ok(self.agents.iter().map(|r| r.clone()).collect()) }
    async fn update_agent(&self, a: Agent)        -> AppResult<Agent>         { if !self.agents.contains_key(&a.id) { return Err(nf!("agent", a.id)); } self.agents.insert(a.id, a.clone()); Ok(a) }

    async fn create_sandbox(&self, s: Sandbox)    -> AppResult<Sandbox>       { self.sandboxes.insert(s.id, s.clone()); Ok(s) }
    async fn get_sandbox(&self, id: Uuid)         -> AppResult<Sandbox>       { self.sandboxes.get(&id).map(|r| r.clone()).ok_or_else(|| nf!("sandbox", id)) }
    async fn list_sandboxes(&self, pid: Uuid)     -> AppResult<Vec<Sandbox>>  { Ok(self.sandboxes.iter().filter(|r| r.project_id == pid).map(|r| r.clone()).collect()) }
    async fn update_sandbox(&self, s: Sandbox)    -> AppResult<Sandbox>       { if !self.sandboxes.contains_key(&s.id) { return Err(nf!("sandbox", s.id)); } self.sandboxes.insert(s.id, s.clone()); Ok(s) }

    async fn create_deployment(&self, d: Deployment) -> AppResult<Deployment> { self.deployments.insert(d.id, d.clone()); Ok(d) }
    async fn get_deployment(&self, id: Uuid)          -> AppResult<Deployment> { self.deployments.get(&id).map(|r| r.clone()).ok_or_else(|| nf!("deployment", id)) }
    async fn list_deployments(&self, pid: Uuid)       -> AppResult<Vec<Deployment>> { Ok(self.deployments.iter().filter(|r| r.project_id == pid).map(|r| r.clone()).collect()) }
    async fn update_deployment(&self, d: Deployment)  -> AppResult<Deployment> { if !self.deployments.contains_key(&d.id) { return Err(nf!("deployment", d.id)); } self.deployments.insert(d.id, d.clone()); Ok(d) }

    async fn save_snapshot(&self, s: WorldSnapshot)  -> AppResult<WorldSnapshot> { self.snapshots.insert(s.id, s.clone()); Ok(s) }
    async fn get_snapshot(&self, id: Uuid)            -> AppResult<WorldSnapshot> { self.snapshots.get(&id).map(|r| r.clone()).ok_or_else(|| nf!("snapshot", id)) }
    async fn list_snapshots(&self, pid: Uuid)         -> AppResult<Vec<WorldSnapshot>> { Ok(self.snapshots.iter().filter(|r| r.project_id == pid).map(|r| r.clone()).collect()) }
}
