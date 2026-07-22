//! SQLite implementation of the [`BookRepository`] port.
//!
//! Handles connections, migrations, and transactional mappings from domain
//! types to relational tables.

use rusqlite::Connection;
use std::sync::Mutex;

use crate::domain::error::DomainError;
use crate::domain::model::{
    ArchivedState, Book, BookId, BookMetadata, Bookmark, Format, Highlight, LibraryEntry, Locator,
    Progress, ReadingSession, ReadingStyle, Settings, Theme,
};
use crate::domain::ports::BookRepository;

/// SQLite-backed implementation of [`BookRepository`].
pub struct SqliteBookRepository {
    conn: Mutex<Connection>,
}

impl SqliteBookRepository {
    /// Open or create a repository at the given database file path.
    pub fn new(db_path: std::path::PathBuf) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON;", [])?;

        let repo = Self {
            conn: Mutex::new(conn),
        };
        repo.run_migrations()?;
        Ok(repo)
    }

    /// Open an in-memory repository for unit and integration testing.
    pub fn in_memory() -> Self {
        let conn = Connection::open_in_memory().expect("open memory db");
        conn.execute("PRAGMA foreign_keys = ON;", [])
            .expect("enable foreign keys");
        let repo = Self {
            conn: Mutex::new(conn),
        };
        repo.run_migrations().expect("run migrations");
        repo
    }

    fn run_migrations(&self) -> Result<(), rusqlite::Error> {
        let mut conn = self.conn.lock().unwrap();
        // Create schema_migrations table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);",
            [],
        )?;

        let current_version: u32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let migrations = vec![
            // Migration 1: Initial schema setup
            vec![
                r#"
                CREATE TABLE books (
                    id TEXT PRIMARY KEY,
                    format TEXT NOT NULL,
                    title TEXT NOT NULL,
                    author TEXT,
                    cover TEXT
                );
                "#,
                r#"
                CREATE TABLE progress (
                    book_id TEXT PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
                    payload TEXT NOT NULL,
                    progression REAL NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                "#,
                r#"
                CREATE TABLE bookmarks (
                    id TEXT PRIMARY KEY,
                    book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
                    payload TEXT NOT NULL,
                    progression REAL NOT NULL,
                    created_at INTEGER NOT NULL
                );
                "#,
                r#"
                CREATE TABLE highlights (
                    id TEXT PRIMARY KEY,
                    book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
                    payload TEXT NOT NULL,
                    progression REAL NOT NULL,
                    text TEXT NOT NULL,
                    color TEXT,
                    created_at INTEGER NOT NULL
                );
                "#,
                r#"
                CREATE TABLE settings (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    schema_version INTEGER NOT NULL,
                    theme TEXT NOT NULL,
                    font_family TEXT NOT NULL,
                    font_size REAL NOT NULL,
                    line_height REAL NOT NULL,
                    margin REAL NOT NULL
                );
                "#,
                r#"
                CREATE TABLE sync_state (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );
                "#,
                r#"
                CREATE TABLE outbox (
                    key TEXT PRIMARY KEY,
                    op TEXT NOT NULL
                );
                "#,
            ],
            // Migration 2: Deleted books tombstones
            vec![
                r#"
                CREATE TABLE deleted_books (
                    id TEXT PRIMARY KEY,
                    deleted_at INTEGER NOT NULL
                );
                "#,
            ],
            // Migration 3: Reading sessions (statistics atoms)
            vec![
                r#"
                CREATE TABLE reading_sessions (
                    id TEXT PRIMARY KEY,
                    book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
                    started_at INTEGER NOT NULL,
                    duration_seconds INTEGER NOT NULL
                );
                "#,
                r#"
                CREATE INDEX idx_reading_sessions_book_id ON reading_sessions(book_id);
                "#,
            ],
            // Migration 4: Forced text alignment for reflowable content
            vec![
                r#"
                ALTER TABLE settings ADD COLUMN text_align TEXT NOT NULL DEFAULT 'left';
                "#,
            ],
            // Migration 5: Deleted reading-session tombstones
            vec![
                r#"
                CREATE TABLE deleted_sessions (
                    id TEXT PRIMARY KEY,
                    deleted_at INTEGER NOT NULL
                );
                "#,
            ],
            // Migration 6: Book archiving flag
            vec![
                r#"
                ALTER TABLE books ADD COLUMN archived INTEGER NOT NULL DEFAULT 0;
                "#,
            ],
            // Migration 7: Archived change timestamp for last-write-wins sync.
            // 0 means the flag has never been set, so there is nothing to sync.
            vec![
                r#"
                ALTER TABLE books ADD COLUMN archived_at INTEGER NOT NULL DEFAULT 0;
                "#,
            ],
        ];

        for (idx, statements) in migrations.into_iter().enumerate() {
            let version = (idx + 1) as u32;
            if version > current_version {
                let tx = conn.transaction()?;
                for sql in statements {
                    tx.execute(sql, [])?;
                }
                tx.execute(
                    "INSERT INTO schema_migrations (version) VALUES (?1);",
                    [version],
                )?;
                tx.commit()?;
            }
        }

        Ok(())
    }
}

fn format_to_str(f: Format) -> &'static str {
    match f {
        Format::Epub => "epub",
        Format::Pdf => "pdf",
    }
}

impl BookRepository for SqliteBookRepository {
    fn insert_book(&self, book: &Book) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO books (id, format, title, author, cover) VALUES (?1, ?2, ?3, ?4, ?5);",
            (
                book.id.as_str(),
                format_to_str(book.format),
                &book.metadata.title,
                &book.metadata.author,
                &book.metadata.cover,
            ),
        ).map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_book(&self, id: &BookId) -> Result<Option<Book>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT format, title, author, cover FROM books WHERE id = ?1;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query([id.as_str()])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| DomainError::Storage(e.to_string()))?
        {
            let format_str: String = row
                .get(0)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let title: String = row
                .get(1)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let author: Option<String> = row
                .get(2)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let cover: Option<String> = row
                .get(3)
                .map_err(|e| DomainError::Storage(e.to_string()))?;

            let format = match format_str.as_str() {
                "epub" => Format::Epub,
                "pdf" => Format::Pdf,
                _ => return Err(DomainError::Storage("invalid format in db".to_string())),
            };

            Ok(Some(Book::new(
                id.clone(),
                format,
                BookMetadata {
                    title,
                    author,
                    cover,
                },
            )))
        } else {
            Ok(None)
        }
    }

    fn list_entries(&self) -> Result<Vec<LibraryEntry>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT
                b.id, b.format, b.title, b.author, b.cover,
                p.payload, p.progression, p.updated_at, b.archived
             FROM books b
             LEFT JOIN progress p ON b.id = p.book_id",
            )
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        let entries = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let format_str: String = row.get(1)?;
                let title: String = row.get(2)?;
                let author: Option<String> = row.get(3)?;
                let cover: Option<String> = row.get(4)?;

                let progress_progression: Option<f32> = row.get(6)?;
                let progress_updated_at: Option<i64> = row.get(7)?;
                let archived: bool = row.get(8)?;

                let format = match format_str.as_str() {
                    "epub" => Format::Epub,
                    "pdf" => Format::Pdf,
                    _ => {
                        return Err(rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "invalid format",
                            )),
                        ))
                    }
                };

                Ok(LibraryEntry {
                    book: Book::new(
                        BookId::from_hash(id_str),
                        format,
                        BookMetadata {
                            title,
                            author,
                            cover,
                        },
                    ),
                    progress: progress_progression.unwrap_or(0.0),
                    last_read: progress_updated_at,
                    archived,
                })
            })
            .map_err(|e| DomainError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        Ok(entries)
    }

    fn remove_book(&self, id: &BookId) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn
            .execute("DELETE FROM books WHERE id = ?1;", [id.as_str()])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if rows_affected == 0 {
            return Err(DomainError::BookNotFound(id.as_str().to_string()));
        }
        Ok(())
    }

    fn set_archived(&self, id: &BookId, state: &ArchivedState) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn
            .execute(
                "UPDATE books SET archived = ?1, archived_at = ?2 WHERE id = ?3;",
                (state.archived, state.updated_at, id.as_str()),
            )
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if rows_affected == 0 {
            return Err(DomainError::BookNotFound(id.as_str().to_string()));
        }
        Ok(())
    }

    fn get_archived(&self, id: &BookId) -> Result<Option<ArchivedState>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT archived, archived_at FROM books WHERE id = ?1;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query([id.as_str()])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| DomainError::Storage(e.to_string()))?
        {
            let archived: bool = row
                .get(0)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let updated_at: i64 = row
                .get(1)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            // A zero timestamp marks a book whose flag was never set: nothing to sync.
            if updated_at == 0 {
                Ok(None)
            } else {
                Ok(Some(ArchivedState {
                    archived,
                    updated_at,
                }))
            }
        } else {
            Ok(None)
        }
    }

    fn save_progress(&self, id: &BookId, progress: &Progress) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO progress (book_id, payload, progression, updated_at) VALUES (?1, ?2, ?3, ?4);",
            (
                id.as_str(),
                &progress.locator.payload,
                progress.locator.progression,
                progress.updated_at,
            ),
        ).map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_progress(&self, id: &BookId) -> Result<Option<Progress>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT payload, progression, updated_at FROM progress WHERE book_id = ?1;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query([id.as_str()])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| DomainError::Storage(e.to_string()))?
        {
            let payload: String = row
                .get(0)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let progression: f32 = row
                .get(1)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let updated_at: i64 = row
                .get(2)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            Ok(Some(Progress {
                locator: Locator::new(payload, progression),
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    fn add_bookmark(&self, bookmark: &Bookmark) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO bookmarks (id, book_id, payload, progression, created_at) VALUES (?1, ?2, ?3, ?4, ?5);",
            (
                &bookmark.id,
                bookmark.book_id.as_str(),
                &bookmark.locator.payload,
                bookmark.locator.progression,
                bookmark.created_at,
            ),
        ).map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn list_bookmarks(&self, id: &BookId) -> Result<Vec<Bookmark>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, payload, progression, created_at FROM bookmarks WHERE book_id = ?1 ORDER BY created_at ASC;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let entries = stmt
            .query_map([id.as_str()], |row| {
                let bookmark_id: String = row.get(0)?;
                let payload: String = row.get(1)?;
                let progression: f32 = row.get(2)?;
                let created_at: i64 = row.get(3)?;
                Ok(Bookmark {
                    id: bookmark_id,
                    book_id: id.clone(),
                    locator: Locator::new(payload, progression),
                    created_at,
                })
            })
            .map_err(|e| DomainError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(entries)
    }

    fn delete_bookmark(&self, bookmark_id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM bookmarks WHERE id = ?1;", [bookmark_id])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn add_highlight(&self, highlight: &Highlight) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO highlights (id, book_id, payload, progression, text, color, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            (
                &highlight.id,
                highlight.book_id.as_str(),
                &highlight.locator.payload,
                highlight.locator.progression,
                &highlight.text,
                &highlight.color,
                highlight.created_at,
            ),
        ).map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn list_highlights(&self, id: &BookId) -> Result<Vec<Highlight>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, payload, progression, text, color, created_at FROM highlights WHERE book_id = ?1 ORDER BY created_at ASC;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let entries = stmt
            .query_map([id.as_str()], |row| {
                let highlight_id: String = row.get(0)?;
                let payload: String = row.get(1)?;
                let progression: f32 = row.get(2)?;
                let text: String = row.get(3)?;
                let color: Option<String> = row.get(4)?;
                let created_at: i64 = row.get(5)?;
                Ok(Highlight {
                    id: highlight_id,
                    book_id: id.clone(),
                    locator: Locator::new(payload, progression),
                    text,
                    color,
                    created_at,
                })
            })
            .map_err(|e| DomainError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(entries)
    }

    fn delete_highlight(&self, highlight_id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM highlights WHERE id = ?1;", [highlight_id])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn add_reading_session(&self, session: &ReadingSession) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO reading_sessions (id, book_id, started_at, duration_seconds) VALUES (?1, ?2, ?3, ?4);",
            (
                &session.id,
                session.book_id.as_str(),
                session.started_at,
                session.duration_seconds,
            ),
        )
        .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn list_reading_sessions(&self, id: &BookId) -> Result<Vec<ReadingSession>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, started_at, duration_seconds FROM reading_sessions WHERE book_id = ?1 ORDER BY started_at ASC;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let entries = stmt
            .query_map([id.as_str()], |row| {
                Ok(ReadingSession {
                    id: row.get(0)?,
                    book_id: id.clone(),
                    started_at: row.get(1)?,
                    duration_seconds: row.get(2)?,
                })
            })
            .map_err(|e| DomainError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(entries)
    }

    fn list_all_reading_sessions(&self) -> Result<Vec<ReadingSession>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, book_id, started_at, duration_seconds FROM reading_sessions ORDER BY started_at DESC;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let entries = stmt
            .query_map([], |row| {
                let book_id: String = row.get(1)?;
                Ok(ReadingSession {
                    id: row.get(0)?,
                    book_id: BookId::from_hash(book_id),
                    started_at: row.get(2)?,
                    duration_seconds: row.get(3)?,
                })
            })
            .map_err(|e| DomainError::Storage(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(entries)
    }

    fn delete_reading_session(&self, session_id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM reading_sessions WHERE id = ?1;", [session_id])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_deleted_sessions(&self) -> Result<Vec<String>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id FROM deleted_sessions ORDER BY deleted_at ASC;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let ids: Vec<String> = rows.flatten().collect();
        Ok(ids)
    }

    fn add_deleted_session(&self, id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        conn.execute(
            "INSERT OR REPLACE INTO deleted_sessions (id, deleted_at) VALUES (?1, ?2);",
            rusqlite::params![id, now],
        )
        .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn remove_deleted_session(&self, id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM deleted_sessions WHERE id = ?1;", [id])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_settings(&self) -> Result<Option<Settings>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT schema_version, theme, font_family, font_size, line_height, margin, text_align FROM settings WHERE id = 1;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query([])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| DomainError::Storage(e.to_string()))?
        {
            let schema_version: u32 = row
                .get(0)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let theme_str: String = row
                .get(1)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let font_family: String = row
                .get(2)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let font_size: f32 = row
                .get(3)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let line_height: f32 = row
                .get(4)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let margin: f32 = row
                .get(5)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let text_align: String = row
                .get(6)
                .map_err(|e| DomainError::Storage(e.to_string()))?;

            let theme = match theme_str.as_str() {
                "light" => Theme::Light,
                "paper" => Theme::Paper,
                "dark" => Theme::Dark,
                "oled" => Theme::Oled,
                "night" => Theme::Night,
                "sepia" => Theme::Sepia,
                "sepia-dark" => Theme::SepiaDark,
                "eink" => Theme::Eink,
                "eink-dark" => Theme::EinkDark,
                _ => Theme::Light,
            };

            Ok(Some(Settings {
                schema_version,
                theme,
                reading: ReadingStyle {
                    font_family,
                    font_size,
                    line_height,
                    margin,
                    text_align,
                },
            }))
        } else {
            Ok(None)
        }
    }

    fn save_settings(&self, settings: &Settings) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let theme_str = match settings.theme {
            Theme::Light => "light",
            Theme::Paper => "paper",
            Theme::Dark => "dark",
            Theme::Oled => "oled",
            Theme::Night => "night",
            Theme::Sepia => "sepia",
            Theme::SepiaDark => "sepia-dark",
            Theme::Eink => "eink",
            Theme::EinkDark => "eink-dark",
        };
        conn.execute(
            "INSERT OR REPLACE INTO settings (id, schema_version, theme, font_family, font_size, line_height, margin, text_align) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7);",
            (
                settings.schema_version,
                theme_str,
                &settings.reading.font_family,
                settings.reading.font_size,
                settings.reading.line_height,
                settings.reading.margin,
                &settings.reading.text_align,
            ),
        ).map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_sync_state(&self, key: &str) -> Result<Option<String>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT value FROM sync_state WHERE key = ?1;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query([key])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if let Some(row) = rows
            .next()
            .map_err(|e| DomainError::Storage(e.to_string()))?
        {
            let value: String = row
                .get(0)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn save_sync_state(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO sync_state (key, value) VALUES (?1, ?2);",
            [key, value],
        )
        .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn delete_sync_state(&self, key: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sync_state WHERE key = ?1;", [key])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn get_deleted_books(&self) -> Result<Vec<String>, DomainError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id FROM deleted_books ORDER BY deleted_at ASC;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let ids: Vec<String> = rows.flatten().collect();
        Ok(ids)
    }

    fn add_deleted_book(&self, id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        conn.execute(
            "INSERT OR REPLACE INTO deleted_books (id, deleted_at) VALUES (?1, ?2);",
            rusqlite::params![id, now],
        )
        .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    fn remove_deleted_book(&self, id: &str) -> Result<(), DomainError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM deleted_books WHERE id = ?1;", [id])
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> SqliteBookRepository {
        SqliteBookRepository::in_memory()
    }

    #[test]
    fn test_books_crud() {
        let repo = repo();
        let book = Book::new(
            BookId::from_content(b"sqlite-crud-test"),
            Format::Epub,
            BookMetadata {
                title: "Test Book".to_string(),
                author: Some("Author Name".to_string()),
                cover: None,
            },
        );

        // Get non-existent
        assert!(repo.get_book(&book.id).unwrap().is_none());

        // Insert
        repo.insert_book(&book).unwrap();

        // Get existing
        let loaded = repo.get_book(&book.id).unwrap().unwrap();
        assert_eq!(loaded.id, book.id);
        assert_eq!(loaded.metadata.title, book.metadata.title);
        assert_eq!(loaded.metadata.author, book.metadata.author);

        // List
        let entries = repo.list_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].book.id, book.id);
        assert_eq!(entries[0].progress, 0.0);
        assert_eq!(entries[0].last_read, None);

        // Remove
        repo.remove_book(&book.id).unwrap();
        assert!(repo.get_book(&book.id).unwrap().is_none());

        // Remove non-existent returns error
        assert!(repo.remove_book(&book.id).is_err());
    }

    #[test]
    fn test_progress_cascade() {
        let repo = repo();
        let book = Book::new(
            BookId::from_content(b"cascade-test"),
            Format::Pdf,
            BookMetadata {
                title: "Cascade Book".to_string(),
                author: None,
                cover: None,
            },
        );
        repo.insert_book(&book).unwrap();

        let progress = Progress {
            locator: Locator::new("page-3", 0.3),
            updated_at: 123456789,
        };

        repo.save_progress(&book.id, &progress).unwrap();

        let loaded = repo.get_progress(&book.id).unwrap().unwrap();
        assert_eq!(loaded.locator.payload, "page-3");
        assert_eq!(loaded.locator.progression, 0.3);
        assert_eq!(loaded.updated_at, 123456789);

        // Verify list_entries reflects progress
        let entries = repo.list_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].progress, 0.3);
        assert_eq!(entries[0].last_read, Some(123456789));

        // Delete book and ensure progress is cascade-deleted
        repo.remove_book(&book.id).unwrap();
        assert!(repo.get_progress(&book.id).unwrap().is_none());
    }

    #[test]
    fn test_bookmarks_highlights_cascade() {
        let repo = repo();
        let book = Book::new(
            BookId::from_content(b"bookmarks-test"),
            Format::Epub,
            BookMetadata {
                title: "Bookmarks Book".to_string(),
                author: None,
                cover: None,
            },
        );
        repo.insert_book(&book).unwrap();

        let bookmark = Bookmark {
            id: "b1".to_string(),
            book_id: book.id.clone(),
            locator: Locator::new("epubcfi(1)", 0.1),
            created_at: 100,
        };
        repo.add_bookmark(&bookmark).unwrap();

        let highlight = Highlight {
            id: "h1".to_string(),
            book_id: book.id.clone(),
            locator: Locator::new("epubcfi(2)", 0.2),
            text: "Hello highlight".to_string(),
            color: Some("yellow".to_string()),
            created_at: 200,
        };
        repo.add_highlight(&highlight).unwrap();

        let bookmarks = repo.list_bookmarks(&book.id).unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].id, "b1");

        let highlights = repo.list_highlights(&book.id).unwrap();
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].id, "h1");

        // Delete bookmark directly
        repo.delete_bookmark("b1").unwrap();
        assert!(repo.list_bookmarks(&book.id).unwrap().is_empty());

        // Delete highlight directly
        repo.delete_highlight("h1").unwrap();
        assert!(repo.list_highlights(&book.id).unwrap().is_empty());
    }

    #[test]
    fn test_reading_sessions_and_cascade() {
        let repo = repo();
        let book = Book::new(
            BookId::from_content(b"sessions-test"),
            Format::Epub,
            BookMetadata {
                title: "Sessions Book".to_string(),
                author: None,
                cover: None,
            },
        );
        repo.insert_book(&book).unwrap();

        let s1 = ReadingSession {
            id: "s1".to_string(),
            book_id: book.id.clone(),
            started_at: 1_000,
            duration_seconds: 60,
        };
        let s2 = ReadingSession {
            id: "s2".to_string(),
            book_id: book.id.clone(),
            started_at: 2_000,
            duration_seconds: 120,
        };
        repo.add_reading_session(&s1).unwrap();
        repo.add_reading_session(&s2).unwrap();

        // Idempotent on id: re-adding s1 does not duplicate it.
        repo.add_reading_session(&s1).unwrap();

        let per_book = repo.list_reading_sessions(&book.id).unwrap();
        assert_eq!(per_book.len(), 2);
        assert_eq!(per_book[0].id, "s1");

        let all = repo.list_all_reading_sessions().unwrap();
        assert_eq!(all.len(), 2);
        // Newest first.
        assert_eq!(all[0].id, "s2");

        // Deleting the book cascade-removes its sessions.
        repo.remove_book(&book.id).unwrap();
        assert!(repo.list_all_reading_sessions().unwrap().is_empty());
    }

    #[test]
    fn test_settings() {
        let repo = repo();
        assert!(repo.get_settings().unwrap().is_none());

        let settings = Settings {
            schema_version: 1,
            theme: Theme::Sepia,
            reading: ReadingStyle {
                font_family: "Georgia".to_string(),
                font_size: 19.0,
                line_height: 1.6,
                margin: 1.2,
                text_align: "justify".to_string(),
            },
        };
        repo.save_settings(&settings).unwrap();

        let loaded = repo.get_settings().unwrap().unwrap();
        assert_eq!(loaded.schema_version, 1);
        assert_eq!(loaded.theme, Theme::Sepia);
        assert_eq!(loaded.reading.font_family, "Georgia");
        assert_eq!(loaded.reading.font_size, 19.0);
        assert_eq!(loaded.reading.text_align, "justify");
    }

    #[test]
    fn test_temp_file_persistence_and_fk_atomicity() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = std::env::temp_dir().join(format!("prose_test_{}.db", now));

        if db_path.exists() {
            let _ = std::fs::remove_file(&db_path);
        }

        {
            let repo = SqliteBookRepository::new(db_path.clone()).unwrap();
            let settings = Settings {
                schema_version: 1,
                theme: Theme::Dark,
                reading: ReadingStyle::default(),
            };
            repo.save_settings(&settings).unwrap();
        }

        {
            let repo = SqliteBookRepository::new(db_path.clone()).unwrap();
            let settings = repo.get_settings().unwrap().unwrap();
            assert_eq!(settings.theme, Theme::Dark);

            // FK constraint violation test
            let progress = Progress {
                locator: Locator::new("xyz", 0.5),
                updated_at: 100,
            };
            let book_id = BookId::from_hash("non-existent-id");
            let result = repo.save_progress(&book_id, &progress);
            assert!(result.is_err());
        }

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_deleted_books_tombstones() {
        let repo = repo();
        assert!(repo.get_deleted_books().unwrap().is_empty());

        repo.add_deleted_book("book1").unwrap();
        repo.add_deleted_book("book2").unwrap();

        let deleted = repo.get_deleted_books().unwrap();
        assert_eq!(deleted.len(), 2);
        assert_eq!(deleted[0], "book1");
        assert_eq!(deleted[1], "book2");
    }
}
