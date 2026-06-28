//! The domain core: platform-independent library, reading, annotation,
//! settings, and sync logic.
//!
//! Nothing here touches Tauri, reqwest, rusqlite, or the filesystem. Everything
//! external is reached through a port (see `ports.rs` as it lands), so the core
//! compiles once and is unit-testable without a UI, disk, or network.

pub mod error;
pub mod model;

pub use error::DomainError;
pub use model::{
    Book, BookId, BookMetadata, Bookmark, Format, Highlight, Locator, Progress, ReadingStyle,
    Settings, Theme, SETTINGS_SCHEMA_VERSION,
};
