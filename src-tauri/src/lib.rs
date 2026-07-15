//! Application entry point for the Prose core. `run()` builds the concrete
//! adapters, wires them into the domain services, and registers the Tauri
//! commands and the `prose://` protocol.

pub mod adapters;
pub mod domain;
pub mod ipc;
pub mod protocol;
pub mod state;

use crate::adapters::credentials::KeyringCredentialStore;
use crate::adapters::dictionary::WordNetDictionary;
use crate::adapters::memory::{InMemoryRemoteStore, WallClock};
use crate::adapters::storage::SqliteBookRepository;
use crate::domain::ports::CredentialStore;
use crate::domain::{
    AnnotationService, DictionaryService, LibraryService, ReaderRegistry, ReadingService,
    SettingsService, SyncService,
};
use crate::state::{AppState, SyncConfig};
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .register_asynchronous_uri_scheme_protocol("prose", protocol::handle)
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

            let credentials: Arc<dyn CredentialStore> =
                Arc::new(KeyringCredentialStore::new("prose", app_data.clone()));

            // The bundled WordNet data set ships as a Tauri resource; resolve it
            // and hand the path to the dictionary, which loads it lazily.
            let wordnet_path = app
                .path()
                .resolve(
                    "resources/wordnet.json",
                    tauri::path::BaseDirectory::Resource,
                )
                .expect("failed to resolve dictionary resource path");
            let dictionary = Arc::new(DictionaryService::new(Arc::new(WordNetDictionary::new(
                wordnet_path,
            ))));

            // Restore sync configuration from keychain if previously saved.
            let sync_config = {
                let url = credentials.retrieve("prose_webdav_url").ok().flatten();
                let username = credentials.retrieve("prose_webdav_username").ok().flatten();
                SyncConfig { url, username }
            };

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
                dictionary,
                sync: SyncService::new(Arc::clone(&remote) as Arc<dyn domain::ports::RemoteStore>),
                credentials,
                sync_config: std::sync::Mutex::new(sync_config),
                sync_dirs_created: std::sync::atomic::AtomicBool::new(false),
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
            ipc::library::library_set_archived,
            ipc::reading::reading_save_position,
            ipc::reading::reading_get_position,
            ipc::reading::reading_log_session,
            ipc::reading::reading_list_sessions,
            ipc::reading::reading_delete_session,
            ipc::annotation::annotation_add_bookmark,
            ipc::annotation::annotation_list_bookmarks,
            ipc::annotation::annotation_delete_bookmark,
            ipc::annotation::annotation_add_highlight,
            ipc::annotation::annotation_list_highlights,
            ipc::annotation::annotation_delete_highlight,
            ipc::dictionary::dictionary_lookup,
            ipc::sync::sync_configure,
            ipc::sync::sync_status,
            ipc::sync::sync_disconnect,
            ipc::sync::sync_list_remote,
            ipc::sync::sync_download_book,
            ipc::sync::sync_trigger,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            handle_run_event(app_handle, event);
        });
}

/// React to Tauri runtime events. The `Opened` variant (a file handed to the app
/// to open) only exists on macOS and iOS, so the handler is compiled only there;
/// every other platform gets a no-op that keeps the run closure signature stable.
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn handle_run_event(app_handle: &tauri::AppHandle, event: tauri::RunEvent) {
    if let tauri::RunEvent::Opened { urls } = event {
        use tauri::Emitter;
        let app = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            for url in urls {
                // Desktop and iOS deliver `file://` URLs; Android's VIEW
                // intents deliver `content://` URIs that have no file
                // path. Hand the raw URI to the importer, which resolves
                // `content://` through the Android content resolver.
                let location = match url.to_file_path() {
                    Ok(path) => path.to_string_lossy().into_owned(),
                    Err(_) => url.to_string(),
                };
                if let Err(err) = ipc::library::import_book_from_path(&app, location).await {
                    // The import emits its own progress events that the
                    // library store listens to. Without a terminal event
                    // on failure the UI sticks at the last fraction (it
                    // treats anything below 1.0 as still importing), so
                    // emit a final event to clear that spinner and report
                    // the reason.
                    eprintln!("Failed to import opened file: {}", err.message);
                    let _ = app.emit(
                        ipc::event::IMPORT_PROGRESS,
                        ipc::event::ImportProgressPayload {
                            message: format!("Import failed: {}", err.message),
                            fraction: 1.0,
                        },
                    );
                }
            }
        });
    }
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn handle_run_event(_app_handle: &tauri::AppHandle, _event: tauri::RunEvent) {}
