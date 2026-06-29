//! The library service: import, catalog listing, and removal.
//!
//! Pure logic over the [`BookRepository`] port and a [`ReaderRegistry`]. Import
//! derives a content-hash identity so the same file converges to one entry, and
//! listing applies search and sort in memory so the rules stay testable without
//! a database.

use std::cmp::Ordering;
use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::model::{Book, BookId, Format, LibraryEntry, LibraryQuery, SortKey};
use crate::domain::ports::{BookRepository, ReaderAdapter, ResourceContent};

/// Selects the reader adapter for a format. Adding a format is one more adapter
/// registered here, with no change to the service or the rest of the core.
#[derive(Clone, Default)]
pub struct ReaderRegistry {
    readers: Vec<Arc<dyn ReaderAdapter>>,
}

impl ReaderRegistry {
    pub fn new(readers: Vec<Arc<dyn ReaderAdapter>>) -> Self {
        ReaderRegistry { readers }
    }

    /// The first registered adapter that supports `format`.
    pub fn for_format(&self, format: Format) -> Result<&Arc<dyn ReaderAdapter>, DomainError> {
        self.readers
            .iter()
            .find(|reader| reader.supports(format))
            .ok_or(DomainError::NoReaderForFormat(format))
    }
}

/// Imports books, lists the catalog, and removes books.
pub struct LibraryService {
    repo: Arc<dyn BookRepository>,
    readers: ReaderRegistry,
}

impl LibraryService {
    pub fn new(repo: Arc<dyn BookRepository>, readers: ReaderRegistry) -> Self {
        LibraryService { repo, readers }
    }

    pub fn repo(&self) -> Arc<dyn BookRepository> {
        Arc::clone(&self.repo)
    }

    /// Import a book from its raw bytes (FR-LIB-01..03).
    ///
    /// Identity is the content hash, so re-importing the same file returns the
    /// existing entry instead of duplicating it. A book the library has not seen
    /// is probed for metadata and inserted.
    pub fn import(&self, bytes: &[u8], format: Format) -> Result<Book, DomainError> {
        let id = BookId::from_content(bytes);
        if let Some(existing) = self.repo.get_book(&id)? {
            return Ok(existing);
        }
        let metadata = self.readers.for_format(format)?.probe(bytes)?;
        let book = Book::new(id, format, metadata);
        self.repo.insert_book(&book)?;
        Ok(book)
    }

    /// List the catalog with search applied and the result sorted (FR-LIB-04..06).
    pub fn list(&self, query: &LibraryQuery) -> Result<Vec<LibraryEntry>, DomainError> {
        let mut entries = self.repo.list_entries()?;
        if let Some(search) = &query.search {
            let needle = search.trim().to_lowercase();
            if !needle.is_empty() {
                entries.retain(|entry| matches_search(entry, &needle));
            }
        }
        entries.sort_by(|a, b| {
            let ordering = compare(a, b, query.sort);
            if query.descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
        Ok(entries)
    }

    pub fn remove(&self, id: &BookId) -> Result<(), DomainError> {
        self.repo.remove_book(id)?;
        self.repo.add_deleted_book(id.as_str())?;
        Ok(())
    }

    /// Look up a single catalog entry by id, used by the `prose://` protocol to
    /// scope access so only books the library knows about resolve.
    pub fn get_book(&self, id: &BookId) -> Result<Option<Book>, DomainError> {
        self.repo.get_book(id)
    }

    /// Read a resource from a book through its format's reader adapter. The
    /// `prose://` protocol supplies the stored file bytes and the requested
    /// resource path.
    pub fn read_resource(
        &self,
        format: Format,
        bytes: &[u8],
        resource_path: &str,
    ) -> Result<ResourceContent, DomainError> {
        self.readers
            .for_format(format)?
            .read_resource(bytes, resource_path)
    }
}

/// Case-insensitive substring match against title and author (FR-LIB-05).
fn matches_search(entry: &LibraryEntry, needle: &str) -> bool {
    entry.book.metadata.title.to_lowercase().contains(needle)
        || entry
            .book
            .metadata
            .author
            .as_deref()
            .is_some_and(|author| author.to_lowercase().contains(needle))
}

/// Order two entries by the requested key, breaking ties on the stable id so the
/// result is deterministic regardless of store iteration order.
fn compare(a: &LibraryEntry, b: &LibraryEntry, sort: SortKey) -> Ordering {
    let primary = match sort {
        SortKey::Title => title_key(a).cmp(&title_key(b)),
        SortKey::Author => author_key(a).cmp(&author_key(b)),
        SortKey::LastRead => a
            .last_read
            .unwrap_or(i64::MIN)
            .cmp(&b.last_read.unwrap_or(i64::MIN)),
        SortKey::Progress => a.progress.total_cmp(&b.progress),
    };
    primary.then_with(|| a.book.id.as_str().cmp(b.book.id.as_str()))
}

fn title_key(entry: &LibraryEntry) -> String {
    entry.book.metadata.title.to_lowercase()
}

fn author_key(entry: &LibraryEntry) -> String {
    entry
        .book
        .metadata
        .author
        .as_deref()
        .unwrap_or("")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{BookMetadata, Locator, Progress};
    use crate::domain::testing::{FakeReader, InMemoryBookRepository};

    fn epub_registry() -> ReaderRegistry {
        ReaderRegistry::new(vec![Arc::new(FakeReader::new(
            Format::Epub,
            "Probed Title",
            Some("Probed Author"),
        ))])
    }

    fn book(seed: &[u8], title: &str, author: Option<&str>) -> Book {
        Book::new(
            BookId::from_content(seed),
            Format::Epub,
            BookMetadata {
                title: title.to_string(),
                author: author.map(str::to_string),
                cover: None,
            },
        )
    }

    fn titles(entries: &[LibraryEntry]) -> Vec<&str> {
        entries
            .iter()
            .map(|entry| entry.book.metadata.title.as_str())
            .collect()
    }

    #[test]
    fn import_probes_metadata_and_inserts() {
        let repo = Arc::new(InMemoryBookRepository::new());
        let service = LibraryService::new(repo.clone(), epub_registry());

        let imported = service.import(b"epub bytes", Format::Epub).unwrap();
        assert_eq!(imported.metadata.title, "Probed Title");
        assert_eq!(imported.id, BookId::from_content(b"epub bytes"));
        assert!(repo.get_book(&imported.id).unwrap().is_some());
    }

    #[test]
    fn import_dedupes_identical_content() {
        let repo = Arc::new(InMemoryBookRepository::new());
        let service = LibraryService::new(repo.clone(), epub_registry());

        let first = service.import(b"same bytes", Format::Epub).unwrap();
        let second = service.import(b"same bytes", Format::Epub).unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(repo.list_entries().unwrap().len(), 1);
    }

    #[test]
    fn import_without_a_reader_for_the_format_errors() {
        let repo = Arc::new(InMemoryBookRepository::new());
        let service = LibraryService::new(repo, epub_registry());

        assert!(matches!(
            service.import(b"pdf bytes", Format::Pdf),
            Err(DomainError::NoReaderForFormat(Format::Pdf))
        ));
    }

    #[test]
    fn import_propagates_a_probe_failure() {
        let repo = Arc::new(InMemoryBookRepository::new());
        let registry = ReaderRegistry::new(vec![Arc::new(FakeReader::failing(Format::Epub))]);
        let service = LibraryService::new(repo, registry);

        assert!(matches!(
            service.import(b"corrupt", Format::Epub),
            Err(DomainError::InvalidFormat)
        ));
    }

    #[test]
    fn list_searches_title_and_author_case_insensitively() {
        let repo = Arc::new(InMemoryBookRepository::new());
        repo.insert_book(&book(b"1", "The Odyssey", Some("Homer")))
            .unwrap();
        repo.insert_book(&book(b"2", "Dune", Some("Frank Herbert")))
            .unwrap();
        let service = LibraryService::new(repo, ReaderRegistry::default());

        let query = LibraryQuery {
            search: Some("HERB".to_string()),
            ..LibraryQuery::default()
        };
        let entries = service.list(&query).unwrap();
        assert_eq!(titles(&entries), vec!["Dune"]);
    }

    #[test]
    fn list_sorts_by_title_ascending_then_descending() {
        let repo = Arc::new(InMemoryBookRepository::new());
        repo.insert_book(&book(b"1", "banana", None)).unwrap();
        repo.insert_book(&book(b"2", "Apple", None)).unwrap();
        repo.insert_book(&book(b"3", "Cherry", None)).unwrap();
        let service = LibraryService::new(repo, ReaderRegistry::default());

        let ascending = service.list(&LibraryQuery::default()).unwrap();
        assert_eq!(titles(&ascending), vec!["Apple", "banana", "Cherry"]);

        let descending = service
            .list(&LibraryQuery {
                descending: true,
                ..LibraryQuery::default()
            })
            .unwrap();
        assert_eq!(titles(&descending), vec!["Cherry", "banana", "Apple"]);
    }

    #[test]
    fn list_sorts_by_progress_and_last_read() {
        let repo = Arc::new(InMemoryBookRepository::new());
        let early = book(b"1", "Early", None);
        let late = book(b"2", "Late", None);
        repo.insert_book(&early).unwrap();
        repo.insert_book(&late).unwrap();
        repo.save_progress(
            &early.id,
            &Progress {
                locator: Locator::new("a", 0.2),
                updated_at: 100,
            },
        )
        .unwrap();
        repo.save_progress(
            &late.id,
            &Progress {
                locator: Locator::new("b", 0.8),
                updated_at: 200,
            },
        )
        .unwrap();
        let service = LibraryService::new(repo, ReaderRegistry::default());

        let by_progress = service
            .list(&LibraryQuery {
                sort: SortKey::Progress,
                descending: true,
                ..LibraryQuery::default()
            })
            .unwrap();
        assert_eq!(titles(&by_progress), vec!["Late", "Early"]);

        let by_last_read = service
            .list(&LibraryQuery {
                sort: SortKey::LastRead,
                ..LibraryQuery::default()
            })
            .unwrap();
        assert_eq!(titles(&by_last_read), vec!["Early", "Late"]);
    }

    #[test]
    fn remove_deletes_the_book() {
        let repo = Arc::new(InMemoryBookRepository::new());
        let target = book(b"1", "Doomed", None);
        repo.insert_book(&target).unwrap();
        let service = LibraryService::new(repo.clone(), ReaderRegistry::default());

        service.remove(&target.id).unwrap();
        assert!(repo.get_book(&target.id).unwrap().is_none());
    }
}
