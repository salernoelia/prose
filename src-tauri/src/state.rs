//! `AppState`: owns the wired domain services and lives in Tauri's managed
//! state.
//!
//! Built once in `run()` with the concrete adapters injected behind their
//! ports, then shared across command invocations. The struct lands with the
//! first service wiring.

use std::sync::{Arc, Mutex};

use crate::domain::{
    ports::CredentialStore, AnnotationService, DictionaryService, LibraryService, ReadingService,
    SettingsService, SyncService,
};

/// The server URL and username that were last configured by the user. The
/// password is never held in memory beyond the sync call that needs it.
#[derive(Debug, Clone, Default)]
pub struct SyncConfig {
    pub url: Option<String>,
    pub username: Option<String>,
}

pub struct AppState {
    pub settings: SettingsService,
    pub library: LibraryService,
    pub reading: ReadingService,
    pub annotations: AnnotationService,
    /// Offline dictionary, shared so its lazily built index survives across
    /// lookups and can move onto a blocking thread for the first, heavy build.
    pub dictionary: Arc<DictionaryService>,
    pub sync: SyncService,
    pub credentials: Arc<dyn CredentialStore>,
    /// Mutable sync configuration (URL + username). Password stays in keychain.
    pub sync_config: Mutex<SyncConfig>,
    /// Cached flag to avoid redundant MKCOL folder checks on subsequent sync runs
    pub sync_dirs_created: std::sync::atomic::AtomicBool,
}
