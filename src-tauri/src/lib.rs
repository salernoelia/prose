//! Application entry point for the Prose core. `run()` builds the concrete
//! adapters, wires them into the domain services, and registers the Tauri
//! commands and the `prose://` protocol.

pub mod adapters;
pub mod domain;
pub mod ipc;
pub mod protocol;
pub mod state;

use std::sync::Arc;
use crate::adapters::memory::{InMemoryBookRepository, InMemoryRemoteStore, WallClock};
use crate::domain::{
    AnnotationService, LibraryService, ReaderRegistry, ReadingService, SettingsService,
    SyncService,
};
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let repo = Arc::new(InMemoryBookRepository::new());
    let remote = Arc::new(InMemoryRemoteStore::new());
    let clock = Arc::new(WallClock);
    let readers = ReaderRegistry::default();

    let app_state = AppState {
        settings: SettingsService::new(Arc::clone(&repo) as Arc<dyn domain::ports::BookRepository>),
        library: LibraryService::new(Arc::clone(&repo) as Arc<dyn domain::ports::BookRepository>, readers),
        reading: ReadingService::new(
            Arc::clone(&repo) as Arc<dyn domain::ports::BookRepository>,
            Arc::clone(&clock) as Arc<dyn domain::ports::Clock>,
        ),
        annotations: AnnotationService::new(
            Arc::clone(&repo) as Arc<dyn domain::ports::BookRepository>,
            Arc::clone(&clock) as Arc<dyn domain::ports::Clock>,
        ),
        sync: SyncService::new(Arc::clone(&remote) as Arc<dyn domain::ports::RemoteStore>),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            ipc::settings::settings_get,
            ipc::settings::settings_patch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
