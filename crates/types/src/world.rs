use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Technology-stack biome for a region of the world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BiomeType {
    RustDesert, JavaScriptForest, PythonJungle, GoTundra,
    JavaMountain, TypeScriptOcean, CSharpPlains, Unknown,
}

impl BiomeType {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs"                 => Self::RustDesert,
            "js" | "jsx" | "mjs" => Self::JavaScriptForest,
            "py"                 => Self::PythonJungle,
            "go"                 => Self::GoTundra,
            "java"               => Self::JavaMountain,
            "ts" | "tsx"         => Self::TypeScriptOcean,
            "cs"                 => Self::CSharpPlains,
            _                    => Self::Unknown,
        }
    }
}

/// Semantic classification of a voxel block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoxelKind {
    Module, Function, Type, Endpoint, TestSuite,
    Config, BuildScript, Documentation, Unknown,
}

/// One voxel block — represents a code entity at a 3-D world position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelBlock {
    pub x: i32, pub y: i32, pub z: i32,
    pub kind:       VoxelKind,
    pub biome:      BiomeType,
    /// Human-readable label (function name, file name, …).
    pub label:      String,
    /// Source file path relative to the project root.
    pub source:     String,
    pub line:       u32,
    /// Number of source lines spanned by this entity.
    pub size:       u32,
    /// Whether the entity is covered by tests.
    pub covered:    bool,
    pub updated_at: DateTime<Utc>,
}

/// A 16 × 16 × 16 chunk of voxel blocks (analogous to a git sub-tree).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub cx: i32, pub cy: i32, pub cz: i32,
    pub biome:  BiomeType,
    pub blocks: Vec<VoxelBlock>,
}

impl Chunk {
    pub const SIZE: i32 = 16;
    pub fn new(cx: i32, cy: i32, cz: i32, biome: BiomeType) -> Self {
        Self { cx, cy, cz, biome, blocks: Vec::new() }
    }
}

/// Immutable snapshot of the whole world at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub id:          Uuid,
    pub project_id:  Uuid,
    pub chunks:      Vec<Chunk>,
    pub block_count: usize,
    pub message:     String,
    pub created_at:  DateTime<Utc>,
}

/// Real-time world event streamed to clients via SSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum WorldEvent {
    BlockPlaced  { block: VoxelBlock },
    BlockRemoved { x: i32, y: i32, z: i32 },
    ChunkLoaded  { chunk: Chunk },
    Snapshot     { snapshot_id: Uuid, message: String },
}
