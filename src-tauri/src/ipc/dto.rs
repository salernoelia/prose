//! Boundary DTOs: the flat request and response shapes that cross the IPC seam.
//!
//! These are deliberately separate from the domain types in `crate::domain`. The
//! boundary is a contract: a DTO does not move when an internal domain type is
//! refactored. Fields are plain data, serialized camelCase to match the
//! hand-mirrored TypeScript types in `src/ipc/types.ts` (architecture section
//! 4.5). Each DTO is defined once per language.

use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;
use crate::domain::model::{
    Book, Bookmark, Definition, Format, Highlight, LibraryEntry, LibraryQuery, Locator, Progress,
    ReadingSession, Settings, SortKey, Theme,
};
use crate::domain::settings::{ReadingStylePatch, SettingsPatch};

/// The full settings, flattened: the nested reading typography is lifted to the
/// top level so the wire shape is one flat object. `theme` travels as a string
/// so the boundary does not depend on the domain enum's representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub schema_version: u32,
    pub theme: String,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub margin: f32,
}

impl From<Settings> for SettingsDto {
    fn from(settings: Settings) -> Self {
        SettingsDto {
            schema_version: settings.schema_version,
            theme: theme_to_str(settings.theme).to_string(),
            font_family: settings.reading.font_family,
            font_size: settings.reading.font_size,
            line_height: settings.reading.line_height,
            margin: settings.reading.margin,
        }
    }
}

/// A partial settings update from the UI: every field optional, only the present
/// ones change. Converted to the domain [`SettingsPatch`] at the boundary, which
/// is where an unknown `theme` string becomes an [`DomainError::InvalidInput`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatchDto {
    pub theme: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub margin: Option<f32>,
}

impl TryFrom<SettingsPatchDto> for SettingsPatch {
    type Error = DomainError;

    fn try_from(dto: SettingsPatchDto) -> Result<Self, Self::Error> {
        let theme = dto.theme.as_deref().map(theme_from_str).transpose()?;
        let reading = ReadingStylePatch {
            font_family: dto.font_family,
            font_size: dto.font_size,
            line_height: dto.line_height,
            margin: dto.margin,
        };
        // Collapse an all-absent typography patch to `None`, so a theme-only
        // update carries no reading patch at all.
        let reading = (reading != ReadingStylePatch::default()).then_some(reading);
        Ok(SettingsPatch { theme, reading })
    }
}

fn theme_to_str(theme: Theme) -> &'static str {
    match theme {
        Theme::Light => "light",
        Theme::Paper => "paper",
        Theme::Dark => "dark",
        Theme::Oled => "oled",
        Theme::Sepia => "sepia",
        Theme::SepiaDark => "sepia-dark",
        Theme::Eink => "eink",
        Theme::EinkDark => "eink-dark",
    }
}

fn theme_from_str(value: &str) -> Result<Theme, DomainError> {
    match value {
        "light" => Ok(Theme::Light),
        "paper" => Ok(Theme::Paper),
        "dark" => Ok(Theme::Dark),
        "oled" => Ok(Theme::Oled),
        "sepia" => Ok(Theme::Sepia),
        "sepia-dark" => Ok(Theme::SepiaDark),
        "eink" => Ok(Theme::Eink),
        "eink-dark" => Ok(Theme::EinkDark),
        other => Err(DomainError::InvalidInput(format!("unknown theme: {other}"))),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookDto {
    pub id: String,
    pub format: String,
    pub title: String,
    pub author: Option<String>,
    pub cover: Option<String>,
}

impl From<Book> for BookDto {
    fn from(book: Book) -> Self {
        BookDto {
            id: book.id.as_str().to_string(),
            format: match book.format {
                Format::Epub => "epub".to_string(),
                Format::Pdf => "pdf".to_string(),
            },
            title: book.metadata.title,
            author: book.metadata.author,
            cover: book.metadata.cover,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntryDto {
    pub book: BookDto,
    pub progress: f32,
    pub last_read: Option<i64>,
}

impl From<LibraryEntry> for LibraryEntryDto {
    fn from(entry: LibraryEntry) -> Self {
        LibraryEntryDto {
            book: BookDto::from(entry.book),
            progress: entry.progress,
            last_read: entry.last_read,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryQueryDto {
    pub search: Option<String>,
    pub sort: String,
    pub descending: bool,
}

impl TryFrom<LibraryQueryDto> for LibraryQuery {
    type Error = DomainError;

    fn try_from(dto: LibraryQueryDto) -> Result<Self, Self::Error> {
        let sort = match dto.sort.as_str() {
            "title" => SortKey::Title,
            "author" => SortKey::Author,
            "last_read" => SortKey::LastRead,
            "progress" => SortKey::Progress,
            other => {
                return Err(DomainError::InvalidInput(format!(
                    "unknown sort key: {other}"
                )))
            }
        };
        Ok(LibraryQuery {
            search: dto.search,
            sort,
            descending: dto.descending,
        })
    }
}

/// The format-neutral reading position, mirroring the domain [`Locator`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocatorDto {
    pub payload: String,
    pub progression: f32,
}

impl From<Locator> for LocatorDto {
    fn from(l: Locator) -> Self {
        LocatorDto {
            payload: l.payload,
            progression: l.progression,
        }
    }
}

impl From<LocatorDto> for Locator {
    fn from(d: LocatorDto) -> Self {
        Locator::new(d.payload, d.progression)
    }
}

/// The saved reading position with its timestamp, returned by position commands.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressDto {
    pub locator: LocatorDto,
    pub updated_at: i64,
}

impl From<Progress> for ProgressDto {
    fn from(p: Progress) -> Self {
        ProgressDto {
            locator: LocatorDto::from(p.locator),
            updated_at: p.updated_at,
        }
    }
}

/// A bookmark at a saved location, returned by the bookmark commands.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkDto {
    pub id: String,
    pub book_id: String,
    pub locator: LocatorDto,
    pub created_at: i64,
}

impl From<Bookmark> for BookmarkDto {
    fn from(b: Bookmark) -> Self {
        BookmarkDto {
            id: b.id,
            book_id: b.book_id.as_str().to_string(),
            locator: LocatorDto::from(b.locator),
            created_at: b.created_at,
        }
    }
}

/// A highlight over a selected text range, returned by the highlight commands.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HighlightDto {
    pub id: String,
    pub book_id: String,
    pub locator: LocatorDto,
    pub text: String,
    pub color: Option<String>,
    pub created_at: i64,
}

impl From<Highlight> for HighlightDto {
    fn from(h: Highlight) -> Self {
        HighlightDto {
            id: h.id,
            book_id: h.book_id.as_str().to_string(),
            locator: LocatorDto::from(h.locator),
            text: h.text,
            color: h.color,
            created_at: h.created_at,
        }
    }
}

/// A reading session, returned by the session commands. Aggregate statistics
/// (reading time, streaks, charts) are derived on the client from these.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingSessionDto {
    pub id: String,
    pub book_id: String,
    pub started_at: i64,
    pub duration_seconds: i64,
}

impl From<ReadingSession> for ReadingSessionDto {
    fn from(s: ReadingSession) -> Self {
        ReadingSessionDto {
            id: s.id,
            book_id: s.book_id.as_str().to_string(),
            started_at: s.started_at,
            duration_seconds: s.duration_seconds,
        }
    }
}

/// One dictionary sense, returned by the lookup command (FR-NOTE-03).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionDto {
    pub part_of_speech: String,
    pub gloss: String,
    pub synonyms: Vec<String>,
    pub examples: Vec<String>,
}

impl From<Definition> for DefinitionDto {
    fn from(d: Definition) -> Self {
        DefinitionDto {
            part_of_speech: d.part_of_speech,
            gloss: d.gloss,
            synonyms: d.synonyms,
            examples: d.examples,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::ReadingStyle;

    #[test]
    fn settings_dto_flattens_the_domain_struct() {
        let dto = SettingsDto::from(Settings::default());
        assert_eq!(dto.schema_version, Settings::default().schema_version);
        assert_eq!(dto.theme, "light");
        assert_eq!(dto.font_family, ReadingStyle::default().font_family);
        assert_eq!(dto.font_size, ReadingStyle::default().font_size);
    }

    #[test]
    fn settings_dto_serializes_camel_case() {
        let json = serde_json::to_value(SettingsDto::from(Settings::default())).unwrap();
        assert!(json.get("schemaVersion").is_some());
        assert!(json.get("fontFamily").is_some());
        assert!(json.get("font_family").is_none());
    }

    #[test]
    fn theme_only_patch_carries_no_reading_patch() {
        let patch = SettingsPatch::try_from(SettingsPatchDto {
            theme: Some("dark".to_string()),
            ..SettingsPatchDto::default()
        })
        .unwrap();
        assert_eq!(patch.theme, Some(Theme::Dark));
        assert_eq!(patch.reading, None);
    }

    #[test]
    fn typography_patch_maps_present_fields_only() {
        let patch = SettingsPatch::try_from(SettingsPatchDto {
            font_size: Some(22.0),
            ..SettingsPatchDto::default()
        })
        .unwrap();
        assert_eq!(patch.theme, None);
        let reading = patch.reading.expect("reading patch present");
        assert_eq!(reading.font_size, Some(22.0));
        assert_eq!(reading.font_family, None);
    }

    #[test]
    fn unknown_theme_is_rejected_as_invalid_input() {
        let result = SettingsPatch::try_from(SettingsPatchDto {
            theme: Some("chartreuse".to_string()),
            ..SettingsPatchDto::default()
        });
        assert!(matches!(result, Err(DomainError::InvalidInput(_))));
    }
}
