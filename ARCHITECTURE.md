# ForgeFabrik DevStudio — Architecture Spec (freeze-ready)

## 1. Schichtenmodell

```
foundation/   →  Contracts, Typen, Fehler (kein I/O, keine Logik)
domain/       →  Fachdomänen, Traits, pure Logik (kein I/O)
runtime/      →  I/O, HTTP, DB, Prozesse, Infrastruktur
plugins/      →  Domain-Behavior-Erweiterungen (kein I/O)
infra/        →  Cloud-Provisionierung (Pulumi IaC)
```

**Dependency-Richtung — niemals umkehren:**

```
foundation  ←  domain  ←  runtime
                             ↑
                           plugins (dürfen runtime + domain + foundation kennen)
```

---

## 2. Plugin vs. Driver — die entscheidende Grenze

### Plugin = Domain-Behavior-Erweiterung

Ein Plugin erweitert **was das System tut**, nicht **wie es kommuniziert**.

```
✔ Plugin darf:
  - DevRolePlugin-Trait implementieren
  - Domain-Logik kapseln (GM-Regeln, Ökonomie, Welt-Visualisierung)
  - C-ABI exportieren für dynamisches Laden

✘ Plugin darf nicht:
  - HTTP-Calls machen
  - API-Keys lesen
  - externe Dienste anbinden
  - Infrastruktur steuern
```

**Erkennungsregel:** Wenn eine Komponente `reqwest`, `sqlx`, `bollard` oder `std::process` importiert, gehört sie in `runtime/`, nicht in `plugins/`.

---

### Driver = Infrastruktur-Adapter

Ein Driver implementiert **wie das System mit der Außenwelt kommuniziert**.

```
✔ Driver darf:
  - HTTP-Calls machen (LLM-APIs, Webhooks)
  - Trait-Implementierungen aus domain/ verwenden
  - Umgebungsvariablen lesen (API-Keys)
  - externe Dienste anbinden

✘ Driver darf nicht:
  - Domain-Logik enthalten
  - direkt in runtime/server oder runtime/api importiert werden
    ohne durch eine domain-seitige Abstraktion zu gehen
```

---

## 3. LLM-Layer — konkrete Einordnung

### Falsch (bisheriger Stand):

```
plugins/plugin-llm-free/   ← HTTP + API-Keys in Plugin-Layer = FALSCH
```

### Richtig:

```
domain/agents/
└── llm_driver.rs          ← LlmDriver-Trait (pure interface, kein I/O)
└── roles/free_llm.rs      ← FreeLlmAgent (nimmt Arc<dyn LlmDriver>)

runtime/drivers/
└── llm/
    ├── mod.rs              ← FreeProviderDriver: impl LlmDriver
    ├── client.rs           ← generischer OpenAI-HTTP-Client
    ├── router.rs           ← Failover-Router
    ├── types.rs            ← ChatMessage, LlmError, ProviderConfig
    └── providers/
        ├── openrouter.rs
        ├── groq.rs
        ├── cerebras.rs
        ├── sambanova.rs
        ├── mistral.rs
        ├── gemini.rs
        ├── llm7.rs
        └── ollama.rs

runtime/server/
└── main.rs                 ← verdrahtet FreeProviderDriver → FreeLlmAgent
```

---

## 4. AgentKind / Provider-Gruppierung

Provider **nicht** flach in AgentKind einbauen:

```rust
// ✘ FALSCH — Provider-Explosion
enum AgentKind { Groq, Ollama, Cerebras, OpenRouter, ... }

// ✔ RICHTIG — semantische Gruppierung
enum AgentKind {
    Anthropic,
    Free(FreeProvider),
    Local,
}

enum FreeProvider {
    Groq,
    OpenRouter,
    Cerebras,
    SambaNova,
    Mistral,
    Gemini,
    Llm7,
    Ollama,
}
```

`AgentKind` beschreibt **Semantik** (wer ist der Agent?), nicht **Transport** (welcher HTTP-Endpunkt?).

---

## 5. Plugin-Namespace-Konvention

| Namespace | Bedeutung |
|---|---|
| `forgefabrik.world` | Voxel-Welt-Behavior |
| `forgefabrik.agents` | Agenten-Verhaltens-Erweiterung |
| `forgefabrik.gm` | Game-Master-Regelwerk |
| `forgefabrik.economy` | Token-Budget, Rate-Limiting |
| `forgefabrik.community.*` | Community-Plugins (Marketplace) |

**LLM-Provider gehören NICHT in diesen Namespace** — sie sind Infrastruktur, kein Behavior.

---

## 6. Neue Infra-Crates (runtime/)

Wenn neue externe Dienste angebunden werden:

| Dienst | Crate | Aktivierung |
|---|---|---|
| LLM-Anbieter (free) | `runtime/drivers/llm` | ENV-Keys |
| PostgreSQL | `runtime/store` (`postgres.rs`) | `DATABASE_URL=postgres://...` |
| Redis | `runtime/queue` (`redis.rs`, geplant) | `REDIS_URL=redis://...` |
| Docker | `runtime/sandbox` (`docker.rs`, geplant) | `SANDBOX_USE_DOCKER=true` |

**Pattern immer gleich:**
1. Trait in `domain/` oder `foundation/` definieren (kein I/O)
2. Implementierung in `runtime/` (I/O erlaubt)
3. Auto-Select in `runtime/server/main.rs` anhand ENV

---

## 7. Verbotene Muster

```rust
// ✘ domain-Crate importiert reqwest
use reqwest::Client;  // in domain/agents → VERBOTEN (neu)

// ✘ Plugin macht HTTP-Call
// in plugins/plugin-llm-free → VERBOTEN (wurde refactored)

// ✘ foundation kennt domain
use agents::Orchestrator;  // in foundation/types → VERBOTEN

// ✔ Domain definiert Trait, Runtime implementiert
// domain/agents: pub trait LlmDriver { async fn chat(...) }
// runtime/drivers: impl LlmDriver for FreeProviderDriver { ... }
```

---

## 8. Entscheidungsbaum: Wo gehört eine neue Komponente hin?

```
Neu hinzufügen?
│
├── Ist es ein Datentyp / Fehler?          → foundation/
│
├── Ist es Fachlogik (kein I/O)?           → domain/
│   └── Erweitert es ein Verhalten?        → plugins/ (mit Trait aus domain/)
│
├── Macht es I/O (HTTP, DB, Prozess)?      → runtime/
│   ├── Ist es ein Protokoll-Adapter?      → runtime/drivers/
│   └── Ist es Server/API-Infrastruktur?   → runtime/server oder runtime/api
│
└── Ist es Cloud-Provisionierung?          → infra/
```
