//! `AppState`: owns the wired domain services and lives in Tauri's managed
//! state.
//!
//! Built once in `run()` with the concrete adapters injected behind their
//! ports, then shared across command invocations. The struct lands with the
//! first service wiring.
