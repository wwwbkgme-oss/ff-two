# `runtime/store`

`Store`-Trait und `MemoryStore` — abstraktes Storage-Interface.

## Store-Trait

```rust
#[async_trait]
pub trait Store: Send + Sync + 'static {
    // Projects
    async fn create_project(&self, p: Project)  -> AppResult<Project>;
    async fn get_project(&self, id: Uuid)        -> AppResult<Project>;
    async fn list_projects(&self)                -> AppResult<Vec<Project>>;
    async fn update_project(&self, p: Project)   -> AppResult<Project>;
    async fn delete_project(&self, id: Uuid)     -> AppResult<()>;

    // Tasks, Agents, Sandboxes, Deployments, World Snapshots ...
}
```

Vollständige Trait-Definition: [`src/store.rs`](src/store.rs).

## MemoryStore

Thread-sicherer In-Memory-Store auf Basis von `DashMap`.

```rust
use store::MemoryStore;

let store = Arc::new(MemoryStore::new());
```

Geeignet für Entwicklung und Integration-Tests. Für Produktion: `PostgresStore` (geplant).

## Fehlerverhalten

Nicht gefundene Ressourcen geben `AppError::NotFound(...)` zurück.  
Konflikte (z. B. `update` auf nicht existierende ID) geben `AppError::NotFound` zurück.
