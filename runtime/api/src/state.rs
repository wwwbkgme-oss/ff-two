use std::sync::Arc;

use agents::Orchestrator;
use config::Settings;
use dashmap::DashMap;
use deployment::DeploymentManager;
use sandbox::SandboxManager;
use security::{Scanner, ScanResult};
use store::Store;
use queue::TaskQueue;
use uuid::Uuid;
use world::WorldState;

/// Geteilter Anwendungszustand — als Extension in jeden Handler injiziert.
#[derive(Clone)]
pub struct AppState {
    pub store:        Arc<dyn Store>,
    pub queue:        Arc<dyn TaskQueue>,
    pub world:        Arc<WorldState>,
    pub sandbox:      Arc<dyn SandboxManager>,
    pub scanner:      Arc<Scanner>,
    pub deployer:     Arc<DeploymentManager>,
    pub orchestrator: Arc<Orchestrator>,
    pub settings:     Arc<Settings>,
    /// In-Memory-Cache für Scan-Ergebnisse je Projekt.
    /// Key: project_id, Value: Liste der letzten Scan-Ergebnisse.
    pub scan_reviews: Arc<DashMap<Uuid, Vec<ScanResult>>>,
}
