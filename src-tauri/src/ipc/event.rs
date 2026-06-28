//! Event-name constants and payload structs emitted from the Rust core to the
//! TypeScript frontend.
//!
//! Every event the backend emits is declared here once. Commands emit events
//! through `AppHandle::emit(EVENT, payload)`. The TypeScript side listens in
//! `src/ipc/events.ts` with matching typed wrappers. Naming follows the
//! `domain:event` convention (architecture section 4.2).

use serde::Serialize;

use crate::ipc::dto::SettingsDto;

/// The settings were changed (by a local patch or a sync merge).
pub const SETTINGS_CHANGED: &str = "settings:changed";

/// The library was mutated (book imported, removed, metadata updated).
pub const LIBRARY_CHANGED: &str = "library:changed";

/// Progress update during a long-running book import.
pub const IMPORT_PROGRESS: &str = "import:progress";

/// Sync progress report (one of potentially many during a sync run).
pub const SYNC_PROGRESS: &str = "sync:progress";

/// Sync run finished (success or error).
pub const SYNC_FINISHED: &str = "sync:finished";

/// Payload for `settings:changed`. Carries the full updated settings so every
/// listening window can replace its local copy atomically.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsChangedPayload {
    pub settings: SettingsDto,
}

/// Payload for `import:progress`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProgressPayload {
    /// A human-readable status message (e.g. "Extracting metadata…").
    pub message: String,
    /// Fraction of the import completed, in `[0.0, 1.0]`.
    pub fraction: f32,
}

/// Payload for `sync:progress`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgressPayload {
    /// Which sync stage is running (e.g. "uploading", "downloading", "merging").
    pub stage: String,
    /// Fraction of the current stage completed, in `[0.0, 1.0]`.
    pub fraction: f32,
}

/// Payload for `sync:finished`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncFinishedPayload {
    /// `true` if the sync completed without errors.
    pub success: bool,
    /// Human-readable summary or error message.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_names_follow_domain_colon_event_convention() {
        // All constants must use the `domain:event` format.
        for name in [
            SETTINGS_CHANGED,
            LIBRARY_CHANGED,
            IMPORT_PROGRESS,
            SYNC_PROGRESS,
            SYNC_FINISHED,
        ] {
            assert!(
                name.contains(':'),
                "event name `{name}` must use `domain:event` format"
            );
        }
    }

    #[test]
    fn import_progress_payload_serializes_camel_case() {
        let json = serde_json::to_value(ImportProgressPayload {
            message: "Extracting".to_string(),
            fraction: 0.5,
        })
        .unwrap();
        assert!(json.get("message").is_some());
        assert!(json.get("fraction").is_some());
        // No snake_case leaking through
        assert!(json.get("Message").is_none());
    }

    #[test]
    fn sync_finished_payload_serializes_camel_case() {
        let json = serde_json::to_value(SyncFinishedPayload {
            success: true,
            message: "done".to_string(),
        })
        .unwrap();
        assert!(json.get("success").is_some());
        assert!(json.get("message").is_some());
    }

    #[test]
    fn settings_changed_payload_wraps_a_settings_dto() {
        use crate::domain::model::Settings;
        let payload = SettingsChangedPayload {
            settings: SettingsDto::from(Settings::default()),
        };
        let json = serde_json::to_value(payload).unwrap();
        let inner = json.get("settings").expect("settings key present");
        assert!(inner.get("schemaVersion").is_some());
        assert!(inner.get("theme").is_some());
    }
}
