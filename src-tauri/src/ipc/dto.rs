//! Boundary DTOs: the flat request and response shapes that cross the IPC seam.
//!
//! These are deliberately separate from the domain types in `crate::domain`. The
//! boundary is a contract: a DTO does not move when an internal domain type is
//! refactored. Fields are plain data, serialized camelCase to match the
//! hand-mirrored TypeScript types in `src/ipc/types.ts` (architecture section
//! 4.5). Each DTO is defined once per language.

use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;
use crate::domain::model::{Book, Format, LibraryEntry, LibraryQuery, Settings, SortKey, Theme};
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
        Theme::Dark => "dark",
        Theme::Sepia => "sepia",
    }
}

fn theme_from_str(value: &str) -> Result<Theme, DomainError> {
    match value {
        "light" => Ok(Theme::Light),
        "dark" => Ok(Theme::Dark),
        "sepia" => Ok(Theme::Sepia),
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
