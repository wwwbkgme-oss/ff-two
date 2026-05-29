# `plugins/plugin-world`

**Plugin-ID:** `forgefabrik.world`  
**Crate-Name:** `world-runtime` *(domain/world belegt "world")*  
**Typ:** `cdylib` — runtime-loadable Dynamic Library

## BKG-Konventionen (SYNC_CONTRACT §6)

| Ebene | Wert |
|---|---|
| Ordner | `plugins/plugin-world` |
| Crate-Name | `world-runtime` |
| Plugin-ID | `forgefabrik.world` |

## Kanonische Lifecycle-ABI (SYNC_CONTRACT §6)

```c
// Kanonische Lifecycle-Hooks (ABI v1 — eingefroren)
int32_t ff_plugin_init(const FfPluginCtx* ctx);   // Plugin laden
int32_t ff_plugin_tick(uint64_t tick);             // pro World-Tick
int32_t ff_plugin_shutdown();                      // beim Herunterfahren
```

Implementiert via `types::export_forgefabrik_plugin!` Makro.

## Domain-spezifische C-ABI Exports

```c
// Metadaten (legacy, bleibt für Kompatibilität)
const PluginInfo* plugin_info();
const char*       plugin_id();

// Voxel-Logik
uint8_t voxel_kind_for_ext(const char* ext);          // Dateiendung → Block-Typ (0–6)
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

## Manifest (`plugin.toml`)

```toml
[plugin]
id          = "forgefabrik.world"
version     = "0.1.0"
name        = "ForgeFabrik World Plugin"
description = "Voxel-Welt-Behavior: Koordinaten, Block-Typen, C-ABI"

[capabilities]
provides = ["world"]
requires = []

[entry]
lib = "libworld_runtime.so"
```
