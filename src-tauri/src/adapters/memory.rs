//! Temporary in-memory adapters and a wall clock for runtime use.
//!
//! These mirror the test fakes in `domain::testing` but are compiled for all
//! builds. They keep the app functional before the SQLite adapter lands in
//! Phase 3.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::error::DomainError;
use crate::domain::model::{
    Book, BookId, Bookmark, Highlight, LibraryEntry, Progress, ReadingSession, Settings,
};
use crate::domain::ports::{BookRepository, Clock, RemoteEntry, RemoteStore};

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
        state.bookmarks.retain(|b| &b.book_id != id);
        state.highlights.retain(|h| &h.book_id != id);
        state.sessions.retain(|s| &s.book_id != id);
        Ok(())
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
}

// ── InMemoryRemoteStore ─────────────────────────────────────────────────────

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

/// A [`Clock`] backed by [`SystemTime`], for production use.
pub struct WallClock;

impl Clock for WallClock {
    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_millis() as i64
    }
}
