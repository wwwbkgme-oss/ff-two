use std::collections::HashMap;

use chrono::Utc;
use parking_lot::RwLock;
use tokio::sync::broadcast;
use uuid::Uuid;

use events::WorldEvent;
use types::{BiomeType, Chunk, VoxelBlock, VoxelKind, WorldSnapshot};

use crate::visualizer::voxel_kind_for_ext;

/// Zentraler Weltzustand — Projektion aus allen bisher aufgetretenen WorldEvents.
///
/// ## BKG: Single Mutation Path
/// Alle Änderungen laufen über `place_block()` / `remove_block()`, die
/// jeweils ein `WorldEvent` emittieren. Kein direkter Zugriff auf `chunks`.
pub struct WorldState {
    chunks: RwLock<HashMap<String, Chunk>>,
    tx:     broadcast::Sender<WorldEvent>,
}

impl std::fmt::Debug for WorldState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldState")
            .field("chunks", &self.chunks.read().len())
            .finish()
    }
}

impl WorldState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { chunks: RwLock::new(HashMap::new()), tx }
    }

    // ── Event-Subscription ────────────────────────────────────────────────

    /// Gibt einen Receiver für Echtzeit-WorldEvents zurück.
    /// `runtime/api` verwendet diesen für SSE-Streams.
    pub fn subscribe(&self) -> broadcast::Receiver<WorldEvent> {
        self.tx.subscribe()
    }

    // ── Mutationen (Single Mutation Path) ─────────────────────────────────

    /// Block platzieren und `BlockPlaced`-Event emittieren.
    pub fn place_block(&self, block: VoxelBlock) {
        let key = chunk_key(block.x >> 4, 0, block.z >> 4);
        {
            let mut chunks = self.chunks.write();
            let chunk = chunks
                .entry(key)
                .or_insert_with(|| Chunk::new(block.x >> 4, 0, block.z >> 4, block.biome.clone()));
            chunk.blocks.retain(|b| !(b.x == block.x && b.y == block.y && b.z == block.z));
            chunk.blocks.push(block.clone());
        }
        let _ = self.tx.send(WorldEvent::BlockPlaced { block });
    }

    /// Block entfernen und `BlockRemoved`-Event emittieren.
    pub fn remove_block(&self, x: i32, y: i32, z: i32) {
        let key = chunk_key(x >> 4, 0, z >> 4);
        {
            let mut chunks = self.chunks.write();
            if let Some(c) = chunks.get_mut(&key) {
                c.blocks.retain(|b| !(b.x == x && b.y == y && b.z == z));
            }
        }
        let _ = self.tx.send(WorldEvent::BlockRemoved { x, y, z });
    }

    // ── Projektionen (Read-Only) ───────────────────────────────────────────

    /// Alle Chunks als Snapshot.
    pub fn all_chunks(&self) -> Vec<Chunk> {
        self.chunks.read().values().cloned().collect()
    }

    /// Einzelner Chunk per Chunk-Koordinaten.
    pub fn get_chunk(&self, cx: i32, cy: i32, cz: i32) -> Option<Chunk> {
        self.chunks.read().get(&chunk_key(cx, cy, cz)).cloned()
    }

    /// Deterministischer Snapshot des aktuellen Zustands.
    pub fn snapshot(&self, project_id: Uuid, message: impl Into<String>) -> WorldSnapshot {
        let chunks: Vec<Chunk> = self.all_chunks();
        let block_count = chunks.iter().map(|c| c.blocks.len()).sum();
        let snap = WorldSnapshot {
            id:          Uuid::new_v4(),
            project_id,
            block_count,
            chunks,
            message:     message.into(),
            created_at:  Utc::now(),
        };
        let _ = self.tx.send(WorldEvent::SnapshotCreated { snapshot: snap.clone() });
        snap
    }

    // ── Visualisierung ────────────────────────────────────────────────────

    /// Mappe eine Quelldatei auf einen Voxel-Block und platziere ihn.
    pub fn visualise_file(&self, path: &str, lines: u32) {
        use std::path::Path;
        let ext   = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("");
        let biome = BiomeType::from_extension(ext);
        let hash: u64 = path.bytes().fold(5381u64, |a, b| a.wrapping_mul(33).wrapping_add(b as u64));
        let block = VoxelBlock {
            x: (hash & 0xFF) as i32,
            y: 0,
            z: ((hash >> 8) & 0xFF) as i32,
            kind:       voxel_kind_for_ext(ext),
            biome,
            label:      std::path::Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or(path).to_string(),
            source:     path.to_string(),
            line:       1,
            size:       lines,
            covered:    false,
            updated_at: Utc::now(),
        };
        let count = 1usize;
        self.place_block(block);
        let _ = self.tx.send(WorldEvent::FileVisualized { path: path.to_string(), block_count: count });
    }
}

impl Default for WorldState { fn default() -> Self { Self::new() } }

fn chunk_key(cx: i32, cy: i32, cz: i32) -> String { format!("{cx},{cy},{cz}") }
