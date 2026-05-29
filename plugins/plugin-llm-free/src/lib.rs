//! `forgefabrik.llm-free` — Free LLM Router Plugin.
//!
//! ## Verwendung als rlib (statisches Linken in runtime/server)
//! ```rust
//! use llm_free::agent::{FreeLlmAgent, all_roles};
//! use llm_free::router::chat;
//! ```
//!
//! ## Verwendung als cdylib (dynamisches Laden durch Plugin-Host)
//! ```c
//! // Laden mit libloading o.Ä.:
//! const PluginInfo* info = plugin_info();
//! const char* result     = chat_request_json("{\"system\":\"...\",\"user\":\"...\"}");
//! free_string(result);
//! ```
//!
//! ## Kostenlose Provider (nur echte Free-Tiers, kein Ablaufdatum):
//! 1. OpenRouter `:free` Modelle  — OPENROUTER_API_KEY
//! 2. Groq Free Tier              — GROQ_API_KEY
//! 3. Cerebras Free Tier          — CEREBRAS_API_KEY
//! 4. SambaNova Forever-Free      — SAMBANOVA_API_KEY
//! 5. LLM7.io Gateway             — LLM7_API_KEY
//! 6. Ollama (lokal)              — kein Key nötig
//!
//! Ported + rebranded von github.com/apmantza/pi-free (MIT)

#![allow(clippy::missing_safety_doc)]

use std::os::raw::c_char;
use std::sync::OnceLock;

pub mod agent;
pub mod client;
pub mod providers;
pub mod router;
pub mod types;

// ── Statischer Tokio-Runtime für C-ABI Blocking-Calls ────────────────────────

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("llm-free: Tokio-Runtime konnte nicht initialisiert werden")
    })
}

// ── Stabile C-ABI Metadaten ───────────────────────────────────────────────────

static PLUGIN_ID:      &[u8] = b"forgefabrik.llm-free\0";
static PLUGIN_VERSION: &[u8] = b"0.1.0\0";
static PLUGIN_KIND:    &[u8] = b"runtime-plugin\0";

#[repr(C)]
pub struct PluginInfo {
    pub id:      *const c_char,
    pub version: *const c_char,
    pub kind:    *const c_char,
}

unsafe impl Send for PluginInfo {}
unsafe impl Sync for PluginInfo {}

/// Gibt statische Metadaten dieses Plugins zurück.
#[no_mangle]
pub extern "C" fn plugin_info() -> *const PluginInfo {
    static INFO: PluginInfo = PluginInfo {
        id:      PLUGIN_ID.as_ptr()      as *const c_char,
        version: PLUGIN_VERSION.as_ptr() as *const c_char,
        kind:    PLUGIN_KIND.as_ptr()    as *const c_char,
    };
    &INFO
}

/// Null-terminierter Plugin-ID-String.
#[no_mangle]
pub extern "C" fn plugin_id() -> *const c_char {
    PLUGIN_ID.as_ptr() as *const c_char
}

// ── C-ABI Chat-Interface ──────────────────────────────────────────────────────

/// Sendet eine Chat-Anfrage an den ersten verfügbaren freien Provider (blockierend).
///
/// # Parameter
/// `input_json` — null-terminierter JSON-String:
/// ```json
/// {"system": "...", "user": "...", "max_tokens": 4096}
/// ```
///
/// # Rückgabewert
/// Null-terminierter JSON-String:
/// ```json
/// {"text": "...", "provider": "Groq", "model": "llama-3.3-70b-versatile"}
/// // oder bei Fehler:
/// {"error": "all providers failed: ..."}
/// ```
///
/// Der Caller muss den zurückgegebenen Pointer mit `free_string()` freigeben.
///
/// # Safety
/// `input_json` muss ein gültiger, null-terminierter UTF-8-Zeiger sein.
#[no_mangle]
pub unsafe extern "C" fn chat_request_json(input_json: *const c_char) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        let input = unsafe {
            if input_json.is_null() { return json_error("null input"); }
            match std::ffi::CStr::from_ptr(input_json).to_str() {
                Ok(s)  => s.to_owned(),
                Err(_) => return json_error("invalid UTF-8 input"),
            }
        };

        #[derive(serde::Deserialize)]
        struct Req {
            system:     String,
            user:       String,
            max_tokens: Option<u32>,
        }

        let req: Req = match serde_json::from_str(&input) {
            Ok(r)  => r,
            Err(e) => return json_error(&format!("JSON parse: {e}")),
        };

        let rt  = get_runtime();
        let max = req.max_tokens.unwrap_or(4096);

        match rt.block_on(router::chat(req.system, req.user, max)) {
            Ok(r) => {
                let out = serde_json::json!({
                    "text":     r.text,
                    "provider": r.provider,
                    "model":    r.model,
                    "tokens":   r.tokens,
                });
                to_c_string(&out.to_string())
            }
            Err(e) => json_error(&e.to_string()),
        }
    });

    result.unwrap_or_else(|_| json_error("panic in chat_request_json"))
}

/// Listet alle verfügbaren freien Modelle als JSON-Array (blockierend).
///
/// Rückgabe: null-terminierter JSON-String `["ollama/llama3.1:8b", "openrouter/...", ...]`
/// Der Caller muss den Pointer mit `free_string()` freigeben.
#[no_mangle]
pub extern "C" fn list_free_models_json() -> *mut c_char {
    let rt     = get_runtime();
    let models = rt.block_on(router::list_all_free_models());
    let json   = serde_json::to_string(&models).unwrap_or_else(|_| "[]".into());
    to_c_string(&json)
}

/// Gibt einen mit `chat_request_json` oder `list_free_models_json` zurückgegebenen
/// String wieder frei. Darf NICHT auf anderem Speicher aufgerufen werden.
///
/// # Safety
/// `s` muss ein von dieser Library allozierter Pointer sein.
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() { return; }
    // SAFETY: Pointer wurde von `to_c_string` via `Box::into_raw` alloziert.
    let _ = unsafe { std::ffi::CString::from_raw(s) };
}

// ── Interne Helfer ────────────────────────────────────────────────────────────

fn json_error(msg: &str) -> *mut c_char {
    to_c_string(&format!("{{\"error\":\"{}\"}}", msg.replace('"', "'")))
}

fn to_c_string(s: &str) -> *mut c_char {
    // Null-Byte ersetzen, falls vorhanden (sollte nicht vorkommen)
    let safe = s.replace('\0', "");
    match std::ffi::CString::new(safe) {
        Ok(cs) => cs.into_raw(),
        Err(_) => json_error("output enthält Null-Byte"),
    }
}
