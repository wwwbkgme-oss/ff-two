# `domain/world`

Voxel-Weltzustand — Projektion aus allen aufgetretenen WorldEvents.

## BKG-Regeln

- Reine Domänenlogik: kein HTTP, keine DB
- Single Mutation Path: alle Änderungen über `place_block()` / `remove_block()`
- Replay-safe: gleiche Events → gleicher Zustand

## API

```rust
use world::WorldState;

let world = WorldState::new();

// Block platzieren (emittiert BlockPlaced-Event)
world.place_block(block);

// Block entfernen (emittiert BlockRemoved-Event)
world.remove_block(x, y, z);

// Datei visualisieren (Dateipfad → Voxel-Block)
world.visualise_file("src/main.rs", 120);

// Echtzeit-Events subscriben (für SSE)
let rx = world.subscribe();

// Snapshot
let snap = world.snapshot(project_id, "vor Refactoring");
```

## Voxel-Koordinaten

Chunks werden automatisch aus Block-Koordinaten berechnet: `cx = x >> 4, cz = z >> 4`.  
`visualise_file()` berechnet deterministisch (x, z) aus dem Dateipfad-Hash.

## Visualisierungs-Biome

Jede Dateiendung wird einem Biome-Typ zugeordnet (z. B. `.rs` → Rust-Biome, `.ts` → TypeScript-Biome). Dadurch entstehen in der Voxel-Welt visuell unterscheidbare Codebase-Regionen.
