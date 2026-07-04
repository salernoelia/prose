//! The settings service: the single authority for reader settings.
//!
//! Rust owns settings; the UI holds a reactive copy. `get` always returns a
//! coherent, validated struct, and `patch` applies only the fields the caller
//! changed. Validation clamps out-of-range values and stamps the current schema
//! version, so an older or newer stored file is tolerated rather than trusted
//! blindly (architecture section 7).

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;
use crate::domain::model::{ReadingStyle, Settings, Theme, SETTINGS_SCHEMA_VERSION};
use crate::domain::ports::BookRepository;

/// A partial settings update: every field is optional, and only the present
/// ones change. Mirrors the shape the IPC layer accepts, but stays a domain
/// type the service validates.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SettingsPatch {
    pub theme: Option<Theme>,
    pub reading: Option<ReadingStylePatch>,
}

/// A partial update to the reading typography.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReadingStylePatch {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub margin: Option<f32>,
    pub text_align: Option<String>,
}

/// Reads and updates the one settings record.
pub struct SettingsService {
    repo: Arc<dyn BookRepository>,
}

impl SettingsService {
    pub fn new(repo: Arc<dyn BookRepository>) -> Self {
        SettingsService { repo }
    }

    /// The current settings, validated. Falls back to defaults when nothing is
    /// stored yet, so the caller always receives a usable struct.
    pub fn get(&self) -> Result<Settings, DomainError> {
        let stored = self.repo.get_settings()?.unwrap_or_default();
        Ok(validate(stored))
    }

    /// Apply a partial patch and persist the result. Only the present fields
    /// change; everything else keeps its current value. The persisted struct is
    /// validated and stamped with the current schema version.
    pub fn patch(&self, patch: &SettingsPatch) -> Result<Settings, DomainError> {
        let mut settings = self.get()?;
        if let Some(theme) = patch.theme {
            settings.theme = theme;
        }
        if let Some(reading) = &patch.reading {
            apply_reading(&mut settings.reading, reading);
        }
        let settings = validate(settings);
        self.repo.save_settings(&settings)?;
        Ok(settings)
    }
}

fn apply_reading(style: &mut ReadingStyle, patch: &ReadingStylePatch) {
    if let Some(font_family) = &patch.font_family {
        style.font_family = font_family.clone();
    }
    if let Some(font_size) = patch.font_size {
        style.font_size = font_size;
    }
    if let Some(line_height) = patch.line_height {
        style.line_height = line_height;
    }
    if let Some(margin) = patch.margin {
        style.margin = margin;
    }
    if let Some(text_align) = &patch.text_align {
        style.text_align = text_align.clone();
    }
}

/// The alignment keywords the reader accepts; anything else falls back to default.
const TEXT_ALIGNS: [&str; 4] = ["left", "justify", "center", "right"];

/// Normalize settings into a coherent, current-version struct: clamp numeric
/// fields to sane bounds, replace a blank font with the default, and stamp the
/// schema version this build writes. Tolerant of older and newer files.
fn validate(mut settings: Settings) -> Settings {
    let defaults = ReadingStyle::default();
    settings.schema_version = SETTINGS_SCHEMA_VERSION;

    if settings.reading.font_family.trim().is_empty() {
        settings.reading.font_family = defaults.font_family;
    }
    settings.reading.font_size = clamp(settings.reading.font_size, 8.0, 72.0, defaults.font_size);
    settings.reading.line_height =
        clamp(settings.reading.line_height, 1.0, 3.0, defaults.line_height);
    settings.reading.margin = clamp(settings.reading.margin, 0.0, 5.0, defaults.margin);
    if !TEXT_ALIGNS.contains(&settings.reading.text_align.as_str()) {
        settings.reading.text_align = defaults.text_align;
    }
    settings
}

/// Clamp a value to `[min, max]`, falling back to `default` if it is not finite.
fn clamp(value: f32, min: f32, max: f32, default: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::testing::InMemoryBookRepository;

    fn service() -> (Arc<InMemoryBookRepository>, SettingsService) {
        let repo = Arc::new(InMemoryBookRepository::new());
        let service = SettingsService::new(repo.clone());
        (repo, service)
    }

    #[test]
    fn get_returns_defaults_when_nothing_is_stored() {
        let (_repo, service) = service();
        assert_eq!(service.get().unwrap(), Settings::default());
    }

    #[test]
    fn patch_changes_only_the_given_fields() {
        let (_repo, service) = service();

        let patched = service
            .patch(&SettingsPatch {
                theme: Some(Theme::Dark),
                reading: None,
            })
            .unwrap();
        assert_eq!(patched.theme, Theme::Dark);
        assert_eq!(patched.reading, ReadingStyle::default());

        let patched = service
            .patch(&SettingsPatch {
                theme: None,
                reading: Some(ReadingStylePatch {
                    font_size: Some(22.0),
                    ..ReadingStylePatch::default()
                }),
            })
            .unwrap();
        // Theme from the first patch persists; only the font size moved.
        assert_eq!(patched.theme, Theme::Dark);
        assert_eq!(patched.reading.font_size, 22.0);
        assert_eq!(
            patched.reading.font_family,
            ReadingStyle::default().font_family
        );
    }

    #[test]
    fn patch_persists_across_reads() {
        let (_repo, service) = service();
        service
            .patch(&SettingsPatch {
                theme: Some(Theme::Sepia),
                reading: None,
            })
            .unwrap();
        assert_eq!(service.get().unwrap().theme, Theme::Sepia);
    }

    #[test]
    fn patch_clamps_out_of_range_values() {
        let (_repo, service) = service();
        let patched = service
            .patch(&SettingsPatch {
                theme: None,
                reading: Some(ReadingStylePatch {
                    font_size: Some(1_000.0),
                    line_height: Some(0.1),
                    ..ReadingStylePatch::default()
                }),
            })
            .unwrap();
        assert_eq!(patched.reading.font_size, 72.0);
        assert_eq!(patched.reading.line_height, 1.0);
    }

    #[test]
    fn patch_sets_text_align_and_rejects_unknown_values() {
        let (_repo, service) = service();
        let patched = service
            .patch(&SettingsPatch {
                theme: None,
                reading: Some(ReadingStylePatch {
                    text_align: Some("justify".to_string()),
                    ..ReadingStylePatch::default()
                }),
            })
            .unwrap();
        assert_eq!(patched.reading.text_align, "justify");

        let patched = service
            .patch(&SettingsPatch {
                theme: None,
                reading: Some(ReadingStylePatch {
                    text_align: Some("sideways".to_string()),
                    ..ReadingStylePatch::default()
                }),
            })
            .unwrap();
        assert_eq!(patched.reading.text_align, "left");
    }

    #[test]
    fn get_normalizes_an_off_version_stored_record() {
        let (repo, service) = service();
        // A record written by a different build: unknown version, blank font,
        // a non-finite size. The service still returns a coherent struct.
        repo.save_settings(&Settings {
            schema_version: 999,
            theme: Theme::Light,
            reading: ReadingStyle {
                font_family: "  ".to_string(),
                font_size: f32::NAN,
                line_height: 1.5,
                margin: 1.0,
                text_align: "left".to_string(),
            },
        })
        .unwrap();

        let settings = service.get().unwrap();
        assert_eq!(settings.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(
            settings.reading.font_family,
            ReadingStyle::default().font_family
        );
        assert_eq!(
            settings.reading.font_size,
            ReadingStyle::default().font_size
        );
    }
}
