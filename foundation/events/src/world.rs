use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::{VoxelBlock, WorldSnapshot};

/// Echtzeit-World-Änderungsereignisse — werden per SSE an Clients gestreamt.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum WorldEvent {
    /// Ein neuer oder aktualisierter Block wurde platziert.
    BlockPlaced { block: VoxelBlock },
    /// Ein Block wurde entfernt.
    BlockRemoved { x: i32, y: i32, z: i32 },
    /// Ein Chunk wurde in die Welt geladen.
    ChunkLoaded { cx: i32, cy: i32, cz: i32 },
    /// Ein deterministischer Snapshot wurde erzeugt.
    SnapshotCreated { snapshot: WorldSnapshot },
    /// Alle Blöcke einer Datei wurden aktualisiert.
    FileVisualized { path: String, block_count: usize },
}
