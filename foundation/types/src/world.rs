use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Hinweis: WorldEvent lebt in foundation/events (BKG: Events = eigenes Crate).
// WorldState lebt in domain/world (BKG: Fachlogik = domain).

/// Technologie-Stack-Biom einer Weltregion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BiomeType {
    RustDesert, JavaScriptForest, PythonJungle, GoTundra,
    JavaMountain, TypeScriptOcean, CSharpPlains, Unknown,
}
impl BiomeType {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs"                  => Self::RustDesert,
            "js" | "jsx" | "mjs" => Self::JavaScriptForest,
            "py"                  => Self::PythonJungle,
            "go"                  => Self::GoTundra,
            "java"                => Self::JavaMountain,
            "ts" | "tsx"          => Self::TypeScriptOcean,
            "cs"                  => Self::CSharpPlains,
            _                     => Self::Unknown,
        }
    }
}

/// Semantische Klassifikation eines Voxel-Blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VoxelKind {
    Module, Function, Type, Endpoint, TestSuite,
    Config, BuildScript, Documentation, Unknown,
}

/// Ein einzelner Voxel-Block — repräsentiert eine Code-Entität an einer 3D-Position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelBlock {
    pub x: i32, pub y: i32, pub z: i32,
    pub kind:       VoxelKind,
    pub biome:      BiomeType,
    pub label:      String,
    pub source:     String,
    pub line:       u32,
    pub size:       u32,
    pub covered:    bool,
    pub updated_at: DateTime<Utc>,
}

/// 16×16×16 Chunk (analog zu einem Git-Sub-Tree).
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

/// Unveränderlicher Snapshot des gesamten Weltzustands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub id:          Uuid,
    pub project_id:  Uuid,
    pub chunks:      Vec<Chunk>,
    pub block_count: usize,
    pub message:     String,
    pub created_at:  DateTime<Utc>,
}
