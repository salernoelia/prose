//! The annotation service: bookmarks and highlights.
//!
//! Each annotation is minted with a stable, content-derived id and a creation
//! timestamp from the [`Clock`] port, so sync stays idempotent and ordered.
//! The id hashes the salient fields, so the same annotation created at the same
//! instant converges to one record instead of duplicating.

use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::model::{BookId, Bookmark, Highlight, Locator};
use crate::domain::ports::{BookRepository, Clock};

/// Creates, lists, and deletes bookmarks and highlights.
pub struct AnnotationService {
    repo: Arc<dyn BookRepository>,
    clock: Arc<dyn Clock>,
}

impl AnnotationService {
    pub fn new(repo: Arc<dyn BookRepository>, clock: Arc<dyn Clock>) -> Self {
        AnnotationService { repo, clock }
    }

    /// Bookmark the given location (FR-NOTE-01).
    pub fn add_bookmark(
        &self,
        book_id: &BookId,
        locator: Locator,
    ) -> Result<Bookmark, DomainError> {
        let created_at = self.clock.now_ms();
        let bookmark = Bookmark {
            id: annotation_id("bm", book_id, &locator, "", created_at),
            book_id: book_id.clone(),
            locator,
            created_at,
        };
        self.repo.add_bookmark(&bookmark)?;
        Ok(bookmark)
    }

    /// The bookmarks for a book, in store order (FR-NOTE-01).
    pub fn list_bookmarks(&self, book_id: &BookId) -> Result<Vec<Bookmark>, DomainError> {
        self.repo.list_bookmarks(book_id)
    }

    /// Delete a bookmark by its id (FR-NOTE-01).
    pub fn delete_bookmark(&self, bookmark_id: &str) -> Result<(), DomainError> {
        self.repo.delete_bookmark(bookmark_id)
    }

    /// Highlight a selected text range (FR-NOTE-02). The selected `text` is
    /// stored so the highlight survives re-pagination, and must be non-empty.
    pub fn add_highlight(
        &self,
        book_id: &BookId,
        locator: Locator,
        text: String,
        color: Option<String>,
    ) -> Result<Highlight, DomainError> {
        if text.trim().is_empty() {
            return Err(DomainError::InvalidInput(
                "highlight text is empty".to_string(),
            ));
        }
        let created_at = self.clock.now_ms();
        let highlight = Highlight {
            id: annotation_id("hl", book_id, &locator, &text, created_at),
            book_id: book_id.clone(),
            locator,
            text,
            color,
            created_at,
        };
        self.repo.add_highlight(&highlight)?;
        Ok(highlight)
    }

    /// The highlights for a book, in store order (FR-NOTE-02).
    pub fn list_highlights(&self, book_id: &BookId) -> Result<Vec<Highlight>, DomainError> {
        self.repo.list_highlights(book_id)
    }

    /// Delete a highlight by its id (FR-NOTE-02).
    pub fn delete_highlight(&self, highlight_id: &str) -> Result<(), DomainError> {
        self.repo.delete_highlight(highlight_id)
    }
}

/// A stable id for an annotation, derived from the kind, the book, the location,
/// the selected text, and the creation time. Deterministic, so the same logical
/// annotation maps to one id and sync neither loses nor duplicates it.
fn annotation_id(
    kind: &str,
    book_id: &BookId,
    locator: &Locator,
    text: &str,
    created_at: i64,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(kind.as_bytes());
    hasher.update(book_id.as_str().as_bytes());
    hasher.update(locator.payload.as_bytes());
    hasher.update(text.as_bytes());
    hasher.update(&created_at.to_le_bytes());
    format!("{}_{}", kind, hasher.finalize().to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::testing::{FixedClock, InMemoryBookRepository};

    fn service(now_ms: i64) -> (Arc<InMemoryBookRepository>, AnnotationService) {
        let repo = Arc::new(InMemoryBookRepository::new());
        let clock = Arc::new(FixedClock::new(now_ms));
        let service = AnnotationService::new(repo.clone(), clock);
        (repo, service)
    }

    #[test]
    fn bookmarks_round_trip_and_delete() {
        let (_repo, service) = service(10);
        let book = BookId::from_content(b"book");

        let bookmark = service
            .add_bookmark(&book, Locator::new("ch1", 0.1))
            .unwrap();
        assert!(bookmark.id.starts_with("bm_"));
        assert_eq!(bookmark.created_at, 10);
        assert_eq!(
            service.list_bookmarks(&book).unwrap(),
            vec![bookmark.clone()]
        );

        service.delete_bookmark(&bookmark.id).unwrap();
        assert!(service.list_bookmarks(&book).unwrap().is_empty());
    }

    #[test]
    fn highlights_round_trip_and_delete() {
        let (_repo, service) = service(20);
        let book = BookId::from_content(b"book");

        let highlight = service
            .add_highlight(
                &book,
                Locator::new("ch2", 0.4),
                "a passage".to_string(),
                Some("yellow".to_string()),
            )
            .unwrap();
        assert!(highlight.id.starts_with("hl_"));
        assert_eq!(highlight.text, "a passage");
        assert_eq!(
            service.list_highlights(&book).unwrap(),
            vec![highlight.clone()]
        );

        service.delete_highlight(&highlight.id).unwrap();
        assert!(service.list_highlights(&book).unwrap().is_empty());
    }

    #[test]
    fn empty_highlight_text_is_rejected() {
        let (_repo, service) = service(0);
        let book = BookId::from_content(b"book");

        assert!(matches!(
            service.add_highlight(&book, Locator::new("p", 0.0), "   ".to_string(), None),
            Err(DomainError::InvalidInput(_))
        ));
    }

    #[test]
    fn annotations_are_scoped_to_their_book() {
        let (_repo, service) = service(0);
        let first = BookId::from_content(b"first");
        let second = BookId::from_content(b"second");

        service
            .add_bookmark(&first, Locator::new("p", 0.0))
            .unwrap();

        assert_eq!(service.list_bookmarks(&first).unwrap().len(), 1);
        assert!(service.list_bookmarks(&second).unwrap().is_empty());
    }

    #[test]
    fn the_same_bookmark_maps_to_a_stable_id() {
        let (_repo, service) = service(42);
        let book = BookId::from_content(b"book");

        let first = service.add_bookmark(&book, Locator::new("p", 0.5)).unwrap();
        let again = service.add_bookmark(&book, Locator::new("p", 0.5)).unwrap();
        assert_eq!(first.id, again.id);
    }
}
