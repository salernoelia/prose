//! In-memory fakes for every port, used by the service unit tests.
//!
//! These let the domain be exercised with no filesystem, network, or database.
//! The module is compiled only under `cfg(test)`; `allow(dead_code)` covers
//! helpers that a given service test does not happen to use yet.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use crate::domain::error::DomainError;
use crate::domain::model::{
    ArchivedState, Book, BookId, BookMetadata, Bookmark, Format, Highlight, LibraryEntry, Progress,
    ReadingSession, Settings,
};
use crate::domain::ports::{
    BookRepository, Clock, CredentialStore, ReaderAdapter, RemoteEntry, RemoteStore,
};

/// In-memory implementation of [`BookRepository`].
#[derive(Default)]
pub struct InMemoryBookRepository {
    inner: Mutex<RepoState>,
}

#[derive(Default)]
struct RepoState {
    books: HashMap<BookId, Book>,
    progress: HashMap<BookId, Progress>,
    bookmarks: Vec<Bookmark>,
    highlights: Vec<Highlight>,
    sessions: Vec<ReadingSession>,
    settings: Option<Settings>,
    sync_state: HashMap<String, String>,
    deleted_books: Vec<String>,
    deleted_sessions: Vec<String>,
    archived: HashMap<BookId, ArchivedState>,
}

impl InMemoryBookRepository {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, RepoState> {
        self.inner.lock().expect("repository mutex poisoned")
    }
}

impl BookRepository for InMemoryBookRepository {
    fn insert_book(&self, book: &Book) -> Result<(), DomainError> {
        self.lock().books.insert(book.id.clone(), book.clone());
        Ok(())
    }

    fn get_book(&self, id: &BookId) -> Result<Option<Book>, DomainError> {
        Ok(self.lock().books.get(id).cloned())
    }

    fn list_entries(&self) -> Result<Vec<LibraryEntry>, DomainError> {
        let state = self.lock();
        Ok(state
            .books
            .values()
            .map(|book| {
                let progress = state.progress.get(&book.id);
                LibraryEntry {
                    book: book.clone(),
                    progress: progress.map(|p| p.locator.progression).unwrap_or(0.0),
                    last_read: progress.map(|p| p.updated_at),
                    archived: state.archived.get(&book.id).is_some_and(|s| s.archived),
                }
            })
            .collect())
    }

    fn remove_book(&self, id: &BookId) -> Result<(), DomainError> {
        let mut state = self.lock();
        if state.books.remove(id).is_none() {
            return Err(DomainError::BookNotFound(id.as_str().to_string()));
        }
        state.progress.remove(id);
        state.archived.remove(id);
        state.bookmarks.retain(|b| &b.book_id != id);
        state.highlights.retain(|h| &h.book_id != id);
        state.sessions.retain(|s| &s.book_id != id);
        Ok(())
    }

    fn set_archived(&self, id: &BookId, new_state: &ArchivedState) -> Result<(), DomainError> {
        let mut state = self.lock();
        if !state.books.contains_key(id) {
            return Err(DomainError::BookNotFound(id.as_str().to_string()));
        }
        state.archived.insert(id.clone(), new_state.clone());
        Ok(())
    }

    fn get_archived(&self, id: &BookId) -> Result<Option<ArchivedState>, DomainError> {
        Ok(self.lock().archived.get(id).cloned())
    }

    fn save_progress(&self, id: &BookId, progress: &Progress) -> Result<(), DomainError> {
        self.lock().progress.insert(id.clone(), progress.clone());
        Ok(())
    }

    fn get_progress(&self, id: &BookId) -> Result<Option<Progress>, DomainError> {
        Ok(self.lock().progress.get(id).cloned())
    }

    fn add_bookmark(&self, bookmark: &Bookmark) -> Result<(), DomainError> {
        self.lock().bookmarks.push(bookmark.clone());
        Ok(())
    }

    fn list_bookmarks(&self, id: &BookId) -> Result<Vec<Bookmark>, DomainError> {
        Ok(self
            .lock()
            .bookmarks
            .iter()
            .filter(|b| &b.book_id == id)
            .cloned()
            .collect())
    }

    fn delete_bookmark(&self, bookmark_id: &str) -> Result<(), DomainError> {
        self.lock().bookmarks.retain(|b| b.id != bookmark_id);
        Ok(())
    }

    fn add_highlight(&self, highlight: &Highlight) -> Result<(), DomainError> {
        self.lock().highlights.push(highlight.clone());
        Ok(())
    }

    fn list_highlights(&self, id: &BookId) -> Result<Vec<Highlight>, DomainError> {
        Ok(self
            .lock()
            .highlights
            .iter()
            .filter(|h| &h.book_id == id)
            .cloned()
            .collect())
    }

    fn delete_highlight(&self, highlight_id: &str) -> Result<(), DomainError> {
        self.lock().highlights.retain(|h| h.id != highlight_id);
        Ok(())
    }

    fn add_reading_session(&self, session: &ReadingSession) -> Result<(), DomainError> {
        let mut state = self.lock();
        if let Some(existing) = state.sessions.iter_mut().find(|s| s.id == session.id) {
            *existing = session.clone();
        } else {
            state.sessions.push(session.clone());
        }
        Ok(())
    }

    fn list_reading_sessions(&self, id: &BookId) -> Result<Vec<ReadingSession>, DomainError> {
        Ok(self
            .lock()
            .sessions
            .iter()
            .filter(|s| &s.book_id == id)
            .cloned()
            .collect())
    }

    fn list_all_reading_sessions(&self) -> Result<Vec<ReadingSession>, DomainError> {
        Ok(self.lock().sessions.clone())
    }

    fn delete_reading_session(&self, session_id: &str) -> Result<(), DomainError> {
        self.lock().sessions.retain(|s| s.id != session_id);
        Ok(())
    }

    fn get_deleted_sessions(&self) -> Result<Vec<String>, DomainError> {
        Ok(self.lock().deleted_sessions.clone())
    }

    fn add_deleted_session(&self, id: &str) -> Result<(), DomainError> {
        let mut state = self.lock();
        if !state.deleted_sessions.contains(&id.to_string()) {
            state.deleted_sessions.push(id.to_string());
        }
        Ok(())
    }

    fn remove_deleted_session(&self, id: &str) -> Result<(), DomainError> {
        self.lock().deleted_sessions.retain(|s| s != id);
        Ok(())
    }

    fn get_settings(&self) -> Result<Option<Settings>, DomainError> {
        Ok(self.lock().settings.clone())
    }

    fn save_settings(&self, settings: &Settings) -> Result<(), DomainError> {
        self.lock().settings = Some(settings.clone());
        Ok(())
    }

    fn get_sync_state(&self, key: &str) -> Result<Option<String>, DomainError> {
        Ok(self.lock().sync_state.get(key).cloned())
    }

    fn save_sync_state(&self, key: &str, value: &str) -> Result<(), DomainError> {
        self.lock()
            .sync_state
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn delete_sync_state(&self, key: &str) -> Result<(), DomainError> {
        self.lock().sync_state.remove(key);
        Ok(())
    }

    fn get_deleted_books(&self) -> Result<Vec<String>, DomainError> {
        Ok(self.lock().deleted_books.clone())
    }

    fn add_deleted_book(&self, id: &str) -> Result<(), DomainError> {
        let mut state = self.lock();
        if !state.deleted_books.contains(&id.to_string()) {
            state.deleted_books.push(id.to_string());
        }
        Ok(())
    }

    fn remove_deleted_book(&self, id: &str) -> Result<(), DomainError> {
        self.lock().deleted_books.retain(|b| b != id);
        Ok(())
    }
}

/// A reader adapter that supports one format and returns canned metadata.
pub struct FakeReader {
    format: Format,
    metadata: BookMetadata,
    probe_fails: bool,
}

impl FakeReader {
    pub fn new(format: Format, title: &str, author: Option<&str>) -> Self {
        FakeReader {
            format,
            metadata: BookMetadata {
                title: title.to_string(),
                author: author.map(str::to_string),
                cover: None,
            },
            probe_fails: false,
        }
    }

    /// A reader whose `probe` always fails, for invalid-format tests.
    pub fn failing(format: Format) -> Self {
        FakeReader {
            format,
            metadata: BookMetadata {
                title: String::new(),
                author: None,
                cover: None,
            },
            probe_fails: true,
        }
    }
}

impl ReaderAdapter for FakeReader {
    fn supports(&self, format: Format) -> bool {
        self.format == format
    }

    fn probe(&self, _bytes: &[u8]) -> Result<BookMetadata, DomainError> {
        if self.probe_fails {
            Err(DomainError::InvalidFormat)
        } else {
            Ok(self.metadata.clone())
        }
    }
}

/// In-memory [`RemoteStore`]: a flat map of path to bytes with a coarse ETag.
#[derive(Default)]
pub struct InMemoryRemoteStore {
    files: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryRemoteStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RemoteStore for InMemoryRemoteStore {
    fn list(&self, dir: &str) -> Result<Vec<RemoteEntry>, DomainError> {
        let files = self.files.lock().expect("remote mutex poisoned");
        Ok(files
            .iter()
            .filter(|(path, _)| path.starts_with(dir))
            .map(|(path, bytes)| RemoteEntry {
                path: path.clone(),
                etag: Some(format!("{}", bytes.len())),
            })
            .collect())
    }

    fn download(&self, path: &str) -> Result<Vec<u8>, DomainError> {
        self.files
            .lock()
            .expect("remote mutex poisoned")
            .get(path)
            .cloned()
            .ok_or_else(|| DomainError::ResourceNotFound(path.to_string()))
    }

    fn upload(&self, path: &str, bytes: &[u8]) -> Result<(), DomainError> {
        self.files
            .lock()
            .expect("remote mutex poisoned")
            .insert(path.to_string(), bytes.to_vec());
        Ok(())
    }

    fn delete(&self, path: &str) -> Result<(), DomainError> {
        self.files
            .lock()
            .expect("remote mutex poisoned")
            .remove(path);
        Ok(())
    }
}

/// In-memory [`CredentialStore`].
#[derive(Default)]
pub struct InMemoryCredentialStore {
    secrets: Mutex<HashMap<String, String>>,
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn store(&self, key: &str, secret: &str) -> Result<(), DomainError> {
        self.secrets
            .lock()
            .expect("credential mutex poisoned")
            .insert(key.to_string(), secret.to_string());
        Ok(())
    }

    fn retrieve(&self, key: &str) -> Result<Option<String>, DomainError> {
        Ok(self
            .secrets
            .lock()
            .expect("credential mutex poisoned")
            .get(key)
            .cloned())
    }

    fn delete(&self, key: &str) -> Result<(), DomainError> {
        self.secrets
            .lock()
            .expect("credential mutex poisoned")
            .remove(key);
        Ok(())
    }
}

/// A [`Clock`] frozen at a fixed instant, for deterministic timestamps.
pub struct FixedClock {
    now_ms: i64,
}

impl FixedClock {
    pub fn new(now_ms: i64) -> Self {
        FixedClock { now_ms }
    }
}

impl Clock for FixedClock {
    fn now_ms(&self) -> i64 {
        self.now_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::Locator;

    fn sample_book() -> Book {
        Book::new(
            BookId::from_content(b"sample"),
            Format::Epub,
            BookMetadata {
                title: "Sample".to_string(),
                author: Some("Author".to_string()),
                cover: None,
            },
        )
    }

    #[test]
    fn repository_round_trips_a_book_and_its_progress() {
        let repo = InMemoryBookRepository::new();
        let book = sample_book();
        repo.insert_book(&book).unwrap();
        assert_eq!(repo.get_book(&book.id).unwrap().as_ref(), Some(&book));

        let progress = Progress {
            locator: Locator::new("p", 0.5),
            updated_at: 1_000,
        };
        repo.save_progress(&book.id, &progress).unwrap();

        let entries = repo.list_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].progress, 0.5);
        assert_eq!(entries[0].last_read, Some(1_000));
    }

    #[test]
    fn removing_a_book_drops_its_annotations() {
        let repo = InMemoryBookRepository::new();
        let book = sample_book();
        repo.insert_book(&book).unwrap();
        repo.add_bookmark(&Bookmark {
            id: "bm1".to_string(),
            book_id: book.id.clone(),
            locator: Locator::new("p", 0.1),
            created_at: 1,
        })
        .unwrap();

        repo.remove_book(&book.id).unwrap();
        assert!(repo.get_book(&book.id).unwrap().is_none());
        assert!(repo.list_bookmarks(&book.id).unwrap().is_empty());
        assert!(matches!(
            repo.remove_book(&book.id),
            Err(DomainError::BookNotFound(_))
        ));
    }

    #[test]
    fn fake_reader_reports_support_and_probes() {
        let reader = FakeReader::new(Format::Epub, "Title", Some("Author"));
        assert!(reader.supports(Format::Epub));
        assert!(!reader.supports(Format::Pdf));
        assert_eq!(reader.probe(b"bytes").unwrap().title, "Title");
        assert!(matches!(
            FakeReader::failing(Format::Pdf).probe(b""),
            Err(DomainError::InvalidFormat)
        ));
    }

    #[test]
    fn remote_store_uploads_lists_and_downloads() {
        let remote = InMemoryRemoteStore::new();
        remote.upload("books/a.epub", b"data").unwrap();
        let listed = remote.list("books/").unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(remote.download("books/a.epub").unwrap(), b"data");
        assert!(matches!(
            remote.download("missing"),
            Err(DomainError::ResourceNotFound(_))
        ));
    }

    #[test]
    fn credential_store_round_trips_and_deletes() {
        let creds = InMemoryCredentialStore::new();
        creds.store("webdav", "secret").unwrap();
        assert_eq!(creds.retrieve("webdav").unwrap().as_deref(), Some("secret"));
        creds.delete("webdav").unwrap();
        assert!(creds.retrieve("webdav").unwrap().is_none());
    }

    #[test]
    fn fixed_clock_returns_its_instant() {
        assert_eq!(FixedClock::new(42).now_ms(), 42);
    }
}
