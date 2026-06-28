//! The reading service: persist and resume the reading position, and report
//! progress.
//!
//! Position is a format-neutral [`Locator`]; the domain stores it and interprets
//! only its progression fraction, for the percentage display and the
//! furthest-position comparison sync reuses.

use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::model::{BookId, Locator, Progress};
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
