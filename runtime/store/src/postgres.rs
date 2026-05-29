//! PostgresStore — produktionsreifer Store auf Basis von sqlx + PostgreSQL.
//!
//! Aktivierung: `DEVSTUDIO_DATABASE_URL=postgres://user:pw@host/db`
//!
//! ## Schema-Strategie
//! Jede Entität wird als JSONB-Dokument gespeichert (id + data).
//! Schema-Änderungen an Domain-Typen erfordern keine DB-Migration.
//!
//! ## Migrations
//! Werden beim Start via `PostgresStore::connect_and_migrate()` einmalig
//! ausgeführt. Migrationsdateien liegen in `migrations/`.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use errors::{AppError, AppResult};
use types::{Agent, Deployment, Project, Sandbox, Task, WorldSnapshot};

use super::store::Store;

// ── Konstruktion ──────────────────────────────────────────────────────────────

pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    /// Verbindet mit der DB und führt ausstehende Migrations aus.
    pub async fn connect_and_migrate(url: &str) -> AppResult<Self> {
        let pool = PgPool::connect(url)
            .await
            .map_err(|e| AppError::Internal(format!("DB connect: {e}")))?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| AppError::Internal(format!("migration: {e}")))?;

        tracing::info!("PostgresStore: verbunden und migriert");
        Ok(Self { pool })
    }
}

// ── Generische Helfer ─────────────────────────────────────────────────────────

impl PostgresStore {
    /// INSERT (id, data) — schlägt fehl bei doppelter ID.
    async fn insert_simple<T: serde::Serialize>(
        &self, table: &str, id: Uuid, data: &T,
    ) -> AppResult<()> {
        let json = to_json(data)?;
        sqlx::query(&format!("INSERT INTO {table} (id, data) VALUES ($1, $2)"))
            .bind(id)
            .bind(json)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    /// INSERT (id, project_id, data) — für Entitäten mit project_id-Spalte.
    async fn insert_proj<T: serde::Serialize>(
        &self, table: &str, id: Uuid, project_id: Uuid, data: &T,
    ) -> AppResult<()> {
        let json = to_json(data)?;
        sqlx::query(&format!(
            "INSERT INTO {table} (id, project_id, data) VALUES ($1, $2, $3)"
        ))
        .bind(id)
        .bind(project_id)
        .bind(json)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    /// UPDATE data WHERE id = $1 — liefert NotFound wenn kein Row betroffen.
    async fn update_data<T: serde::Serialize>(
        &self, table: &str, kind: &str, id: Uuid, data: &T,
    ) -> AppResult<()> {
        let json = to_json(data)?;
        let res = sqlx::query(&format!("UPDATE {table} SET data = $1 WHERE id = $2"))
            .bind(json)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if res.rows_affected() == 0 {
            return Err(AppError::not_found(kind, id));
        }
        Ok(())
    }

    /// SELECT data WHERE id = $1.
    async fn get<T: serde::de::DeserializeOwned>(
        &self, table: &str, kind: &str, id: Uuid,
    ) -> AppResult<T> {
        let row: Option<(serde_json::Value,)> =
            sqlx::query_as(&format!("SELECT data FROM {table} WHERE id = $1"))
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        match row {
            Some((v,)) => from_json(v),
            None       => Err(AppError::not_found(kind, id)),
        }
    }

    /// SELECT data FROM table (alle Zeilen).
    async fn list_all<T: serde::de::DeserializeOwned>(
        &self, table: &str,
    ) -> AppResult<Vec<T>> {
        let rows: Vec<(serde_json::Value,)> =
            sqlx::query_as(&format!("SELECT data FROM {table}"))
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        rows.into_iter().map(|(v,)| from_json(v)).collect()
    }

    /// SELECT data WHERE project_id = $1.
    async fn list_by_project<T: serde::de::DeserializeOwned>(
        &self, table: &str, project_id: Uuid,
    ) -> AppResult<Vec<T>> {
        let rows: Vec<(serde_json::Value,)> = sqlx::query_as(&format!(
            "SELECT data FROM {table} WHERE project_id = $1"
        ))
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        rows.into_iter().map(|(v,)| from_json(v)).collect()
    }

    /// DELETE WHERE id = $1 — liefert NotFound wenn kein Row betroffen.
    async fn delete(&self, table: &str, kind: &str, id: Uuid) -> AppResult<()> {
        let res = sqlx::query(&format!("DELETE FROM {table} WHERE id = $1"))
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if res.rows_affected() == 0 {
            return Err(AppError::not_found(kind, id));
        }
        Ok(())
    }
}

// ── Store-Trait-Implementierung ───────────────────────────────────────────────

#[async_trait]
impl Store for PostgresStore {
    // Projects
    async fn create_project(&self, p: Project) -> AppResult<Project> {
        self.insert_simple("ds_projects", p.id, &p).await?;
        Ok(p)
    }
    async fn get_project(&self, id: Uuid) -> AppResult<Project> {
        self.get("ds_projects", "project", id).await
    }
    async fn list_projects(&self) -> AppResult<Vec<Project>> {
        self.list_all("ds_projects").await
    }
    async fn update_project(&self, p: Project) -> AppResult<Project> {
        self.update_data("ds_projects", "project", p.id, &p).await?;
        Ok(p)
    }
    async fn delete_project(&self, id: Uuid) -> AppResult<()> {
        self.delete("ds_projects", "project", id).await
    }

    // Tasks
    async fn create_task(&self, t: Task) -> AppResult<Task> {
        self.insert_proj("ds_tasks", t.id, t.project_id, &t).await?;
        Ok(t)
    }
    async fn get_task(&self, id: Uuid) -> AppResult<Task> {
        self.get("ds_tasks", "task", id).await
    }
    async fn list_tasks(&self, project_id: Uuid) -> AppResult<Vec<Task>> {
        self.list_by_project("ds_tasks", project_id).await
    }
    async fn update_task(&self, t: Task) -> AppResult<Task> {
        self.update_data("ds_tasks", "task", t.id, &t).await?;
        Ok(t)
    }
    async fn delete_task(&self, id: Uuid) -> AppResult<()> {
        self.delete("ds_tasks", "task", id).await
    }

    // Agents
    async fn create_agent(&self, a: Agent) -> AppResult<Agent> {
        self.insert_simple("ds_agents", a.id, &a).await?;
        Ok(a)
    }
    async fn get_agent(&self, id: Uuid) -> AppResult<Agent> {
        self.get("ds_agents", "agent", id).await
    }
    async fn list_agents(&self) -> AppResult<Vec<Agent>> {
        self.list_all("ds_agents").await
    }
    async fn update_agent(&self, a: Agent) -> AppResult<Agent> {
        self.update_data("ds_agents", "agent", a.id, &a).await?;
        Ok(a)
    }

    // Sandboxes
    async fn create_sandbox(&self, s: Sandbox) -> AppResult<Sandbox> {
        self.insert_proj("ds_sandboxes", s.id, s.project_id, &s).await?;
        Ok(s)
    }
    async fn get_sandbox(&self, id: Uuid) -> AppResult<Sandbox> {
        self.get("ds_sandboxes", "sandbox", id).await
    }
    async fn list_sandboxes(&self, project_id: Uuid) -> AppResult<Vec<Sandbox>> {
        self.list_by_project("ds_sandboxes", project_id).await
    }
    async fn update_sandbox(&self, s: Sandbox) -> AppResult<Sandbox> {
        self.update_data("ds_sandboxes", "sandbox", s.id, &s).await?;
        Ok(s)
    }

    // Deployments
    async fn create_deployment(&self, d: Deployment) -> AppResult<Deployment> {
        self.insert_proj("ds_deployments", d.id, d.project_id, &d).await?;
        Ok(d)
    }
    async fn get_deployment(&self, id: Uuid) -> AppResult<Deployment> {
        self.get("ds_deployments", "deployment", id).await
    }
    async fn list_deployments(&self, project_id: Uuid) -> AppResult<Vec<Deployment>> {
        self.list_by_project("ds_deployments", project_id).await
    }
    async fn update_deployment(&self, d: Deployment) -> AppResult<Deployment> {
        self.update_data("ds_deployments", "deployment", d.id, &d).await?;
        Ok(d)
    }

    // World Snapshots
    async fn save_snapshot(&self, s: WorldSnapshot) -> AppResult<WorldSnapshot> {
        self.insert_proj("ds_snapshots", s.id, s.project_id, &s).await?;
        Ok(s)
    }
    async fn get_snapshot(&self, id: Uuid) -> AppResult<WorldSnapshot> {
        self.get("ds_snapshots", "snapshot", id).await
    }
    async fn list_snapshots(&self, project_id: Uuid) -> AppResult<Vec<WorldSnapshot>> {
        self.list_by_project("ds_snapshots", project_id).await
    }
}

// ── Hilfs-Konvertierungen ─────────────────────────────────────────────────────

fn to_json<T: serde::Serialize>(v: &T) -> AppResult<serde_json::Value> {
    serde_json::to_value(v).map_err(|e| AppError::Internal(e.to_string()))
}

fn from_json<T: serde::de::DeserializeOwned>(v: serde_json::Value) -> AppResult<T> {
    serde_json::from_value(v).map_err(|e| AppError::Internal(e.to_string()))
}
