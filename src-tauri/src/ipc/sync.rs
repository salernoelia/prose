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

/// A snapshot of the remote tree: normalized file path -> etag. Fetched once
/// per sync run so the stages never list a folder of their own.
type RemoteIndex = std::collections::HashMap<String, Option<String>>;

/// The full sync cycle, run on a blocking thread.
fn run_full_sync(
    store: Arc<WebDavRemoteStore>,
    repo: Arc<dyn BookRepository>,
    app_data: std::path::PathBuf,
    app: &AppHandle,
) -> Result<(), crate::domain::error::DomainError> {
    let app_state = app.state::<AppState>();

    // Ensure the directory structure exists. The "created" mark is persisted per
    // server so the seven MKCOLs do not re-run on the first sync after each
    // launch; a fresh or changed server URL has no mark and is provisioned once.
    ensure_remote_dirs(store.as_ref(), &repo, &app_state)?;

    // A single recursive listing covers every folder below, so each stage reads
    // the prefetched index instead of issuing its own PROPFIND.
    let remote_index = build_remote_index(store.as_ref())?;

    emit_progress(app, "syncing_settings", 0.0);
    sync_settings(store.as_ref(), &repo, &remote_index)?;

    emit_progress(app, "syncing_progress", 0.2);
    sync_progress(store.as_ref(), &repo, &remote_index)?;

    emit_progress(app, "syncing_bookmarks", 0.5);
    sync_bookmarks(store.as_ref(), &repo, &remote_index)?;

    emit_progress(app, "syncing_highlights", 0.7);
    sync_highlights(store.as_ref(), &repo, &remote_index)?;

    emit_progress(app, "syncing_sessions", 0.8);
    sync_sessions(store.as_ref(), &repo, &remote_index)?;

    emit_progress(app, "syncing_books", 0.9);
    sync_books(store.as_ref(), &repo, &remote_index, &app_data, app)?;

    emit_progress(app, "done", 1.0);
    Ok(())
}

/// Provision the remote folder layout, skipping the round trips when this
/// server has already been provisioned. The in-memory flag short-circuits
/// within a session; the sync-state row carries the mark across launches.
fn ensure_remote_dirs(
    store: &WebDavRemoteStore,
    repo: &Arc<dyn BookRepository>,
    app_state: &AppState,
) -> Result<(), crate::domain::error::DomainError> {
    if app_state
        .sync_dirs_created
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        return Ok(());
    }

    let url = app_state
        .sync_config
        .lock()
        .unwrap()
        .url
        .clone()
        .unwrap_or_default();
    // v2: provisioning gained prose/tombstones/sessions/, so servers marked
    // under the old key are provisioned once more to create it.
    let dirs_key = format!("state:dirs_created:v2:{url}");
    if repo.get_sync_state(&dirs_key).ok().flatten().is_some() {
        app_state
            .sync_dirs_created
            .store(true, std::sync::atomic::Ordering::Relaxed);
        return Ok(());
    }

    for dir in &[
        "prose/",
        "prose/progress/",
        "prose/bookmarks/",
        "prose/highlights/",
        "prose/sessions/",
        "prose/books/",
        "prose/tombstones/",
        "prose/tombstones/sessions/",
    ] {
        store.ensure_collection(dir)?;
    }
    let _ = repo.save_sync_state(&dirs_key, "1");
    app_state
        .sync_dirs_created
        .store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

/// Take one snapshot of the remote tree. A recursive PROPFIND returns every
/// file etag in a single request; if the server forbids infinite depth the
/// per-folder fallback preserves identical behavior at the cost of more round
/// trips. A failure to list books aborts the sync rather than mistaking an
/// unreachable server for an empty remote.
fn build_remote_index(
    store: &WebDavRemoteStore,
) -> Result<RemoteIndex, crate::domain::error::DomainError> {
    if let Ok(entries) = store.list_tree("prose/") {
        return Ok(entries
            .into_iter()
            .map(|e| (normalize_remote_path(&e.path), e.etag))
            .collect());
    }

    let mut index = RemoteIndex::new();
    for folder in &[
        "prose/",
        "prose/progress/",
        "prose/bookmarks/",
        "prose/highlights/",
        "prose/sessions/",
        "prose/tombstones/",
    ] {
        if let Ok(entries) = store.list(folder) {
            for e in entries {
                index.insert(normalize_remote_path(&e.path), e.etag);
            }
        }
    }
    for e in store.list("prose/books/")? {
        index.insert(normalize_remote_path(&e.path), e.etag);
    }
    Ok(index)
}

fn sync_settings(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
    remote_index: &RemoteIndex,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Settings;

    const REMOTE_PATH: &str = "prose/settings.json";
    const STATE_KEY: &str = "state:settings";

    let local_settings = repo.get_settings()?.unwrap_or_default();
    let local_serialized = serde_json::to_string(&local_settings).unwrap_or_default();

    let stored_state: Option<StoredSyncState> = repo
        .get_sync_state(STATE_KEY)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());

    let current_remote_etag = remote_index.get(REMOTE_PATH).cloned().flatten();
    let remote_exists = remote_index.contains_key(REMOTE_PATH);
    let stored_etag = stored_state.as_ref().and_then(|s| s.remote_etag.clone());
    let remote_etag_matches =
        remote_exists && current_remote_etag.is_some() && current_remote_etag == stored_etag;
    let stored_local = stored_state
        .as_ref()
        .map(|s| s.local_serialized.as_str())
        .unwrap_or("");

    if remote_exists && remote_etag_matches && stored_local == local_serialized {
        // Neither side moved since the last sync: no GET, no PUT.
        return Ok(());
    }

    // Pull remote only when it actually changed (or we have never seen it).
    let remote_settings: Option<Settings> = if remote_exists {
        store
            .download(REMOTE_PATH)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
    } else {
        None
    };

    // Last-write-wins by schema_version (higher version wins; same keeps local).
    let winner = match remote_settings.as_ref() {
        Some(r) if r.schema_version > local_settings.schema_version => r.clone(),
        _ => local_settings.clone(),
    };

    if winner != local_settings {
        repo.save_settings(&winner)?;
    }

    let mut new_etag = current_remote_etag.clone();
    if remote_settings.as_ref() != Some(&winner) {
        let json = serde_json::to_vec(&winner)
            .map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
        store.upload(REMOTE_PATH, &json)?;
        if let Ok(entries) = store.list(REMOTE_PATH) {
            if let Some(entry) = entries.into_iter().next() {
                new_etag = entry.etag;
            }
        }
    }

    let state_to_save = StoredSyncState {
        remote_etag: new_etag,
        local_serialized: serde_json::to_string(&winner).unwrap_or_default(),
    };
    if let Ok(serialized_state) = serde_json::to_string(&state_to_save) {
        let _ = repo.save_sync_state(STATE_KEY, &serialized_state);
    }
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
        for (i, seg) in segments.iter().enumerate().skip(pos + 1) {
            if *seg == "prose" {
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
    remote_entries: &RemoteIndex,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Progress;

    let books = repo.list_entries()?;

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
    remote_index: &RemoteIndex,
) -> Result<(), crate::domain::error::DomainError> {
    sync_collection(
        store,
        repo,
        remote_index,
        &Default::default(),
        |_| Ok(()),
        "bookmarks",
        |book_id| repo.list_bookmarks(book_id),
        |record| repo.add_bookmark(record),
    )
}

fn sync_highlights(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
    remote_index: &RemoteIndex,
) -> Result<(), crate::domain::error::DomainError> {
    sync_collection(
        store,
        repo,
        remote_index,
        &Default::default(),
        |_| Ok(()),
        "highlights",
        |book_id| repo.list_highlights(book_id),
        |record| repo.add_highlight(record),
    )
}

/// The remote folder holding one empty tombstone file per deleted session, kept
/// apart from the book tombstones so `sync_books` never mistakes one for a book.
const SESSION_TOMBSTONE_DIR: &str = "prose/tombstones/sessions/";

fn sync_sessions(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
    remote_index: &RemoteIndex,
) -> Result<(), crate::domain::error::DomainError> {
    // Deletions known to the remote, plus this device's own pending ones. The
    // merged set filters every merge below, so a deleted session can never be
    // resurrected from a stale remote file or another device's copy.
    let mut deleted: std::collections::HashSet<String> = remote_index
        .keys()
        .filter(|p| p.starts_with(SESSION_TOMBSTONE_DIR))
        .filter_map(|p| {
            std::path::Path::new(p)
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .filter(|id| !id.is_empty())
        .collect();

    // Push local deletions, then forget them: once the tombstone is on the
    // remote it carries the deletion for every device (same one-shot rule as
    // book tombstones). A failed upload keeps the local record for retry.
    for id in repo.get_deleted_sessions()? {
        let propagated = deleted.contains(&id)
            || store
                .upload(&format!("{}{}", SESSION_TOMBSTONE_DIR, id), &[])
                .is_ok();
        if propagated {
            let _ = repo.remove_deleted_session(&id);
        }
        deleted.insert(id);
    }

    sync_collection(
        store,
        repo,
        remote_index,
        &deleted,
        |id| repo.delete_reading_session(id),
        "sessions",
        |book_id| repo.list_reading_sessions(book_id),
        |record| repo.add_reading_session(record),
    )
}

/// Sync one id-keyed collection (bookmarks, highlights, sessions) for every local book.
///
/// The shape is identical across kinds: for each book take the etag fast path
/// when nothing changed, merge by id when remote moved, and refresh the stored
/// sync state. The remote folder etags come from the prefetched `remote_entries`
/// snapshot, so no per-kind PROPFIND is issued here. `kind` names the remote
/// subfolder and the sync-state key; `load` and `store_local` bind the kind's
/// repository methods.
///
/// `deleted` holds tombstoned record ids: they are dropped from the local store
/// (through `remove_local`), filtered out of every merge, and never written back
/// to the remote. Kinds without deletion support pass an empty set.
#[allow(clippy::too_many_arguments)]
fn sync_collection<T>(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
    remote_entries: &RemoteIndex,
    deleted: &std::collections::HashSet<String>,
    remove_local: impl Fn(&str) -> Result<(), crate::domain::error::DomainError>,
    kind: &str,
    load: impl Fn(&crate::domain::model::BookId) -> Result<Vec<T>, crate::domain::error::DomainError>,
    store_local: impl Fn(&T) -> Result<(), crate::domain::error::DomainError>,
) -> Result<(), crate::domain::error::DomainError>
where
    T: Syncable + Clone + Serialize + serde::de::DeserializeOwned,
{
    let books = repo.list_entries()?;

    for entry in &books {
        let book_id = &entry.book.id;
        let remote_path = format!("prose/{}/{}.json", kind, book_id.as_str());

        let mut local = load(book_id)?;
        if !deleted.is_empty() {
            // Apply tombstones from other devices before anything is compared:
            // the drop changes the local serialization, which also breaks the
            // unchanged fast path below so the deletion is processed.
            for record in local.iter().filter(|r| deleted.contains(r.sync_id())) {
                let _ = remove_local(record.sync_id());
            }
            local.retain(|r| !deleted.contains(r.sync_id()));
        }
        let local_serialized = canonical_serialized(&local);

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

            let mut winner = merge_by_id(&local, &remote);
            // Never merge a tombstoned record back in from the remote copy.
            if !deleted.is_empty() {
                winner.retain(|r| !deleted.contains(r.sync_id()));
            }
            let winner_serialized = canonical_serialized(&winner);

            // Write locally only if the merge changed our local set.
            if winner_serialized != local_serialized {
                for record in &winner {
                    let _ = store_local(record);
                }
            }

            // Upload only if the merge changed the remote set.
            if winner_serialized != canonical_serialized(&remote) {
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

        let state_to_save = StoredSyncState {
            remote_etag: new_etag,
            local_serialized: canonical_serialized(&winner),
        };
        if let Ok(serialized_state) = serde_json::to_string(&state_to_save) {
            let _ = repo.save_sync_state(&state_key, &serialized_state);
        }
    }
    Ok(())
}

/// A stable, order-independent serialization of a syncable collection, keyed by
/// sync id. The repository returns records in a kind-specific order (sessions by
/// start time) while [`merge_by_id`] returns them id-sorted, so a plain
/// serialize would differ on order alone and defeat the unchanged-since-last-sync
/// fast path. Sorting by id first makes both sides comparable.
fn canonical_serialized<T: Syncable + Serialize>(items: &[T]) -> String {
    let mut ordered: Vec<&T> = items.iter().collect();
    ordered.sort_by(|a, b| a.sync_id().cmp(b.sync_id()));
    serde_json::to_string(&ordered).unwrap_or_default()
}

fn sync_books(
    store: &dyn RemoteStore,
    repo: &Arc<dyn BookRepository>,
    remote_index: &RemoteIndex,
    app_data: &std::path::Path,
    app: &AppHandle,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Format;

    // 1. Read remote tombstones and remote books from the prefetched index.
    // Session tombstones live in a subfolder and belong to `sync_sessions`.
    let mut remote_deleted_ids: std::collections::HashSet<String> = remote_index
        .keys()
        .filter(|p| p.starts_with("prose/tombstones/") && !p.starts_with(SESSION_TOMBSTONE_DIR))
        .map(|p| {
            std::path::Path::new(p)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        })
        .filter(|id| !id.is_empty())
        .collect();

    let remote_book_paths: Vec<String> = remote_index
        .keys()
        .filter(|p| p.starts_with("prose/books/"))
        .cloned()
        .collect();
    let remote_ids: std::collections::HashSet<String> = remote_book_paths
        .iter()
        .map(|p| {
            std::path::Path::new(p)
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
    let tombstones_to_process: Vec<String> = remote_deleted_ids.iter().cloned().collect();
    for id_str in &tombstones_to_process {
        if local_ids.contains(id_str) {
            let state_key = format!("state:book:{}", id_str);
            let has_synced = repo
                .get_sync_state(&state_key)
                .unwrap_or_default()
                .is_some();

            if has_synced {
                // Apply the deletion locally but do not record a local tombstone:
                // the remote tombstone already carries the deletion, and a local
                // copy would be pushed back to the remote on every future sync,
                // deleting the book again if another device re-imports it.
                let book_id = crate::domain::model::BookId::from_hash(id_str);
                let _ = repo.remove_book(&book_id);
                let _ = repo.delete_sync_state(&state_key);

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
            } else {
                let remote_tombstone_path = format!("prose/tombstones/{}", id_str);
                let _ = store.delete(&remote_tombstone_path);
                remote_deleted_ids.remove(id_str);
            }
        }
    }

    // 4. Push local tombstones to the remote, then forget them. Book ids are
    // content hashes, so re-importing the same file on any device resurrects the
    // same id: a local tombstone kept past its propagation would re-delete that
    // book from the remote on every sync, and the importing device would then
    // drop its local copy on the next tombstone pass (the "book gets lost after
    // another device syncs" failure). Once the tombstone is on the remote it
    // carries the deletion for every device, so the local record has done its job.
    for id_str in &local_deleted_ids {
        let propagated = if remote_deleted_ids.contains(id_str) {
            true
        } else {
            let remote_tombstone_path = format!("prose/tombstones/{}", id_str);
            store.upload(&remote_tombstone_path, &[]).is_ok()
        };
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
        if propagated {
            let _ = repo.remove_deleted_book(id_str);
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
                if store.upload(&remote_path, &bytes).is_ok() {
                    let _ = repo.save_sync_state(&format!("state:book:{}", id_str), "synced");
                    emit_progress(app, "uploading_book", 0.9);
                }
            }
        }
    }

    // 6. Download remote books that are not present locally (and not deleted)
    for book_path in &remote_book_paths {
        let path = std::path::Path::new(book_path);
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
            if let Ok(bytes) = store.download(book_path) {
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
                if let Ok(book) = app_state.library.import(&bytes, format) {
                    imported_any = true;
                    let _ =
                        repo.save_sync_state(&format!("state:book:{}", book.id.as_str()), "synced");
                }
            }
        }
    }

    // 7. Ensure all already synced books have their sync state set
    for entry in &local_books {
        let id_str = entry.book.id.as_str();
        if remote_ids.contains(id_str) && !local_deleted_set.contains(id_str) {
            let _ = repo.save_sync_state(&format!("state:book:{}", id_str), "synced");
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

    /// Build the prefetched remote snapshot the stages now expect, the same way
    /// `run_full_sync` does, so each test sees current etags.
    fn remote_index(store: &impl RemoteStore) -> RemoteIndex {
        store
            .list_tree("prose/")
            .unwrap_or_default()
            .into_iter()
            .map(|e| (normalize_remote_path(&e.path), e.etag))
            .collect()
    }

    /// A remote store that counts uploads, so a test can assert the unchanged
    /// fast path issues no PUT.
    struct CountingStore {
        inner: crate::domain::testing::InMemoryRemoteStore,
        uploads: std::sync::atomic::AtomicUsize,
    }

    impl CountingStore {
        fn new() -> Self {
            Self {
                inner: crate::domain::testing::InMemoryRemoteStore::new(),
                uploads: std::sync::atomic::AtomicUsize::new(0),
            }
        }
        fn upload_count(&self) -> usize {
            self.uploads.load(std::sync::atomic::Ordering::Relaxed)
        }
    }

    impl RemoteStore for CountingStore {
        fn list(
            &self,
            dir: &str,
        ) -> Result<Vec<crate::domain::ports::RemoteEntry>, crate::domain::error::DomainError>
        {
            self.inner.list(dir)
        }
        fn download(&self, path: &str) -> Result<Vec<u8>, crate::domain::error::DomainError> {
            self.inner.download(path)
        }
        fn upload(
            &self,
            path: &str,
            bytes: &[u8],
        ) -> Result<(), crate::domain::error::DomainError> {
            self.uploads
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.inner.upload(path, bytes)
        }
        fn delete(&self, path: &str) -> Result<(), crate::domain::error::DomainError> {
            self.inner.delete(path)
        }
    }

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

        sync_progress(&store, &repo, &remote_index(&store)).unwrap();

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
        sync_progress(&store, &repo, &remote_index(&store)).unwrap();

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

        sync_bookmarks(&store, &repo, &remote_index(&store)).unwrap();

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
        sync_bookmarks(&store, &repo, &remote_index(&store)).unwrap();

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
        sync_sessions(&store, &repo, &remote_index(&store)).unwrap();

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
        sync_sessions(&store, &repo, &remote_index(&store)).unwrap();

        let local = repo.list_reading_sessions(&book_id).unwrap();
        assert_eq!(local.len(), 2);
        assert!(local.iter().any(|s| s.id == "s1"));
        assert!(local.iter().any(|s| s.id == "s2"));
    }

    #[test]
    fn sync_sessions_deletion_propagates_and_never_resurrects() {
        use crate::domain::model::{Book, BookId, BookMetadata, Format, ReadingSession};
        use crate::domain::testing::{InMemoryBookRepository, InMemoryRemoteStore};
        use std::sync::Arc;

        let store = InMemoryRemoteStore::new();
        let repo_a: Arc<dyn BookRepository> = Arc::new(InMemoryBookRepository::new());
        let repo_b: Arc<dyn BookRepository> = Arc::new(InMemoryBookRepository::new());

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
        repo_a.insert_book(&book).unwrap();
        repo_b.insert_book(&book).unwrap();

        // Device A records a session; both devices converge on it.
        let s1 = ReadingSession {
            id: "s1".to_string(),
            book_id: book_id.clone(),
            started_at: 1_000,
            duration_seconds: 60_000,
        };
        repo_a.add_reading_session(&s1).unwrap();
        sync_sessions(&store, &repo_a, &remote_index(&store)).unwrap();
        sync_sessions(&store, &repo_b, &remote_index(&store)).unwrap();
        assert_eq!(repo_b.list_reading_sessions(&book_id).unwrap().len(), 1);

        // Device A deletes it (what ReadingService::delete_session records) and
        // syncs: the tombstone reaches the remote and the local one is dropped.
        repo_a.delete_reading_session("s1").unwrap();
        repo_a.add_deleted_session("s1").unwrap();
        sync_sessions(&store, &repo_a, &remote_index(&store)).unwrap();
        assert!(repo_a.get_deleted_sessions().unwrap().is_empty());

        // Device B still holds the session; its sync applies the deletion
        // instead of merging its copy back onto the remote.
        sync_sessions(&store, &repo_b, &remote_index(&store)).unwrap();
        assert!(repo_b.list_reading_sessions(&book_id).unwrap().is_empty());

        // Later syncs on either device never resurrect it.
        sync_sessions(&store, &repo_a, &remote_index(&store)).unwrap();
        sync_sessions(&store, &repo_b, &remote_index(&store)).unwrap();
        assert!(repo_a.list_reading_sessions(&book_id).unwrap().is_empty());
        assert!(repo_b.list_reading_sessions(&book_id).unwrap().is_empty());

        let remote_path = format!("prose/sessions/{}.json", book_id.as_str());
        let remote: Vec<ReadingSession> =
            serde_json::from_slice(&store.download(&remote_path).unwrap()).unwrap();
        assert!(remote.is_empty(), "remote file must not keep the session");
    }

    #[test]
    fn sync_sessions_unchanged_does_not_reupload() {
        use crate::domain::model::{Book, BookId, BookMetadata, Format, ReadingSession};
        use crate::domain::testing::InMemoryBookRepository;
        use std::sync::Arc;

        let repo: Arc<dyn BookRepository> = Arc::new(InMemoryBookRepository::new());
        let store = CountingStore::new();

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

        // Insert sessions whose stored (insertion / started-at) order differs
        // from id order, the exact shape that defeated the old fast path because
        // merge_by_id re-sorts by id.
        for (id, started_at) in [("zzz", 1_000_i64), ("aaa", 2_000), ("mmm", 3_000)] {
            repo.add_reading_session(&ReadingSession {
                id: id.to_string(),
                book_id: book_id.clone(),
                started_at,
                duration_seconds: 10,
            })
            .unwrap();
        }

        // First sync pushes the sessions file up exactly once.
        sync_sessions(&store, &repo, &remote_index(&store)).unwrap();
        let after_first = store.upload_count();
        assert!(after_first >= 1);

        // Re-syncing with nothing changed must take the fast path: no PUT.
        sync_sessions(&store, &repo, &remote_index(&store)).unwrap();
        assert_eq!(
            store.upload_count(),
            after_first,
            "unchanged sessions must not re-upload"
        );
    }
}
