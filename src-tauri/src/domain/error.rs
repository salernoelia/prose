//! The domain error type shared across the core services.

use thiserror::Error;

use crate::domain::model::Format;

/// Errors produced by the domain core. The boundary maps each one to a
/// serializable `AppError` in `ipc/error.rs`. Adapter-specific failures
/// (SQLite, WebDAV, keychain) are flattened into the message-carrying variants
/// so the underlying crate never leaks into a domain type.
#[derive(Debug, Error)]
pub enum DomainError {
    /// No book with the given id exists in the library.
    #[error("book not found: {0}")]
    BookNotFound(String),

    /// The bytes could not be parsed as the claimed format.
    #[error("invalid or unreadable book format")]
    InvalidFormat,

    /// No reader adapter is registered for the requested format.
    #[error("no reader for format {0:?}")]
    NoReaderForFormat(Format),

    /// A requested resource inside a book does not exist.
    #[error("resource not found: {0}")]
    ResourceNotFound(String),

    /// Caller-supplied input was rejected by a domain invariant.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// A synchronization conflict that could not be resolved automatically.
    #[error("sync conflict: {0}")]
    Conflict(String),

    /// The local store failed (flattened from the storage adapter).
    #[error("storage error: {0}")]
    Storage(String),

    /// The remote store failed (flattened from the WebDAV adapter).
    #[error("remote error: {0}")]
    Remote(String),

    /// The OS credential store failed (flattened from the keychain adapter).
    #[error("credential error: {0}")]
    Credential(String),

    /// The bundled dictionary data set could not be read or parsed.
    #[error("dictionary error: {0}")]
    Dictionary(String),
}
