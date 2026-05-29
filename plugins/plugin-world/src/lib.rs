//! `forgefabrik.world` — Voxel-Welt Plugin (runtime-loadable cdylib).
//!
//! ## BKG-Architektur
//! * Ordner:      `plugins/plugin-world/`
//! * Crate-Name:  `world-runtime`  (domain/world belegt "world")
//! * Plugin-ID:   `forgefabrik.world`
//!
//! Dieses Crate ist eine eigenständige Dynamic Library.
//! Der Host lädt sie zur Laufzeit und ruft `plugin_info()` auf.

use std::os::raw::c_char;

// ── Stabile C-ABI Metadaten ────────────────────────────────────────────────

/// Null-terminierte Konstanten für das C-ABI — statische Lifetime garantiert
/// gültige Zeiger für die gesamte Laufzeit der Library.
static PLUGIN_ID:      &[u8] = b"forgefabrik.world\0";
static PLUGIN_VERSION: &[u8] = b"0.1.0\0";
static PLUGIN_KIND:    &[u8] = b"runtime-plugin\0";

/// Stabile Plugin-Metadaten — `#[repr(C)]` garantiert ABI-Kompatibilität.
///
/// Der Host dereferenziert diesen Pointer direkt nach dem dynamischen Laden.
#[repr(C)]
pub struct PluginInfo {
    /// Null-terminierte Plugin-ID, z. B. `"forgefabrik.world\0"`.
    pub id:      *const c_char,
    /// Semantische Versionsnummer, null-terminiert.
    pub version: *const c_char,
    /// Plugin-Kategorie: `"runtime-plugin"`, `"utility"`, u. ä.
    pub kind:    *const c_char,
}

// SAFETY: Die Pointer zeigen auf statische, unveränderliche Byte-Arrays
// (`&[u8; N]`). Diese werden nie moved oder gedroppt.
unsafe impl Send for PluginInfo {}
unsafe impl Sync for PluginInfo {}

/// Gibt statische Metadaten dieses Plugins zurück.
///
/// # Safety
/// Der zurückgegebene Zeiger ist für die gesamte Prozesslebensdauer gültig.
/// Der Host darf ihn lesen, aber nicht schreiben oder freigeben.
#[no_mangle]
pub extern "C" fn plugin_info() -> *const PluginInfo {
    static INFO: PluginInfo = PluginInfo {
        id:      PLUGIN_ID.as_ptr()      as *const c_char,
        version: PLUGIN_VERSION.as_ptr() as *const c_char,
        kind:    PLUGIN_KIND.as_ptr()    as *const c_char,
    };
    &INFO
}

/// Null-terminierter Plugin-ID-String — Kurzform für schnelle Prüfungen.
#[no_mangle]
pub extern "C" fn plugin_id() -> *const c_char {
    PLUGIN_ID.as_ptr() as *const c_char
}

// ── Plugin-Logik ──────────────────────────────────────────────────────────

/// Repräsentiert einen einzelnen Voxel-Block im Plugin-eigenen Koordinatensystem.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct VoxelPoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Mappe Dateiendung deterministisch auf einen Voxel-Block-Typ-Index.
///
/// Der Host kann diesen Index auf sein eigenes `VoxelKind`-Enum mappen.
/// Rückgabe: 0 = Unbekannt, 1 = Rust, 2 = TypeScript, 3 = Python,
///           4 = Markdown, 5 = TOML/YAML/JSON, 6 = Shell
#[no_mangle]
pub extern "C" fn voxel_kind_for_ext(ext_ptr: *const c_char) -> u8 {
    // SAFETY: Der Caller übergibt einen gültigen, null-terminierten Zeiger.
    let ext = unsafe {
        if ext_ptr.is_null() { return 0; }
        std::ffi::CStr::from_ptr(ext_ptr)
            .to_str()
            .unwrap_or("")
    };
    match ext {
        "rs"                         => 1,
        "ts" | "tsx" | "js" | "jsx" => 2,
        "py"                         => 3,
        "md"                         => 4,
        "toml" | "yaml" | "yml" | "json" => 5,
        "sh" | "bash" | "zsh"        => 6,
        _                            => 0,
    }
}

/// Deterministische Hash-Funktion für Voxel-Koordinaten aus einem Dateipfad.
/// Gibt (x, z) im Bereich [0, 255] zurück.
#[no_mangle]
pub extern "C" fn coords_for_path(path_ptr: *const c_char, out_x: *mut i32, out_z: *mut i32) {
    // SAFETY: Caller übergibt gültige, nicht-null Zeiger.
    let path = unsafe {
        if path_ptr.is_null() || out_x.is_null() || out_z.is_null() { return; }
        std::ffi::CStr::from_ptr(path_ptr).to_str().unwrap_or("")
    };
    let hash: u64 = path
        .bytes()
        .fold(5381u64, |acc, b| acc.wrapping_mul(33).wrapping_add(b as u64));
    unsafe {
        *out_x = (hash & 0xFF) as i32;
        *out_z = ((hash >> 8) & 0xFF) as i32;
    }
}

// ── SYNC_CONTRACT v0.1 §6 — Kanonische Plugin-Lifecycle-Hooks ────────────────

/// Interner Plugin-Zustand für Lifecycle-Hooks.
struct WorldPluginLifecycle;

impl WorldPluginLifecycle {
    fn new() -> Self { Self }
    fn init(&mut self, _ctx: &types::forge_plugin::FfPluginCtx) -> Result<(), String> {
        tracing::debug!("forgefabrik.world: ff_plugin_init");
        Ok(())
    }
    fn tick(&mut self, _tick: u64) -> Result<(), String> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), String> {
        tracing::debug!("forgefabrik.world: ff_plugin_shutdown");
        Ok(())
    }
}

types::export_forgefabrik_plugin!(WorldPluginLifecycle, WorldPluginLifecycle::new());
