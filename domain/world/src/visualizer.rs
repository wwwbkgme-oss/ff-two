use types::VoxelKind;

/// Leite den VoxelKind aus der Dateiendung ab.
pub fn voxel_kind_for_ext(ext: &str) -> VoxelKind {
    match ext {
        "rs" | "go" | "py" | "ts" | "js" | "java" | "cs" => VoxelKind::Module,
        "toml" | "yaml" | "yml" | "json" | "env"          => VoxelKind::Config,
        "md" | "txt" | "adoc"                              => VoxelKind::Documentation,
        "sh" | "makefile" | "dockerfile"                   => VoxelKind::BuildScript,
        _                                                  => VoxelKind::Unknown,
    }
}
