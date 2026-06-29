//! Sync IPC commands: configure server, trigger sync, browse remote, download books.
//!
//! Network I/O runs inside `spawn_blocking` so the async Tauri runtime is never
//! blocked. Events `sync:progress` and `sync:finished` track progress for the UI.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::adapters::webdav::WebDavRemoteStore;
use crate::domain::ports::{BookRepository, RemoteStore};
use crate::domain::sync::{merge_by_id, resolve_progress, Syncable};
use crate::ipc::error::AppError;
use crate::ipc::event::{
    SyncFinishedPayload, SyncProgressPayload, LIBRARY_CHANGED, SYNC_FINISHED, SYNC_PROGRESS,
};
use crate::state::{AppState, SyncConfig};

const CREDENTIAL_KEY_URL: &str = "prose_webdav_url";
const CREDENTIAL_KEY_USERNAME: &str = "prose_webdav_username";
const CREDENTIAL_KEY_PASSWORD: &str = "prose_webdav_password";

// --- DTOs ---

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatusDto {
    pub configured: bool,
    pub url: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteBookDto {
    pub path: String,
    pub etag: Option<String>,
}

// --- Commands ---

/// Configure the WebDAV server. Stores the URL and username in the keychain;
/// the password never lands in Settings or SQLite (NFR-S-02).
#[tauri::command]
pub async fn sync_configure(
    url: String,
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let url = url.trim().to_string();
    let username = username.trim().to_string();

    // Validate that we can reach the server with these credentials before saving.
    {
        let url2 = url.clone();
        let username2 = username.clone();
        let password2 = password.clone();
        tauri::async_runtime::spawn_blocking(move || {
            WebDavRemoteStore::new(&url2, &username2, &password2)
                .and_then(|store| store.list("prose/"))
        })
        .await
        .map_err(|e| AppError::from_message("internal", e.to_string()))?
        .map_err(AppError::from)?;
    }

    // Persist credentials.
    state
        .credentials
        .store(CREDENTIAL_KEY_URL, &url)
        .map_err(AppError::from)?;
    state
        .credentials
        .store(CREDENTIAL_KEY_USERNAME, &username)
        .map_err(AppError::from)?;
    state
        .credentials
        .store(CREDENTIAL_KEY_PASSWORD, &password)
        .map_err(AppError::from)?;

    *state.sync_config.lock().unwrap() = SyncConfig {
        url: Some(url),
        username: Some(username),
    };

    state
        .sync_dirs_created
        .store(false, std::sync::atomic::Ordering::Relaxed);

    Ok(())
}

/// Return the current sync configuration (no secrets exposed).
#[tauri::command]
pub fn sync_status(state: State<'_, AppState>) -> SyncStatusDto {
    let cfg = state.sync_config.lock().unwrap().clone();
    SyncStatusDto {
        configured: cfg.url.is_some(),
        url: cfg.url,
        username: cfg.username,
    }
}

/// Remove the stored credentials and clear the sync configuration.
#[tauri::command]
pub fn sync_disconnect(state: State<'_, AppState>) -> Result<(), AppError> {
    let _ = state.credentials.delete(CREDENTIAL_KEY_URL);
    let _ = state.credentials.delete(CREDENTIAL_KEY_USERNAME);
    let _ = state.credentials.delete(CREDENTIAL_KEY_PASSWORD);
    *state.sync_config.lock().unwrap() = SyncConfig::default();
    state
        .sync_dirs_created
        .store(false, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

/// List book files available on the remote server (under `prose/books/`).
#[tauri::command]
pub async fn sync_list_remote(state: State<'_, AppState>) -> Result<Vec<RemoteBookDto>, AppError> {
    let (url, username, password) = get_sync_credentials(&state)?;
    let entries = tauri::async_runtime::spawn_blocking(move || {
        let store = WebDavRemoteStore::new(url, username, password)?;
        store.list("prose/books/")
    })
    .await
    .map_err(|e| AppError::from_message("internal", e.to_string()))?
    .map_err(AppError::from)?;

    Ok(entries
        .into_iter()
        .map(|e| RemoteBookDto {
            path: e.path,
            etag: e.etag,
        })
        .collect())
}

/// Download a book from the remote path and import it into the local library.
#[tauri::command]
pub async fn sync_download_book(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let (url, username, password) = get_sync_credentials(&state)?;
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let store = WebDavRemoteStore::new(url, username, password)?;
        store.download(&path)
    })
    .await
    .map_err(|e| AppError::from_message("internal", e.to_string()))?
    .map_err(AppError::from)?;

    // Re-use the library import path, writing to a temp file first.
    let tmp = std::env::temp_dir().join(format!("prose_dl_{}.tmp", uuid_now()));
    std::fs::write(&tmp, &bytes).map_err(|e| AppError::from_message("storage", e.to_string()))?;

    crate::ipc::library::import_book_from_path(&app, tmp.to_string_lossy().to_string()).await?;

    let _ = std::fs::remove_file(&tmp);
    Ok(())
}

/// Run a full sync: upload local changes, pull remote changes, resolve conflicts.
/// Fires `sync:progress` events during the run and `sync:finished` at the end.
#[tauri::command]
pub async fn sync_trigger(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    let (url, username, password) = get_sync_credentials(&state)?;

    let repo = state.library.repo();
    let _clock = state.reading.clock();
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::from_message("internal", e.to_string()))?;
    let app2 = app.clone();

    tauri::async_runtime::spawn(async move {
        let result = tauri::async_runtime::spawn_blocking(move || {
            let store = WebDavRemoteStore::new(url, username, password)?;
            let store = Arc::new(store);
            run_full_sync(store, repo, app_data, &app2)
        })
        .await;

        let (success, message) = match result {
            Ok(Ok(())) => (true, "Sync completed.".to_string()),
            Ok(Err(e)) => (false, e.to_string()),
            Err(e) => (false, e.to_string()),
        };

        let _ = app.emit(SYNC_FINISHED, SyncFinishedPayload { success, message });
    });

    Ok(())
}

// --- Internals ---

fn get_sync_credentials(state: &AppState) -> Result<(String, String, String), AppError> {
    let cfg = state.sync_config.lock().unwrap().clone();
    let url = cfg
        .url
        .ok_or_else(|| AppError::from_message("not_configured", "WebDAV server not configured"))?;
    let username = cfg.username.unwrap_or_default();
    let password = state
        .credentials
        .retrieve(CREDENTIAL_KEY_PASSWORD)
        .map_err(AppError::from)?
        .unwrap_or_default();
    Ok((url, username, password))
}

fn emit_progress(app: &AppHandle, stage: &str, fraction: f32) {
    let _ = app.emit(
        SYNC_PROGRESS,
        SyncProgressPayload {
            stage: stage.to_string(),
            fraction,
        },
    );
}

/// The full sync cycle, run on a blocking thread.
fn run_full_sync(
    store: Arc<WebDavRemoteStore>,
    repo: Arc<dyn BookRepository>,
    app_data: std::path::PathBuf,
    app: &AppHandle,
) -> Result<(), crate::domain::error::DomainError> {
    // Ensure the directory structure exists.
    let app_state = app.state::<AppState>();
    if !app_state
        .sync_dirs_created
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        for dir in &[
            "prose/",
            "prose/progress/",
            "prose/bookmarks/",
            "prose/highlights/",
            "prose/sessions/",
            "prose/books/",
            "prose/tombstones/",
        ] {
            store.ensure_collection(dir)?;
        }
        app_state
            .sync_dirs_created
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    emit_progress(app, "syncing_settings", 0.0);
    sync_settings(store.as_ref(), &repo)?;

    emit_progress(app, "syncing_progress", 0.2);
    sync_progress(store.as_ref(), &repo)?;

    emit_progress(app, "syncing_bookmarks", 0.5);
    sync_bookmarks(store.as_ref(), &repo)?;

    emit_progress(app, "syncing_highlights", 0.7);
    sync_highlights(store.as_ref(), &repo)?;

    emit_progress(app, "syncing_sessions", 0.8);
    sync_sessions(store.as_ref(), &repo)?;

    emit_progress(app, "syncing_books", 0.9);
    sync_books(store.as_ref(), &repo, &app_data, app)?;

    emit_progress(app, "done", 1.0);
    Ok(())
}

fn sync_settings(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Settings;

    let local_settings = repo.get_settings()?.unwrap_or_default();

    // Try to pull remote settings; if absent, upload ours and return.
    let remote_bytes = match store.download("prose/settings.json") {
        Ok(b) => b,
        Err(_) => {
            let json = serde_json::to_vec(&local_settings)
                .map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
            store.upload("prose/settings.json", &json)?;
            return Ok(());
        }
    };

    let remote_settings: Settings = serde_json::from_slice(&remote_bytes).unwrap_or_default();

    // Last-write-wins by schema_version (higher version wins; same keeps local).
    let winner = if remote_settings.schema_version > local_settings.schema_version {
        remote_settings
    } else {
        local_settings.clone()
    };

    repo.save_settings(&winner)?;

    let json = serde_json::to_vec(&winner)
        .map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
    store.upload("prose/settings.json", &json)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredSyncState {
    remote_etag: Option<String>,
    local_serialized: String,
}

fn normalize_remote_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let segments: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    if let Some(pos) = segments.iter().position(|&s| s == "prose") {
        let mut best_pos = pos;
        for i in (pos + 1)..segments.len() {
            if segments[i] == "prose" {
                best_pos = i;
            }
        }
        segments[best_pos..].join("/")
    } else {
        normalized
    }
}

fn sync_progress(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Progress;

    let books = repo.list_entries()?;

    // Fetch remote index once to avoid redundant 404 download round trips
    let remote_entries: std::collections::HashMap<String, Option<String>> = store
        .list("prose/progress/")
        .unwrap_or_default()
        .into_iter()
        .map(|e| (normalize_remote_path(&e.path), e.etag))
        .collect();

    for entry in &books {
        let book_id = &entry.book.id;
        let remote_path = format!("prose/progress/{}.json", book_id.as_str());

        let local = repo.get_progress(book_id)?;
        let local_serialized = serde_json::to_string(&local).unwrap_or_default();

        let state_key = format!("state:progress:{}", book_id.as_str());
        let stored_state: Option<StoredSyncState> = repo
            .get_sync_state(&state_key)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok());

        let current_remote_etag = remote_entries.get(&remote_path).cloned().flatten();
        let remote_exists = remote_entries.contains_key(&remote_path);

        let stored_etag = stored_state.as_ref().and_then(|s| s.remote_etag.clone());
        let remote_etag_matches =
            remote_exists && current_remote_etag.is_some() && current_remote_etag == stored_etag;
        let stored_local = stored_state
            .as_ref()
            .map(|s| s.local_serialized.as_str())
            .unwrap_or("");

        if remote_exists && remote_etag_matches && stored_local == local_serialized {
            // BOTH are unchanged. Fast path: do nothing!
            continue;
        }

        let mut new_etag = current_remote_etag.clone();
        let mut upload_needed = false;

        let winner = if remote_exists && remote_etag_matches {
            // Remote has not changed, but local did.
            // Since remote hasn't changed, local database is guaranteed to have all remote data,
            // so we don't need to download remote. Winner is simply local.
            if let Some(winner_val) = local.clone() {
                // Upload to remote
                let json = serde_json::to_vec(&winner_val)
                    .map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
                store.upload(&remote_path, &json)?;
                upload_needed = true;
                winner_val
            } else {
                continue;
            }
        } else {
            // Remote has changed (or this is the first sync, or remote does not exist).
            // We must download and merge (if remote exists).
            let remote: Option<Progress> = if remote_exists {
                store
                    .download(&remote_path)
                    .ok()
                    .and_then(|b| serde_json::from_slice(&b).ok())
            } else {
                None
            };

            let winner = match (local.as_ref(), remote.as_ref()) {
                (Some(l), Some(r)) => resolve_progress(l, r),
                (Some(l), None) => l.clone(),
                (None, Some(r)) => r.clone(),
                (None, None) => continue,
            };

            if local.as_ref() != Some(&winner) {
                repo.save_progress(book_id, &winner)?;
            }

            if remote.as_ref() != Some(&winner) {
                let json = serde_json::to_vec(&winner)
                    .map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
                store.upload(&remote_path, &json)?;
                upload_needed = true;
            }

            winner
        };

        if upload_needed {
            if let Ok(entries) = store.list(&remote_path) {
                if let Some(entry) = entries.into_iter().next() {
                    new_etag = entry.etag;
                }
            }
        }

        let updated_local_serialized = serde_json::to_string(&winner).unwrap_or_default();
        let state_to_save = StoredSyncState {
            remote_etag: new_etag,
            local_serialized: updated_local_serialized,
        };
        if let Ok(serialized_state) = serde_json::to_string(&state_to_save) {
            let _ = repo.save_sync_state(&state_key, &serialized_state);
        }
    }
    Ok(())
}

fn sync_bookmarks(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
) -> Result<(), crate::domain::error::DomainError> {
    sync_collection(
        store,
        repo,
        "bookmarks",
        |book_id| repo.list_bookmarks(book_id),
        |record| repo.add_bookmark(record),
    )
}

fn sync_highlights(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
) -> Result<(), crate::domain::error::DomainError> {
    sync_collection(
        store,
        repo,
        "highlights",
        |book_id| repo.list_highlights(book_id),
        |record| repo.add_highlight(record),
    )
}

fn sync_sessions(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
) -> Result<(), crate::domain::error::DomainError> {
    sync_collection(
        store,
        repo,
        "sessions",
        |book_id| repo.list_reading_sessions(book_id),
        |record| repo.add_reading_session(record),
    )
}

/// Sync one id-keyed collection (bookmarks, highlights, sessions) for every local book.
///
/// The shape is identical across kinds: list the remote folder once to avoid
/// per-book 404 round trips, then for each book take the etag fast path when
/// nothing changed, merge by id when remote moved, and refresh the stored sync
/// state. `kind` names the remote subfolder and the sync-state key; `load` and
/// `store_local` bind the kind's repository methods.
fn sync_collection<T>(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
    kind: &str,
    load: impl Fn(&crate::domain::model::BookId) -> Result<Vec<T>, crate::domain::error::DomainError>,
    store_local: impl Fn(&T) -> Result<(), crate::domain::error::DomainError>,
) -> Result<(), crate::domain::error::DomainError>
where
    T: Syncable + Clone + PartialEq + Serialize + serde::de::DeserializeOwned,
{
    let books = repo.list_entries()?;

    // Fetch remote index once to avoid redundant 404 download round trips
    let remote_entries: std::collections::HashMap<String, Option<String>> = store
        .list(&format!("prose/{kind}/"))
        .unwrap_or_default()
        .into_iter()
        .map(|e| (normalize_remote_path(&e.path), e.etag))
        .collect();

    for entry in &books {
        let book_id = &entry.book.id;
        let remote_path = format!("prose/{}/{}.json", kind, book_id.as_str());

        let local = load(book_id)?;
        let local_serialized = serde_json::to_string(&local).unwrap_or_default();

        let state_key = format!("state:{}:{}", kind, book_id.as_str());
        let stored_state: Option<StoredSyncState> = repo
            .get_sync_state(&state_key)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok());

        let current_remote_etag = remote_entries.get(&remote_path).cloned().flatten();
        let remote_exists = remote_entries.contains_key(&remote_path);

        let stored_etag = stored_state.as_ref().and_then(|s| s.remote_etag.clone());
        let remote_etag_matches =
            remote_exists && current_remote_etag.is_some() && current_remote_etag == stored_etag;
        let stored_local = stored_state
            .as_ref()
            .map(|s| s.local_serialized.as_str())
            .unwrap_or("");

        if remote_exists && remote_etag_matches && stored_local == local_serialized {
            // BOTH are unchanged. Fast path: do nothing!
            continue;
        }

        let mut new_etag = current_remote_etag.clone();
        let mut upload_needed = false;

        let winner = if remote_exists && remote_etag_matches {
            // Remote has not changed, but local did. Merged result is simply local.
            let json = serde_json::to_vec(&local)
                .map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
            store.upload(&remote_path, &json)?;
            upload_needed = true;
            local.clone()
        } else {
            // Remote has changed (or first sync, or does not exist).
            let remote: Vec<T> = if remote_exists {
                store
                    .download(&remote_path)
                    .ok()
                    .and_then(|b| serde_json::from_slice(&b).ok())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let winner = merge_by_id(&local, &remote);

            // Write locally only if different from local
            if winner != local {
                for record in &winner {
                    let _ = store_local(record);
                }
            }

            // Upload only if different from remote
            if winner != remote {
                let json = serde_json::to_vec(&winner)
                    .map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
                store.upload(&remote_path, &json)?;
                upload_needed = true;
            }

            winner
        };

        if upload_needed {
            if let Ok(entries) = store.list(&remote_path) {
                if let Some(entry) = entries.into_iter().next() {
                    new_etag = entry.etag;
                }
            }
        }

        let updated_local_serialized = serde_json::to_string(&winner).unwrap_or_default();
        let state_to_save = StoredSyncState {
            remote_etag: new_etag,
            local_serialized: updated_local_serialized,
        };
        if let Ok(serialized_state) = serde_json::to_string(&state_to_save) {
            let _ = repo.save_sync_state(&state_key, &serialized_state);
        }
    }
    Ok(())
}

fn sync_books(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
    app_data: &std::path::Path,
    app: &AppHandle,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Format;

    // 1. Fetch remote tombstones and remote books
    let remote_tombstone_entries = store.list("prose/tombstones/").unwrap_or_default();
    let remote_deleted_ids: std::collections::HashSet<String> = remote_tombstone_entries
        .iter()
        .map(|e| {
            std::path::Path::new(&e.path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        })
        .filter(|id| !id.is_empty())
        .collect();

    let remote_entries = store.list("prose/books/")?;
    let remote_ids: std::collections::HashSet<String> = remote_entries
        .iter()
        .map(|e| {
            std::path::Path::new(&e.path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();

    // 2. Fetch local books and local tombstones
    let local_books = repo.list_entries()?;
    let local_ids: std::collections::HashSet<String> = local_books
        .iter()
        .map(|entry| entry.book.id.as_str().to_string())
        .collect();

    let local_deleted_ids = repo.get_deleted_books().unwrap_or_default();
    let local_deleted_set: std::collections::HashSet<String> =
        local_deleted_ids.iter().cloned().collect();

    let mut imported_any = false;
    let mut deleted_any = false;

    // 3. Process remote tombstones on local database/files
    for id_str in &remote_deleted_ids {
        if local_ids.contains(id_str) {
            let book_id = crate::domain::model::BookId::from_hash(id_str);
            let _ = repo.remove_book(&book_id);
            let _ = repo.add_deleted_book(id_str);

            for ext in &["epub", "pdf"] {
                let file_path = app_data.join("books").join(format!("{}.{}", id_str, ext));
                if file_path.exists() {
                    let _ = std::fs::remove_file(file_path);
                }
            }
            for ext in &["png", "jpg"] {
                let cover_path = app_data.join("covers").join(format!("{}.{}", id_str, ext));
                if cover_path.exists() {
                    let _ = std::fs::remove_file(cover_path);
                }
            }
            deleted_any = true;
        }
    }

    // 4. Process local tombstones on remote server
    for id_str in &local_deleted_ids {
        if !remote_deleted_ids.contains(id_str) {
            let remote_tombstone_path = format!("prose/tombstones/{}", id_str);
            let _ = store.upload(&remote_tombstone_path, &[]);
        }
        if remote_ids.contains(id_str) {
            for ext in &["epub", "pdf"] {
                let remote_book_path = format!("prose/books/{}.{}", id_str, ext);
                let _ = store.delete(&remote_book_path);
            }
            let _ = store.delete(&format!("prose/progress/{}.json", id_str));
            let _ = store.delete(&format!("prose/bookmarks/{}.json", id_str));
            let _ = store.delete(&format!("prose/highlights/{}.json", id_str));
            let _ = store.delete(&format!("prose/sessions/{}.json", id_str));
        }
    }

    // 5. Upload local books that are not present on remote (and not deleted)
    for entry in &local_books {
        let book = &entry.book;
        let id_str = book.id.as_str().to_string();
        let ext = match book.format {
            Format::Epub => "epub",
            Format::Pdf => "pdf",
        };
        let remote_path = format!("prose/books/{}.{}", id_str, ext);

        if !remote_ids.contains(&id_str)
            && !remote_deleted_ids.contains(&id_str)
            && !local_deleted_set.contains(&id_str)
        {
            let local_file = app_data.join("books").join(format!("{}.{}", id_str, ext));
            if let Ok(bytes) = std::fs::read(&local_file) {
                let _ = store.upload(&remote_path, &bytes);
                emit_progress(app, "uploading_book", 0.9);
            }
        }
    }

    // 6. Download remote books that are not present locally (and not deleted)
    for entry in &remote_entries {
        let path = std::path::Path::new(&entry.path);
        let id_str = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        if id_str.is_empty() || (ext != "epub" && ext != "pdf") {
            continue;
        }

        if !local_ids.contains(&id_str)
            && !remote_deleted_ids.contains(&id_str)
            && !local_deleted_set.contains(&id_str)
        {
            emit_progress(app, "downloading_book", 0.9);
            if let Ok(bytes) = store.download(&entry.path) {
                let format = match ext {
                    "epub" => Format::Epub,
                    _ => Format::Pdf,
                };

                // Write locally
                let dest_path = app_data.join("books").join(format!("{}.{}", id_str, ext));
                if let Some(parent) = dest_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::write(&dest_path, &bytes) {
                    eprintln!("Failed to write downloaded book: {:?}", e);
                    continue;
                }

                // Import into database
                let app_state = app.state::<AppState>();
                if let Ok(_) = app_state.library.import(&bytes, format) {
                    imported_any = true;
                }
            }
        }
    }

    if imported_any || deleted_any {
        let _ = app.emit(LIBRARY_CHANGED, ());
    }

    Ok(())
}

fn uuid_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_remote_path() {
        assert_eq!(
            normalize_remote_path("prose/progress/123.json"),
            "prose/progress/123.json"
        );
        assert_eq!(
            normalize_remote_path("/prose/progress/123.json"),
            "prose/progress/123.json"
        );
        assert_eq!(
            normalize_remote_path("/webdav/prose/progress/123.json"),
            "prose/progress/123.json"
        );
        assert_eq!(
            normalize_remote_path("https://example.com/webdav/prose/progress/123.json"),
            "prose/progress/123.json"
        );
    }

    #[test]
    fn test_sync_progress_downloads_and_merges() {
        use crate::domain::model::{Book, BookId, BookMetadata, Format, Locator, Progress};
        use crate::domain::testing::{InMemoryBookRepository, InMemoryRemoteStore};
        use std::sync::Arc;

        let repo: Arc<dyn BookRepository> = Arc::new(InMemoryBookRepository::new());
        let store = InMemoryRemoteStore::new();

        // Create a book
        let book_id = BookId::from_content(b"test-book");
        let book = Book::new(
            book_id.clone(),
            Format::Epub,
            BookMetadata {
                title: "Test".to_string(),
                author: None,
                cover: None,
            },
        );
        repo.insert_book(&book).unwrap();

        // 1. First sync when remote is empty but local has progress
        let local_progress = Progress {
            locator: Locator::new("p", 0.5),
            updated_at: 100,
        };
        repo.save_progress(&book_id, &local_progress).unwrap();

        sync_progress(&store, &repo).unwrap();

        // Remote file should exist and contain the local progress
        let remote_path = format!("prose/progress/{}.json", book_id.as_str());
        let remote_bytes = store.download(&remote_path).unwrap();
        let remote_progress: Progress = serde_json::from_slice(&remote_bytes).unwrap();
        assert_eq!(remote_progress.locator.progression, 0.5);
        assert_eq!(remote_progress.updated_at, 100);

        // 2. Now modify progress on remote (Device A simulated)
        let remote_progress_new = Progress {
            locator: Locator::new("p", 0.8),
            updated_at: 2000000,
        };
        let new_bytes = serde_json::to_vec(&remote_progress_new).unwrap();
        store.upload(&remote_path, &new_bytes).unwrap();

        // Sync again (Device B simulated)
        sync_progress(&store, &repo).unwrap();

        // Local progress should be updated to the new remote progress
        let updated_local = repo.get_progress(&book_id).unwrap().unwrap();
        assert_eq!(updated_local.locator.progression, 0.8);
        assert_eq!(updated_local.updated_at, 2000000);
    }

    #[test]
    fn test_sync_bookmarks_downloads_and_merges() {
        use crate::domain::model::{Book, BookId, BookMetadata, Bookmark, Format, Locator};
        use crate::domain::testing::{InMemoryBookRepository, InMemoryRemoteStore};
        use std::sync::Arc;

        let repo: Arc<dyn BookRepository> = Arc::new(InMemoryBookRepository::new());
        let store = InMemoryRemoteStore::new();

        // Create a book
        let book_id = BookId::from_content(b"test-book");
        let book = Book::new(
            book_id.clone(),
            Format::Epub,
            BookMetadata {
                title: "Test".to_string(),
                author: None,
                cover: None,
            },
        );
        repo.insert_book(&book).unwrap();

        // 1. Initial local bookmarks
        let bm1 = Bookmark {
            id: "bm1".to_string(),
            book_id: book_id.clone(),
            locator: Locator::new("p", 0.1),
            created_at: 100,
        };
        repo.add_bookmark(&bm1).unwrap();

        sync_bookmarks(&store, &repo).unwrap();

        // 2. Add a remote bookmark (Device A simulated)
        let bm2 = Bookmark {
            id: "bm2".to_string(),
            book_id: book_id.clone(),
            locator: Locator::new("p", 0.2),
            created_at: 200,
        };
        let remote_bookmarks = vec![bm1.clone(), bm2.clone()];
        let remote_path = format!("prose/bookmarks/{}.json", book_id.as_str());
        let new_bytes = serde_json::to_vec(&remote_bookmarks).unwrap();
        store.upload(&remote_path, &new_bytes).unwrap();

        // Sync again (Device B simulated)
        sync_bookmarks(&store, &repo).unwrap();

        // Local bookmarks should contain both bm1 and bm2
        let local_bookmarks = repo.list_bookmarks(&book_id).unwrap();
        assert!(local_bookmarks.iter().any(|b| b.id == "bm1"));
        assert!(local_bookmarks.iter().any(|b| b.id == "bm2"));
    }

    #[test]
    fn test_sync_sessions_merges_by_id_across_devices() {
        use crate::domain::model::{Book, BookId, BookMetadata, Format, ReadingSession};
        use crate::domain::testing::{InMemoryBookRepository, InMemoryRemoteStore};
        use std::sync::Arc;

        let repo: Arc<dyn BookRepository> = Arc::new(InMemoryBookRepository::new());
        let store = InMemoryRemoteStore::new();

        let book_id = BookId::from_content(b"test-book");
        let book = Book::new(
            book_id.clone(),
            Format::Epub,
            BookMetadata {
                title: "Test".to_string(),
                author: None,
                cover: None,
            },
        );
        repo.insert_book(&book).unwrap();

        // 1. A local session, pushed up on first sync.
        let s1 = ReadingSession {
            id: "s1".to_string(),
            book_id: book_id.clone(),
            started_at: 1_000,
            duration_seconds: 60,
        };
        repo.add_reading_session(&s1).unwrap();
        sync_sessions(&store, &repo).unwrap();

        // 2. Another device records a session and uploads the merged set.
        let s2 = ReadingSession {
            id: "s2".to_string(),
            book_id: book_id.clone(),
            started_at: 2_000,
            duration_seconds: 120,
        };
        let remote_sessions = vec![s1.clone(), s2.clone()];
        let remote_path = format!("prose/sessions/{}.json", book_id.as_str());
        store
            .upload(&remote_path, &serde_json::to_vec(&remote_sessions).unwrap())
            .unwrap();

        // 3. This device syncs again and converges on both sessions.
        sync_sessions(&store, &repo).unwrap();

        let local = repo.list_reading_sessions(&book_id).unwrap();
        assert_eq!(local.len(), 2);
        assert!(local.iter().any(|s| s.id == "s1"));
        assert!(local.iter().any(|s| s.id == "s2"));
    }
}
