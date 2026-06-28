//! Application entry point for the Prose core. `run()` builds the concrete
//! adapters, wires them into the domain services, and registers the Tauri
//! commands and the `prose://` protocol.

pub mod adapters;
pub mod domain;
pub mod ipc;
pub mod protocol;
pub mod state;

use crate::adapters::memory::{InMemoryRemoteStore, WallClock};
use crate::adapters::storage::SqliteBookRepository;
use crate::domain::{
    AnnotationService, LibraryService, ReaderRegistry, ReadingService, SettingsService, SyncService,
};
use crate::state::AppState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            let app_data = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data).expect("failed to create app data dir");
            std::fs::create_dir_all(app_data.join("books")).expect("failed to create books dir");
            std::fs::create_dir_all(app_data.join("covers")).expect("failed to create covers dir");
            let db_path = app_data.join("prose.db");

            let repo =
                Arc::new(SqliteBookRepository::new(db_path).expect("failed to open database"));
            let remote = Arc::new(InMemoryRemoteStore::new());
            let clock = Arc::new(WallClock);
            let readers = ReaderRegistry::new(vec![
                Arc::new(crate::adapters::readers::epub::EpubReader::new(
                    app_data.clone(),
                )),
                Arc::new(crate::adapters::readers::pdf::PdfReader::new(
                    app_data.clone(),
                )),
            ]);

            let app_state = AppState {
                settings: SettingsService::new(
                    Arc::clone(&repo) as Arc<dyn domain::ports::BookRepository>
                ),
                library: LibraryService::new(
                    Arc::clone(&repo) as Arc<dyn domain::ports::BookRepository>,
                    readers,
                ),
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

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::settings::settings_get,
            ipc::settings::settings_patch,
            ipc::library::library_import_book,
            ipc::library::library_list,
            ipc::library::library_remove,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
