# `domain/agents`

`DevRolePlugin`-Trait, 6 Agenten-Rollen und der `Orchestrator`.

## BKG-Regeln

- Reine Domänenlogik: kein HTTP, kein I/O, keine Store/Queue-Deps
- Der `Orchestrator` enthält Geschäftslogik (Aufgabenzuweisung, Konsens)
- Der async Dispatch-Loop (Queue-Polling, Store-Persistenz) lebt in `runtime/server`

## Agenten-Rollen

| Rolle | Crate-Typ | Beschreibung |
|---|---|---|
| `Requirements` | `RequirementsAgent` | Anforderungsanalyse, User-Stories |
| `Architecture` | `ArchitectureAgent` | System-Design, API-Verträge |
| `Coding` | `CodingAgent` | Code-Generierung via Anthropic Claude |
| `Testing` | `TestingAgent` | Test-Auswertung, Coverage-Prüfung |
| `Security` | `SecurityAgent` | Statische Sicherheitsanalyse |
| `Deployment` | `DeploymentAgent` | Deployment-Vorbereitung |

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

## Orchestrator

```rust
let registry = Arc::new(AgentRegistry::new());
// registry.register(role, plugin) ...
let orchestrator = Orchestrator::new(registry);

// Welches Plugin übernimmt die Task?
let plugin = orchestrator.select_plugin(&task).await;

// Ist Konsens erreicht?
let ok = orchestrator.consensus_met(&task);
```

## CodingAgent — Anthropic Claude

`CodingAgent` ruft die Anthropic Claude API auf. Ohne `ANTHROPIC_API_KEY` oder bei leerem Key fällt er auf einen deterministischen Mock zurück (gibt Platzhalter-Code zurück).
