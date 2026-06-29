//! Ports: the traits through which the domain core reaches the outside world.
//!
//! Adapters in `crate::adapters` implement these; services depend only on the
//! traits, so tests inject the in-memory fakes from `crate::domain::testing`.
//! Every method returns `Result<_, DomainError>` and every trait is
//! `Send + Sync` so the wired services can live in shared application state.

use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;
use crate::domain::model::{
    Book, BookId, BookMetadata, Bookmark, Definition, Format, Highlight, LibraryEntry, Progress,
    Settings,
};

/// The single local persistence port: catalog, reading position, annotations,
/// and settings. One atomic store sits behind it (SQLite plus the filesystem),
/// so all local data shares the same transactional guarantees.
pub trait BookRepository: Send + Sync {
    fn insert_book(&self, book: &Book) -> Result<(), DomainError>;
    fn get_book(&self, id: &BookId) -> Result<Option<Book>, DomainError>;
    /// Every book with its derived progress and last-read time, in one pass.
    fn list_entries(&self) -> Result<Vec<LibraryEntry>, DomainError>;
    fn remove_book(&self, id: &BookId) -> Result<(), DomainError>;

    fn save_progress(&self, id: &BookId, progress: &Progress) -> Result<(), DomainError>;
    fn get_progress(&self, id: &BookId) -> Result<Option<Progress>, DomainError>;

    fn add_bookmark(&self, bookmark: &Bookmark) -> Result<(), DomainError>;
    fn list_bookmarks(&self, id: &BookId) -> Result<Vec<Bookmark>, DomainError>;
    fn delete_bookmark(&self, bookmark_id: &str) -> Result<(), DomainError>;

    fn add_highlight(&self, highlight: &Highlight) -> Result<(), DomainError>;
    fn list_highlights(&self, id: &BookId) -> Result<Vec<Highlight>, DomainError>;
    fn delete_highlight(&self, highlight_id: &str) -> Result<(), DomainError>;

    fn get_settings(&self) -> Result<Option<Settings>, DomainError>;
    fn save_settings(&self, settings: &Settings) -> Result<(), DomainError>;

    fn get_sync_state(&self, key: &str) -> Result<Option<String>, DomainError>;
    fn save_sync_state(&self, key: &str, value: &str) -> Result<(), DomainError>;
    fn delete_sync_state(&self, key: &str) -> Result<(), DomainError>;

    fn get_deleted_books(&self) -> Result<Vec<String>, DomainError>;
    fn add_deleted_book(&self, id: &str) -> Result<(), DomainError>;
}

/// A single resource served to the renderer through the `prose://` protocol:
/// the bytes plus the MIME type the WebView should treat them as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceContent {
    pub bytes: Vec<u8>,
    pub mime: String,
}

/// A per-format reader: parses a book enough to extract metadata. Adding a
/// format is a new implementation of this trait plus one registry entry, with
/// no change to the domain core or the UI.
pub trait ReaderAdapter: Send + Sync {
    /// Whether this adapter handles the given format.
    fn supports(&self, format: Format) -> bool;
    /// Extract title, author, and cover from the book bytes.
    fn probe(&self, bytes: &[u8]) -> Result<BookMetadata, DomainError>;
    /// Read a single resource from a book, returning its bytes and MIME type.
    ///
    /// An empty `resource_path` means the whole book file; a non-empty one
    /// names a resource inside the container (an ePub entry). The `prose://`
    /// protocol calls this so only the reader adapter knows a format's
    /// internals. The default serves the whole file as opaque bytes, which
    /// suits a single-file format; container formats override it.
    fn read_resource(
        &self,
        bytes: &[u8],
        _resource_path: &str,
    ) -> Result<ResourceContent, DomainError> {
        Ok(ResourceContent {
            bytes: bytes.to_vec(),
            mime: "application/octet-stream".to_string(),
        })
    }
}

/// A file on the remote WebDAV server, with its current ETag for change
/// detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteEntry {
    pub path: String,
    pub etag: Option<String>,
}

/// The remote synchronization port. All network and TLS handling lives in the
/// adapter; the domain sync logic works only against this trait.
pub trait RemoteStore: Send + Sync {
    /// List the entries directly under `dir`.
    fn list(&self, dir: &str) -> Result<Vec<RemoteEntry>, DomainError>;
    fn download(&self, path: &str) -> Result<Vec<u8>, DomainError>;
    fn upload(&self, path: &str, bytes: &[u8]) -> Result<(), DomainError>;
    fn delete(&self, path: &str) -> Result<(), DomainError>;
}

/// The secure credential port, backed by the OS keychain. Secrets are never
/// stored in plaintext by the domain or the local store.
pub trait CredentialStore: Send + Sync {
    fn store(&self, key: &str, secret: &str) -> Result<(), DomainError>;
    fn retrieve(&self, key: &str) -> Result<Option<String>, DomainError>;
    fn delete(&self, key: &str) -> Result<(), DomainError>;
}

/// A source of wall-clock time, injected so timestamping is deterministic in
/// tests.
pub trait Clock: Send + Sync {
    /// The current time as epoch milliseconds.
    fn now_ms(&self) -> i64;
}

/// The offline dictionary port (FR-NOTE-03). The bundled data set and its
/// parsing live in the adapter; the domain only asks for a word's senses, so a
/// test double returns canned definitions without touching the filesystem.
pub trait Dictionary: Send + Sync {
    /// The senses of `word`, already normalized by the caller. An unknown word
    /// yields an empty vector, not an error.
    fn lookup(&self, word: &str) -> Result<Vec<Definition>, DomainError>;
}
