//! The domain core: platform-independent library, reading, annotation,
//! settings, and sync logic.
//!
//! Nothing here touches Tauri, reqwest, rusqlite, or the filesystem. Everything
//! external is reached through a port in `ports.rs`, so the core compiles once
//! and is unit-testable without a UI, disk, or network.

pub mod error;
pub mod model;
pub mod ports;

#[cfg(test)]
pub mod testing;

pub use error::DomainError;
pub use model::{
    Book, BookId, BookMetadata, Bookmark, Format, Highlight, LibraryEntry, LibraryQuery, Locator,
    Progress, ReadingStyle, Settings, SortKey, Theme, SETTINGS_SCHEMA_VERSION,
};
pub use ports::{BookRepository, Clock, CredentialStore, ReaderAdapter, RemoteEntry, RemoteStore};
