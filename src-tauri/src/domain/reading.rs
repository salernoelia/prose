//! The reading service: persist and resume the reading position, and report
//! progress.
//!
//! Position is a format-neutral [`Locator`]; the domain stores it and interprets
//! only its progression fraction, for the percentage display and the
//! furthest-position comparison sync reuses.

use std::sync::Arc;

use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::error::DomainError;
use crate::domain::model::{BookId, Locator, Progress, ReadingSession};
use crate::domain::ports::{BookRepository, Clock};

/// Saves and resumes reading positions and derives progress.
pub struct ReadingService {
    repo: Arc<dyn BookRepository>,
    clock: Arc<dyn Clock>,
}

impl ReadingService {
    pub fn new(repo: Arc<dyn BookRepository>, clock: Arc<dyn Clock>) -> Self {
        ReadingService { repo, clock }
    }

    pub fn clock(&self) -> Arc<dyn crate::domain::ports::Clock> {
        Arc::clone(&self.clock)
    }

    /// Save the current position for a book, stamped with the current time so
    /// later writes and sync can order it (FR-READ-06).
    pub fn save_position(&self, id: &BookId, locator: Locator) -> Result<Progress, DomainError> {
        let progress = Progress {
            locator,
            updated_at: self.clock.now_ms(),
        };
        self.repo.save_progress(id, &progress)?;
        Ok(progress)
    }

    /// The saved position for a book, used to resume where the reader left off
    /// (FR-READ-06). `None` if the book has not been opened.
    pub fn get_position(&self, id: &BookId) -> Result<Option<Progress>, DomainError> {
        self.repo.get_progress(id)
    }

    /// Reading progress as a whole percentage in `[0, 100]` (FR-READ-07). An
    /// unread book reports `0`.
    pub fn progress_percent(&self, id: &BookId) -> Result<u8, DomainError> {
        Ok(self
            .get_position(id)?
            .map_or(0, |progress| percent(progress.locator.progression)))
    }

    /// Record a reading session for a book: a span that began at `started_at`
    /// (epoch milliseconds) and lasted `duration_seconds`. The session gets a
    /// stable, unique id so it syncs idempotently. Returns the stored session.
    pub fn log_session(
        &self,
        id: &BookId,
        started_at: i64,
        duration_seconds: i64,
    ) -> Result<ReadingSession, DomainError> {
        let session = ReadingSession {
            id: new_session_id(id, started_at),
            book_id: id.clone(),
            started_at,
            duration_seconds,
        };
        self.repo.add_reading_session(&session)?;
        Ok(session)
    }

    /// Every recorded reading session across the library, newest first. The
    /// statistics view derives reading time, streaks, and the activity charts
    /// from these; no aggregate is stored.
    pub fn list_sessions(&self) -> Result<Vec<ReadingSession>, DomainError> {
        let mut sessions = self.repo.list_all_reading_sessions()?;
        sessions.sort_by_key(|s| std::cmp::Reverse(s.started_at));
        Ok(sessions)
    }
}

/// Derive a stable, unique session id. The content (book, start instant) plus a
/// process-monotonic counter guarantees uniqueness even when two sessions share
/// a millisecond, and the hash keeps ids opaque and fixed-width like book ids.
fn new_session_id(book_id: &BookId, started_at: i64) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
    let seed = format!("{}:{}:{}", book_id.as_str(), started_at, nonce);
    blake3::hash(seed.as_bytes()).to_hex()[..32].to_string()
}

/// Convert a progression fraction to a whole percentage, rounded to nearest.
fn percent(progression: f32) -> u8 {
    (progression.clamp(0.0, 1.0) * 100.0).round() as u8
}

/// The position furthest through the book by progression fraction. On an exact
/// tie the first argument wins, so callers pass the local position first to
/// prefer it. This is the rule sync uses for reading-position conflicts
/// (FR-SYNC-04).
pub fn furthest<'a>(a: &'a Progress, b: &'a Progress) -> &'a Progress {
    if b.locator.progression > a.locator.progression {
        b
    } else {
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::testing::{FixedClock, InMemoryBookRepository};

    fn service(now_ms: i64) -> (Arc<InMemoryBookRepository>, ReadingService) {
        let repo = Arc::new(InMemoryBookRepository::new());
        let clock = Arc::new(FixedClock::new(now_ms));
        let service = ReadingService::new(repo.clone(), clock);
        (repo, service)
    }

    #[test]
    fn save_then_get_resumes_the_position() {
        let (_repo, service) = service(1_234);
        let book = BookId::from_content(b"book");

        assert!(service.get_position(&book).unwrap().is_none());

        let saved = service
            .save_position(&book, Locator::new("ch3", 0.42))
            .unwrap();
        assert_eq!(saved.updated_at, 1_234);

        let resumed = service.get_position(&book).unwrap().unwrap();
        assert_eq!(resumed.locator, Locator::new("ch3", 0.42));
        assert_eq!(resumed.updated_at, 1_234);
    }

    #[test]
    fn progress_percent_rounds_and_defaults_to_zero() {
        let (_repo, service) = service(0);
        let book = BookId::from_content(b"book");

        assert_eq!(service.progress_percent(&book).unwrap(), 0);

        service
            .save_position(&book, Locator::new("p", 0.125))
            .unwrap();
        assert_eq!(service.progress_percent(&book).unwrap(), 13);

        service
            .save_position(&book, Locator::new("p", 1.0))
            .unwrap();
        assert_eq!(service.progress_percent(&book).unwrap(), 100);
    }

    #[test]
    fn log_session_records_a_unique_session_per_call() {
        let (repo, service) = service(0);
        let book = BookId::from_content(b"book");
        repo.insert_book(&crate::domain::model::Book::new(
            book.clone(),
            crate::domain::model::Format::Epub,
            crate::domain::model::BookMetadata {
                title: "T".to_string(),
                author: None,
                cover: None,
            },
        ))
        .unwrap();

        let a = service.log_session(&book, 1_000, 60).unwrap();
        let b = service.log_session(&book, 1_000, 90).unwrap();

        // Same book and start instant still yield distinct ids.
        assert_ne!(a.id, b.id);

        let sessions = service.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(
            sessions.iter().map(|s| s.duration_seconds).sum::<i64>(),
            150
        );
    }

    #[test]
    fn furthest_keeps_the_greater_progression() {
        let behind = Progress {
            locator: Locator::new("a", 0.3),
            updated_at: 200,
        };
        let ahead = Progress {
            locator: Locator::new("b", 0.7),
            updated_at: 100,
        };
        assert_eq!(furthest(&behind, &ahead), &ahead);
        assert_eq!(furthest(&ahead, &behind), &ahead);
    }

    #[test]
    fn furthest_breaks_ties_in_favor_of_the_first() {
        let local = Progress {
            locator: Locator::new("local", 0.5),
            updated_at: 1,
        };
        let remote = Progress {
            locator: Locator::new("remote", 0.5),
            updated_at: 2,
        };
        assert_eq!(furthest(&local, &remote), &local);
    }
}
