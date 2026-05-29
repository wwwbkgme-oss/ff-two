use std::sync::Arc;

use agents::Orchestrator;
use config::Settings;
use deployment::DeploymentManager;
use sandbox::SandboxManager;
use security::Scanner;
use store::Store;
use queue::TaskQueue;
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
}
