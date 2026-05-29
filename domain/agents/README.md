# `domain/agents`

`DevRolePlugin`-Trait, `LlmDriver`-Trait, 6+1 Agenten-Rollen und der `Orchestrator`.

## BKG-Regeln

- Reine Domänenlogik: **kein HTTP, kein I/O**
- Der `Orchestrator` enthält Geschäftslogik (Aufgabenzuweisung, Konsens)
- LLM-Provider-Details leben in `runtime/drivers/llm/` — nicht hier
- Der Dispatch-Loop (Queue-Polling) lebt in `runtime/server/dispatcher.rs`

## Agenten-Rollen

| Typ | Rolle | Beschreibung |
|---|---|---|
| `RequirementsAgent` | Requirements | Anforderungsanalyse, User-Stories |
| `ArchitectureAgent` | Architecture | System-Design, API-Verträge |
| `CodingAgent` | Coding | Code-Generierung via Anthropic Claude |
| `TestingAgent` | Testing | Test-Auswertung, Coverage-Prüfung |
| `SecurityAgent` | Security | Statische Sicherheitsanalyse |
| `DeploymentAgent` | Deployment | Deployment-Vorbereitung |
| `FreeLlmAgent` | alle | Beliebige Rolle via injizierbarem `LlmDriver` |

## DevRolePlugin-Trait

```rust
#[async_trait]
pub trait DevRolePlugin: Send + Sync + 'static {
    fn role_name(&self) -> &'static str;
    async fn claim_task(&self, task: &Task) -> Option<TaskClaim>;
    async fn execute(&self, task: &Task, exec: &SandboxExecResult) -> AppResult<AgentOutput>;
    async fn review(&self, change: &CodeChange) -> AppResult<Review>;
    async fn verify(&self, build: &BuildResult) -> AppResult<bool>;
}
```

## LlmDriver-Trait

Abstraktes Interface für LLM-Aufrufe — **kein I/O in dieser Datei**.  
Implementierungen ausschließlich in `runtime/drivers/llm/`.

```rust
#[async_trait]
pub trait LlmDriver: Send + Sync + 'static {
    async fn chat(&self, system: &str, user: &str, max_tokens: u32) -> AppResult<LlmResponse>;
    fn provider_name(&self) -> &str { "unknown" }
}
```

`NullDriver` — deterministischer Mock für Tests ohne echten Provider.

## FreeLlmAgent

`FreeLlmAgent` implementiert `DevRolePlugin` für alle 6 Rollen.  
Nimmt einen `Arc<dyn LlmDriver>` — kennt **keine** HTTP-Details.

```rust
// in runtime/server:
use agents::roles::FreeLlmAgent;
use drivers::FreeProviderDriver;

let driver = Arc::new(FreeProviderDriver::new());  // I/O in runtime/drivers
for (role, agent) in FreeLlmAgent::all_roles(driver) {
    registry.register(role, agent as Arc<dyn DevRolePlugin>);
}
```

Aktivierung: mindestens einen ENV-Key setzen (`GROQ_API_KEY`, `OPENROUTER_API_KEY`, etc.)  
oder `DEVSTUDIO_USE_FREE_LLM=true` für Ollama-only.

## CodingAgent — Anthropic Claude

`CodingAgent` ruft die Anthropic Claude API auf.  
Ohne `ANTHROPIC_API_KEY` → deterministischer Mock-Fallback.

## Orchestrator

```rust
let orchestrator = Orchestrator::new(Arc::new(registry));

// Welches Plugin übernimmt die Task?
let plugin = orchestrator.select_plugin(&task).await;

// Konsens erreicht?
let ok = orchestrator.consensus_met(&task);
```
