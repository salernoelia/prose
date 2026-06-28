//! `AppState`: owns the wired domain services and lives in Tauri's managed
//! state.
//!
//! Built once in `run()` with the concrete adapters injected behind their
//! ports, then shared across command invocations. The struct lands with the
//! first service wiring.

use crate::domain::{
    AnnotationService, LibraryService, ReadingService, SettingsService, SyncService,
};

pub struct AppState {
    pub settings: SettingsService,
    pub library: LibraryService,
    pub reading: ReadingService,
    pub annotations: AnnotationService,
    pub sync: SyncService,
}
