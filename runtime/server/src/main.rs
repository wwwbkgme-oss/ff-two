//! `runtime/server` — Binärer Einstiegspunkt für ForgeFabrik DevStudio.
//!
//! Verdrahtet alle Crates zu einem laufenden Axum-HTTP-Server:
//!
//! * Konfiguration aus `DEVSTUDIO_*` ENV-Variablen / `.env`
//! * MemoryStore + MemoryQueue + LocalSandboxManager (Dev-Standard)
//! * AgentRegistry: Free-LLM-Provider wenn Keys gesetzt, sonst Anthropic
//! * Orchestrator + DeploymentManager
//! * Axum-Server via `runtime/api`
//! * Task-Dispatch-Loop als Hintergrundtask (`dispatcher`)
//!
//! ## Agenten-Auswahl
//!
//! Wenn mindestens einer der kostenlosen Provider konfiguriert ist
//! (OPENROUTER_API_KEY, GROQ_API_KEY, CEREBRAS_API_KEY,
//!  SAMBANOVA_API_KEY, LLM7_API_KEY) oder `DEVSTUDIO_USE_FREE_LLM=true`
//! gesetzt ist, werden **alle 6 Rollen** mit `FreeLlmAgent` belegt.
//! Andernfalls: Anthropic Claude für Coding, einfache Agents für den Rest.

use std::sync::Arc;

use anyhow::Result;
use tracing::info;

mod dispatcher;

use agents::roles::{
    ArchitectureAgent, CodingAgent, DeploymentAgent,
    FreeLlmAgent, RequirementsAgent, SecurityAgent, TestingAgent,
};
use agents::{AgentRegistry, Orchestrator};
use api::AppState;
use config::Settings;
use deployment::{DeploymentManager, PipelineConfig};
use drivers::FreeProviderDriver;
use queue::MemoryQueue;
use sandbox::LocalSandboxManager;
use security::Scanner;
use store::{MemoryStore, PostgresStore, Store};
use world::WorldState;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::from_env()?;
    settings.init_tracing();

    let use_free = has_free_providers();
    info!(
        version   = env!("CARGO_PKG_VERSION"),
        host      = %settings.server.host,
        port      = settings.server.port,
        free_llm  = use_free,
        "ForgeFabrik DevStudio starting"
    );

    let state = build_app_state_async(settings, use_free).await?;

    // Dispatch-Loop im Hintergrund starten
    let _dispatch = dispatcher::spawn(state.clone());
    info!("dispatcher: Task-Dispatch-Loop gestartet");

    api::serve(state).await?;
    info!("DevStudio shutdown complete");
    Ok(())
}

/// Prüft, ob mindestens ein kostenloser LLM-Provider konfiguriert ist.
///
/// Prüfreihenfolge:
/// 1. `DEVSTUDIO_USE_FREE_LLM=true` — explizit aktivieren (nutzt auch nur Ollama)
/// 2. Einer der API-Keys ist gesetzt
fn has_free_providers() -> bool {
    if std::env::var("DEVSTUDIO_USE_FREE_LLM")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    [
        "OPENROUTER_API_KEY",
        "GROQ_API_KEY",
        "CEREBRAS_API_KEY",
        "SAMBANOVA_API_KEY",
        "LLM7_API_KEY",
    ]
    .iter()
    .any(|key| std::env::var(key).map(|v| !v.is_empty()).unwrap_or(false))
}

/// Verdrahtet alle Komponenten zu einem [`AppState`].
///
/// `use_free_llm`:
/// * `true`  → alle 6 Rollen nutzen `FreeLlmAgent` (kostenlose Provider)
/// * `false` → Coding nutzt `CodingAgent` (Anthropic), Rest: einfache Agents
pub async fn build_app_state_async(s: Settings, use_free_llm: bool) -> Result<AppState> {
    let store: Arc<dyn Store> = if s.database.url.starts_with("postgres") {
        info!("Store: PostgresStore ({})", &s.database.url[..s.database.url.find('@').unwrap_or(30).min(30)]);
        Arc::new(PostgresStore::connect_and_migrate(&s.database.url).await?)
    } else {
        info!("Store: MemoryStore (kein Postgres konfiguriert)");
        Arc::new(MemoryStore::new())
    };
    build_app_state_with_store(s, use_free_llm, store)
}

pub fn build_app_state(s: Settings, use_free_llm: bool) -> Result<AppState> {
    build_app_state_with_store(s, use_free_llm, Arc::new(MemoryStore::new()))
}

fn build_app_state_with_store(s: Settings, use_free_llm: bool, store: Arc<dyn Store>) -> Result<AppState> {
    use types::AgentRole;

    let mut registry = AgentRegistry::new();

    if use_free_llm {
        // ── Free-LLM-Modus: alle 6 Rollen via forgefabrik.llm-free ────────────
        info!("AgentRegistry: Free-LLM-Modus (runtime/drivers/llm)");
        let driver = Arc::new(FreeProviderDriver::new());
        for (role, agent) in FreeLlmAgent::all_roles(driver) {
            registry.register(role, agent as Arc<dyn agents::DevRolePlugin>);
        }
    } else {
        // ── Standard-Modus: Anthropic für Coding, einfache Agents für Rest ────
        info!("AgentRegistry: Standard-Modus (Anthropic Claude)");
        registry.register(AgentRole::Requirements, Arc::new(RequirementsAgent::new()));
        registry.register(AgentRole::Architecture, Arc::new(ArchitectureAgent::new()));
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
    }

    let mut orchestrator = Orchestrator::new(Arc::new(registry));
    orchestrator.consensus_threshold = s.deployment.require_consensus;

    let pipeline = PipelineConfig {
        require_consensus: s.deployment.require_consensus,
        staging_url:       s.deployment.staging_url.clone(),
        production_url:    s.deployment.production_url.clone(),
    };

    Ok(AppState {
        store,
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
