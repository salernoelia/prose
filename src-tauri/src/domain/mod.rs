//! The domain core: platform-independent library, reading, annotation,
//! settings, and sync logic.
//!
//! Nothing here touches Tauri, reqwest, rusqlite, or the filesystem. Everything
//! external is reached through a port in `ports.rs`, so the core compiles once
//! and is unit-testable without a UI, disk, or network.

pub mod annotation;
pub mod dictionary;
pub mod error;
pub mod library;
pub mod model;
pub mod ports;
pub mod reading;
pub mod settings;
pub mod sync;

#[cfg(test)]
pub mod testing;

pub use annotation::AnnotationService;
pub use dictionary::DictionaryService;
pub use error::DomainError;
pub use library::{LibraryService, ReaderRegistry};
pub use model::{
    Book, BookId, BookMetadata, Bookmark, Definition, Format, Highlight, LibraryEntry,
    LibraryQuery, Locator, Progress, ReadingStyle, Settings, SortKey, Theme,
    SETTINGS_SCHEMA_VERSION,
};
pub use ports::{
    BookRepository, Clock, CredentialStore, Dictionary, ReaderAdapter, RemoteEntry, RemoteStore,
    ResourceContent,
};
pub use reading::ReadingService;
pub use settings::{ReadingStylePatch, SettingsPatch, SettingsService};
pub use sync::{merge_by_id, resolve_last_write, resolve_progress, Outbox, SyncOp, SyncService};
