//! Core domain model: the value types the services operate on.
//!
//! These types are pure data with `serde`, `Clone`, and `PartialEq`. They carry
//! no behavior beyond construction helpers and the small amount of
//! interpretation the domain is allowed (content hashing for identity, the
//! progression fraction for the furthest-position comparison).

use serde::{Deserialize, Serialize};

/// The settings schema version written by this build. Bumped when the shape of
/// [`Settings`] changes in a way the migration logic must account for.
pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

/// Stable identity for a book: the lowercase hex BLAKE3 hash of its file
/// content. The same book imported on two devices converges to one id, so sync
/// never duplicates it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookId(String);

impl BookId {
    /// Derive the id from the full book bytes. Deterministic: equal content
    /// always yields an equal id.
    pub fn from_content(bytes: &[u8]) -> Self {
        BookId(blake3::hash(bytes).to_hex().to_string())
    }

    /// Construct an id from an already-computed hash string, for rehydrating
    /// from the store.
    pub fn from_hash(hash: impl Into<String>) -> Self {
        BookId(hash.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A supported book format. Adding a format is a new adapter, not a domain
/// change, but the enum is the one place the core names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Epub,
    Pdf,
}

/// Title, author, and cover extracted from a book by a `ReaderAdapter`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: String,
    pub author: Option<String>,
    /// Reference to a stored cover image (a path under app storage), if one was
    /// extracted. The domain treats it as opaque.
    pub cover: Option<String>,
}

/// A book in the library: its content-hash identity, format, and metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub id: BookId,
    pub format: Format,
    pub metadata: BookMetadata,
}

impl Book {
    pub fn new(id: BookId, format: Format, metadata: BookMetadata) -> Self {
        Book {
            id,
            format,
            metadata,
        }
    }
}

/// How the library list is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    #[default]
    Title,
    Author,
    LastRead,
    Progress,
}

/// A request to list the library: free-text search plus sort order. Filtering
/// and sorting are applied by `LibraryService`, not the store, so the logic is
/// pure and unit-tested.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LibraryQuery {
    /// Case-insensitive substring matched against title and author.
    pub search: Option<String>,
    pub sort: SortKey,
    pub descending: bool,
}

/// A book as it appears in the library list: the book plus the derived fields
/// the list sorts by. The store returns these in one pass so there is no
/// per-book lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub book: Book,
    /// Furthest progression recorded for the book, `0.0` if unread.
    pub progress: f32,
    /// When the book was last read, epoch milliseconds, if ever.
    pub last_read: Option<i64>,
}

/// A format-neutral reading position. `payload` is opaque to the domain (a CFI
/// for ePub, a page index plus offset for PDF); only `progression` is
/// interpreted, and only for the furthest-position comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Locator {
    /// Opaque, renderer-produced position payload.
    pub payload: String,
    /// Fraction of the way through the book, in `[0.0, 1.0]`.
    pub progression: f32,
}

impl Locator {
    pub fn new(payload: impl Into<String>, progression: f32) -> Self {
        Locator {
            payload: payload.into(),
            progression: progression.clamp(0.0, 1.0),
        }
    }
}

/// The saved reading position for a book, with the time it was last updated
/// (epoch milliseconds) for conflict resolution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    pub locator: Locator,
    pub updated_at: i64,
}

/// A bookmark at a saved location. `id` is stable and unique so sync stays
/// idempotent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub book_id: BookId,
    pub locator: Locator,
    pub created_at: i64,
}

/// A highlight over a selected text range. The selected `text` is stored so the
/// highlight survives re-pagination of reflowable content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Highlight {
    pub id: String,
    pub book_id: BookId,
    pub locator: Locator,
    pub text: String,
    /// Optional color label chosen by the reader.
    pub color: Option<String>,
    pub created_at: i64,
}

/// One sense of a word from the offline dictionary (FR-NOTE-03). `synonyms` are
/// the other words sharing this sense, and `examples` are usage sentences when
/// the data set provides them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Definition {
    /// The grammatical class in readable form: "noun", "verb", "adjective", etc.
    pub part_of_speech: String,
    /// The definition text.
    pub gloss: String,
    /// Other words in the same sense.
    pub synonyms: Vec<String>,
    /// Example usage sentences, if any.
    pub examples: Vec<String>,
}

/// Typography settings applied to reflowable ePub content. PDF ignores these.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingStyle {
    /// Font family name, chosen from the bundled set.
    #[serde(default = "ReadingStyle::default_font_family")]
    pub font_family: String,
    /// Font size in pixels.
    #[serde(default = "ReadingStyle::default_font_size")]
    pub font_size: f32,
    /// Line height as a multiple of the font size.
    #[serde(default = "ReadingStyle::default_line_height")]
    pub line_height: f32,
    /// Page margin as a multiple of a base margin unit.
    #[serde(default = "ReadingStyle::default_margin")]
    pub margin: f32,
}

impl ReadingStyle {
    fn default_font_family() -> String {
        "Literata".to_string()
    }
    fn default_font_size() -> f32 {
        18.0
    }
    fn default_line_height() -> f32 {
        1.5
    }
    fn default_margin() -> f32 {
        1.0
    }
}

impl Default for ReadingStyle {
    fn default() -> Self {
        ReadingStyle {
            font_family: Self::default_font_family(),
            font_size: Self::default_font_size(),
            line_height: Self::default_line_height(),
            margin: Self::default_margin(),
        }
    }
}

/// The reading theme, applied to both the reading view and the app shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
    Sepia,
}

/// All reader-customizable settings. Versioned and fully defaulted: a missing
/// or unknown field falls back to its default, so an older or newer settings
/// file never crashes the app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "Settings::default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub reading: ReadingStyle,
}

impl Settings {
    fn default_schema_version() -> u32 {
        SETTINGS_SCHEMA_VERSION
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            schema_version: SETTINGS_SCHEMA_VERSION,
            theme: Theme::default(),
            reading: ReadingStyle::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn book_id_is_deterministic() {
        let a = BookId::from_content(b"the same bytes");
        let b = BookId::from_content(b"the same bytes");
        assert_eq!(a, b);
    }

    #[test]
    fn book_id_differs_for_different_content() {
        let a = BookId::from_content(b"one book");
        let b = BookId::from_content(b"another book");
        assert_ne!(a, b);
    }

    #[test]
    fn book_id_is_hex_of_expected_length() {
        let id = BookId::from_content(b"content");
        // BLAKE3 default output is 32 bytes -> 64 lowercase hex characters.
        assert_eq!(id.as_str().len(), 64);
        assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn locator_clamps_progression() {
        assert_eq!(Locator::new("p", 1.7).progression, 1.0);
        assert_eq!(Locator::new("p", -0.4).progression, 0.0);
    }

    #[test]
    fn settings_default_is_current_version_and_light() {
        let s = Settings::default();
        assert_eq!(s.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(s.theme, Theme::Light);
        assert_eq!(s.reading, ReadingStyle::default());
    }

    #[test]
    fn settings_fills_missing_fields_from_default() {
        // A settings file from an older build that only wrote the theme still
        // deserializes, with every other field defaulted.
        let json = r#"{ "theme": "dark" }"#;
        let s: Settings = serde_json::from_str(json).expect("partial settings deserialize");
        assert_eq!(s.theme, Theme::Dark);
        assert_eq!(s.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(s.reading, ReadingStyle::default());
    }

    #[test]
    fn settings_ignores_unknown_fields() {
        // A settings file from a newer build with extra fields still loads.
        let json = r#"{ "theme": "sepia", "future_flag": true }"#;
        let s: Settings = serde_json::from_str(json).expect("forward-compatible deserialize");
        assert_eq!(s.theme, Theme::Sepia);
    }

    #[test]
    fn book_id_round_trips_through_serde() {
        let id = BookId::from_content(b"x");
        let json = serde_json::to_string(&id).expect("serialize");
        // A newtype struct serializes transparently as the inner string.
        assert_eq!(json, format!("\"{}\"", id.as_str()));
        let back: BookId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(id, back);
    }
}
