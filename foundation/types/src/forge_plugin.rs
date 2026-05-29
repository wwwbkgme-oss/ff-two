//! Kanonische Plugin-Typen + `export_forgefabrik_plugin!` Makro.
//!
//! SYNC_CONTRACT v0.1 §6 — ABI eingefroren bei v1.
//! C-Symbole: `ff_plugin_init`, `ff_plugin_tick`, `ff_plugin_shutdown`.

/// ABI-Kontext, übergeben an `ff_plugin_init`.
/// `#[repr(C)]` garantiert stabiles ABI über cdylib-Grenze.
#[repr(C)]
pub struct FfPluginCtx {
    /// API-Versions-Aushandlung — derzeit `1`.
    pub api_version: u32,
    /// Opaker Host-Pointer (vom Host gecastet).
    pub host_ptr:    *const std::ffi::c_void,
}

unsafe impl Send for FfPluginCtx {}
unsafe impl Sync for FfPluginCtx {}

/// Erfolgscode für C-ABI-Plugin-Funktionen.
pub const FF_PLUGIN_OK:    i32 = 0;
/// Fehlercode für C-ABI-Plugin-Funktionen.
pub const FF_PLUGIN_ERROR: i32 = -1;

/// Kanonische C-ABI-Symbolnamen — eingefroren, nie umbenennen.
pub mod abi {
    pub const INIT:        &str = "ff_plugin_init";
    pub const TICK:        &str = "ff_plugin_tick";
    pub const SHUTDOWN:    &str = "ff_plugin_shutdown";
    pub const API_VERSION: u32  = 1;
}

/// Generiert die drei kanonischen C-ABI-Exports für ein cdylib-Plugin.
///
/// ```rust,ignore
/// use types::export_forgefabrik_plugin;
/// export_forgefabrik_plugin!(MyPlugin, MyPlugin::default());
/// ```
///
/// Exportiert:
/// - `ff_plugin_init(*const FfPluginCtx) -> i32`
/// - `ff_plugin_tick(u64) -> i32`
/// - `ff_plugin_shutdown() -> i32`
#[macro_export]
macro_rules! export_forgefabrik_plugin {
    ($plugin_ty:ty, $constructor:expr) => {
        static mut __FF_PLUGIN_INSTANCE: Option<$plugin_ty> = None;

        #[no_mangle]
        pub unsafe extern "C" fn ff_plugin_init(
            ctx: *const $crate::forge_plugin::FfPluginCtx,
        ) -> i32 {
            let mut p = $constructor;
            match p.init(unsafe { &*ctx }) {
                Ok(())  => { unsafe { __FF_PLUGIN_INSTANCE = Some(p) }; $crate::forge_plugin::FF_PLUGIN_OK }
                Err(_)  => $crate::forge_plugin::FF_PLUGIN_ERROR,
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn ff_plugin_tick(tick: u64) -> i32 {
            match unsafe { __FF_PLUGIN_INSTANCE.as_mut() } {
                Some(p) => match p.tick(tick) {
                    Ok(())  => $crate::forge_plugin::FF_PLUGIN_OK,
                    Err(_)  => $crate::forge_plugin::FF_PLUGIN_ERROR,
                },
                None => $crate::forge_plugin::FF_PLUGIN_ERROR,
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn ff_plugin_shutdown() -> i32 {
            match unsafe { __FF_PLUGIN_INSTANCE.take() } {
                Some(mut p) => match p.shutdown() {
                    Ok(())  => $crate::forge_plugin::FF_PLUGIN_OK,
                    Err(_)  => $crate::forge_plugin::FF_PLUGIN_ERROR,
                },
                None => $crate::forge_plugin::FF_PLUGIN_OK,
            }
        }
    };
}
