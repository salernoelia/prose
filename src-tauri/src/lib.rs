//! Application entry point for the Prose core. `run()` builds the concrete
//! adapters, wires them into the domain services, and registers the Tauri
//! commands and the `prose://` protocol.

pub mod adapters;
pub mod domain;
pub mod ipc;
pub mod protocol;
pub mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
