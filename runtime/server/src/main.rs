//! `runtime/server` — Binärer Einstiegspunkt für ForgeFabrik DevStudio.
//!
//! Verdrahtet alle Crates zu einem laufenden Axum-HTTP-Server:
//!
//! * Konfiguration aus `DEVSTUDIO_*` ENV-Variablen / `.env`
//! * MemoryStore + MemoryQueue + LocalSandboxManager (Dev-Standard)
//! * AgentRegistry mit allen 6 Rollen-Plugins
//! * Orchestrator + DeploymentManager
//! * Axum-Server via `runtime/api`

use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use agents::roles::{
    ArchitectureAgent, CodingAgent, DeploymentAgent,
    RequirementsAgent, SecurityAgent, TestingAgent,
};
use agents::{AgentRegistry, Orchestrator};
use api::AppState;
use config::Settings;
use deployment::{DeploymentManager, PipelineConfig};
use queue::MemoryQueue;
use sandbox::LocalSandboxManager;
use security::Scanner;
use store::MemoryStore;
use world::WorldState;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::from_env()?;
    settings.init_tracing();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        host    = %settings.server.host,
        port    = settings.server.port,
        "ForgeFabrik DevStudio starting"
    );

    let state = build_app_state(settings)?;
    api::serve(state).await?;
    info!("DevStudio shutdown complete");
    Ok(())
}

/// Verdrahtet alle Komponenten zu einem [`AppState`].
///
/// Kann von Tests oder anderen Binaries wiederverwendet werden.
pub fn build_app_state(s: Settings) -> Result<AppState> {
    use types::AgentRole;

    // ── Agent-Registry: alle 6 Rollen ────────────────────────────────────────
    let mut registry = AgentRegistry::new();
    registry.register(
        AgentRole::Requirements,
        Arc::new(RequirementsAgent::new()),
    );
    registry.register(
        AgentRole::Architecture,
        Arc::new(ArchitectureAgent::new()),
    );
    registry.register(
        AgentRole::Coding,
        Arc::new(CodingAgent::new(
            &s.agents.model,
            s.agents.max_tokens,
            s.agents.anthropic_api_key.clone(),
        )),
    );
    registry.register(AgentRole::Testing,    Arc::new(TestingAgent::new()));
    registry.register(AgentRole::Security,   Arc::new(SecurityAgent::new()));
    registry.register(AgentRole::Deployment, Arc::new(DeploymentAgent::new()));

    let mut orchestrator = Orchestrator::new(Arc::new(registry));
    orchestrator.consensus_threshold = s.deployment.require_consensus;

    let pipeline = PipelineConfig {
        require_consensus: s.deployment.require_consensus,
        staging_url:       s.deployment.staging_url.clone(),
        production_url:    s.deployment.production_url.clone(),
    };

    Ok(AppState {
        store:        Arc::new(MemoryStore::new()),
        queue:        Arc::new(MemoryQueue::new(s.queue.capacity)),
        world:        Arc::new(WorldState::new()),
        sandbox:      Arc::new(LocalSandboxManager::new(s.sandbox.clone())),
        scanner:      Arc::new(Scanner::new(
            s.security.block_on_critical,
            s.security.block_on_high,
        )),
        deployer:     Arc::new(DeploymentManager::new(pipeline)),
        orchestrator: Arc::new(orchestrator),
        settings:     Arc::new(s),
    })
}
