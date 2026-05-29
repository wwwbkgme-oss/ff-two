use chrono::Utc;
use tracing::info;

use errors::{AppError, AppResult};
use types::{Deployment, DeploymentEnv, DeploymentStatus};

use crate::pipeline::{PipelineConfig, PipelineStage};

/// Steuert die Deployment-Pipeline-Logik.
///
/// Enthält Geschäftsregeln:
/// - Welche Stufen müssen durchlaufen werden?
/// - Wann ist Konsens ausreichend?
/// - Wie wird die Ziel-URL bestimmt?
///
/// **Kein I/O**: Die Runtime-Schicht führt die echten Aktionen aus.
#[derive(Debug, Clone)]
pub struct DeploymentManager {
    pub config: PipelineConfig,
}

impl DeploymentManager {
    pub fn new(config: PipelineConfig) -> Self { Self { config } }

    /// Führt alle Pipeline-Stufen durch und gibt den aktualisierten Deployment-Record zurück.
    /// Die "Aktionen" sind hier als deterministischer Zustandsübergang modelliert.
    pub async fn run(&self, mut d: Deployment) -> AppResult<Deployment> {
        info!(id = %d.id, env = %d.env, version = %d.version, "deployment pipeline start");

        self.advance(&mut d, PipelineStage::Build,        DeploymentStatus::Building)?;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        d.logs.push(format!("[{}] Build succeeded", Utc::now()));

        self.advance(&mut d, PipelineStage::Test,         DeploymentStatus::Testing)?;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        d.logs.push(format!("[{}] Tests passed", Utc::now()));

        self.advance(&mut d, PipelineStage::SecurityScan, DeploymentStatus::SecurityScan)?;
        d.logs.push(format!("[{}] Security scan: clean", Utc::now()));

        self.advance(&mut d, PipelineStage::Deploy,       DeploymentStatus::Deploying)?;
        let url = self.target_url(&d);
        d.url = Some(url.clone());
        d.logs.push(format!("[{}] Deploying to {url}", Utc::now()));
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        self.advance(&mut d, PipelineStage::HealthCheck,  DeploymentStatus::Healthy)?;
        d.deployed_at = Some(Utc::now());
        d.logs.push(format!("[{}] Health check passed", Utc::now()));

        info!(id = %d.id, "deployment healthy");
        Ok(d)
    }

    /// Rollback zu einem früheren Stand.
    pub fn rollback(&self, mut d: Deployment) -> AppResult<Deployment> {
        d.status = DeploymentStatus::RolledBack;
        d.logs.push(format!("[{}] Rolled back", Utc::now()));
        Ok(d)
    }

    /// Prüft, ob ausreichend Konsens-Votes vorliegen.
    pub fn consensus_met(&self, approvals: &[String]) -> bool {
        approvals.len() >= self.config.require_consensus
    }

    // ── Hilfsmethoden ─────────────────────────────────────────────────────

    fn advance(&self, d: &mut Deployment, stage: PipelineStage, status: DeploymentStatus) -> AppResult<()> {
        tracing::debug!(id = %d.id, stage = %stage, "pipeline stage");
        d.status     = status;
        d.updated_at = Utc::now();
        Ok(())
    }

    fn target_url(&self, d: &Deployment) -> String {
        match &d.env {
            DeploymentEnv::Staging    => self.config.staging_url.clone()
                .unwrap_or_else(|| format!("https://staging.devstudio.local/{}", d.project_id)),
            DeploymentEnv::Production => self.config.production_url.clone()
                .unwrap_or_else(|| format!("https://devstudio.local/{}", d.project_id)),
            DeploymentEnv::Preview    => format!("https://preview-{}.devstudio.local", d.id),
        }
    }
}
