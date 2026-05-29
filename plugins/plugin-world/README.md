# `plugins/plugin-world`

**Plugin-ID:** `forgefabrik.world`  
**Crate-Name:** `world-runtime` *(domain/world belegt "world")*  
**Typ:** `cdylib` — runtime-loadable Dynamic Library

## BKG-Konventionen

| Ebene | Wert |
|---|---|
| Ordner | `plugins/plugin-world` |
| Crate-Name | `world-runtime` |
| Plugin-ID | `forgefabrik.world` |

## C-ABI Exports

```c
// Metadaten des Plugins
const PluginInfo* plugin_info();
const char*       plugin_id();

// Voxel-Logik
uint8_t voxel_kind_for_ext(const char* ext);   // Dateiendung → Block-Typ (0–6)
void    coords_for_path(const char* path, int32_t* x, int32_t* z);
```

## Block-Typen

| Index | Typ | Dateiendungen |
|---|---|---|
| 0 | Unbekannt | alle anderen |
| 1 | Rust | `.rs` |
| 2 | TypeScript | `.ts`, `.tsx`, `.js`, `.jsx` |
| 3 | Python | `.py` |
| 4 | Markdown | `.md` |
| 5 | Config | `.toml`, `.yaml`, `.yml`, `.json` |
| 6 | Shell | `.sh`, `.bash`, `.zsh` |

## Manifest

Capabilities in `plugin.toml`:
- `voxel_world`, `chunk_streaming`, `world_snapshots`, `sse_events`
