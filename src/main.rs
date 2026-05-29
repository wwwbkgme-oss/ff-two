// ForgeFabrik DevStudio — binary entry point.
// Wires all crates together and starts the HTTP server.

use std::sync::Arc;
use anyhow::Context;
use tracing::info;

use devstudio_api::AppState;
use devstudio_config::Settings;
use devstudio_store::{Store, memory::MemoryStore};
use devstudio_queue::{TaskQueue, memory::MemoryQueue};
use devstudio_world::WorldState;
use devstudio_sandbox::{SandboxManager, local::LocalSandboxManager};
use devstudio_security::Scanner;
use devstudio_deployment::DeploymentManager;
use devstudio_agents::Orchestrator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let settings = Settings::from_env().context("loading configuration")?;
    settings.init_tracing();
    info!(version = env!("CARGO_PKG_VERSION"), addr = %settings.server.addr(), "devstudio starting");

    let store:   Arc<dyn Store>          = Arc::new(MemoryStore::new());
    let queue:   Arc<dyn TaskQueue>      = Arc::new(MemoryQueue::new(settings.queue.capacity));
    let world                            = Arc::new(WorldState::new());
    let sandbox: Arc<dyn SandboxManager> = Arc::new(LocalSandboxManager::new(settings.sandbox.clone()));
    let scanner                          = Arc::new(Scanner::new(settings.security.clone()));
    let deployer                         = Arc::new(DeploymentManager::new(settings.deployment.clone()));
    let orchestrator = Arc::new(Orchestrator::new(
        Arc::clone(&store), Arc::clone(&queue), Arc::clone(&world),
        Arc::clone(&sandbox), Arc::clone(&scanner), settings.agents.clone(),
    ));
    orchestrator.start().await;

    devstudio_api::serve(AppState {
        store, queue, world, sandbox, scanner, deployer, orchestrator,
        settings: Arc::new(settings),
    }).await
}
