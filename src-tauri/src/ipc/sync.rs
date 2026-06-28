//! Sync IPC commands: configure server, trigger sync, browse remote, download books.
//!
//! Network I/O runs inside `spawn_blocking` so the async Tauri runtime is never
//! blocked. Events `sync:progress` and `sync:finished` track progress for the UI.

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::adapters::webdav::WebDavRemoteStore;
use crate::domain::ports::{BookRepository, RemoteStore};
use crate::domain::sync::{merge_by_id, resolve_progress};
use crate::ipc::error::AppError;
use crate::ipc::event::{SyncFinishedPayload, SyncProgressPayload, SYNC_FINISHED, SYNC_PROGRESS};
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

    state.sync_dirs_created.store(false, std::sync::atomic::Ordering::Relaxed);

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
    state.sync_dirs_created.store(false, std::sync::atomic::Ordering::Relaxed);
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

    crate::ipc::library::import_book_from_path(&app, tmp.to_string_lossy().to_string())
        .await?;

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

        let _ = app.emit(
            SYNC_FINISHED,
            SyncFinishedPayload { success, message },
        );
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
    if !app_state.sync_dirs_created.load(std::sync::atomic::Ordering::Relaxed) {
        for dir in &[
            "prose/",
            "prose/progress/",
            "prose/bookmarks/",
            "prose/highlights/",
            "prose/books/",
        ] {
            store.ensure_collection(dir)?;
        }
        app_state.sync_dirs_created.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    emit_progress(app, "syncing_settings", 0.0);
    sync_settings(&store, &repo)?;

    emit_progress(app, "syncing_progress", 0.2);
    sync_progress(&store, &repo)?;

    emit_progress(app, "syncing_bookmarks", 0.5);
    sync_bookmarks(&store, &repo)?;

    emit_progress(app, "syncing_highlights", 0.7);
    sync_highlights(&store, &repo)?;

    emit_progress(app, "syncing_books", 0.9);
    sync_books(&store, &repo, &app_data, app)?;

    emit_progress(app, "done", 1.0);
    Ok(())
}

fn sync_settings(
    store: &WebDavRemoteStore,
    repo: &Arc<dyn BookRepository>,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Settings;

    let local_settings = repo.get_settings()?.unwrap_or_default();

    // Try to pull remote settings; if absent, upload ours and return.
    let remote_bytes = match store.download("prose/settings.json") {
        Ok(b) => b,
        Err(_) => {
            let json =
                serde_json::to_vec(&local_settings).map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
            store.upload("prose/settings.json", &json)?;
            return Ok(());
        }
    };

    let remote_settings: Settings = serde_json::from_slice(&remote_bytes)
        .unwrap_or_default();

    // Last-write-wins by schema_version (higher version wins; same keeps local).
    let winner = if remote_settings.schema_version > local_settings.schema_version {
        remote_settings
    } else {
        local_settings.clone()
    };

    repo.save_settings(&winner)?;

    let json = serde_json::to_vec(&winner).map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
    store.upload("prose/settings.json", &json)?;
    Ok(())
}

fn sync_progress(
    store: &WebDavRemoteStore,
    repo: &Arc<dyn BookRepository>,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Progress;
    use std::collections::HashSet;

    let books = repo.list_entries()?;
    
    // Fetch remote index once to avoid redundant 404 download round trips
    let remote_files: HashSet<String> = store
        .list("prose/progress/")
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.path)
        .collect();

    for entry in &books {
        let book_id = &entry.book.id;
        let remote_path = format!("prose/progress/{}.json", book_id.as_str());

        let local = repo.get_progress(book_id)?;

        let remote: Option<Progress> = if remote_files.contains(&remote_path) {
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

        // Write locally only if different from local
        if local.as_ref() != Some(&winner) {
            repo.save_progress(book_id, &winner)?;
        }

        // Upload only if different from remote
        if remote.as_ref() != Some(&winner) {
            let json = serde_json::to_vec(&winner).map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
            store.upload(&remote_path, &json)?;
        }
    }
    Ok(())
}

fn sync_bookmarks(
    store: &WebDavRemoteStore,
    repo: &Arc<dyn BookRepository>,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Bookmark;
    use std::collections::HashSet;

    let books = repo.list_entries()?;
    
    // Fetch remote index once to avoid redundant 404 download round trips
    let remote_files: HashSet<String> = store
        .list("prose/bookmarks/")
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.path)
        .collect();

    for entry in &books {
        let book_id = &entry.book.id;
        let remote_path = format!("prose/bookmarks/{}.json", book_id.as_str());

        let local = repo.list_bookmarks(book_id)?;
        let remote: Vec<Bookmark> = if remote_files.contains(&remote_path) {
            store
                .download(&remote_path)
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let merged = merge_by_id(&local, &remote);

        // Write locally only if different from local
        if merged != local {
            for bm in &merged {
                let _ = repo.add_bookmark(bm);
            }
        }

        // Upload only if different from remote
        if merged != remote {
            let json = serde_json::to_vec(&merged).map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
            store.upload(&remote_path, &json)?;
        }
    }
    Ok(())
}

fn sync_highlights(
    store: &WebDavRemoteStore,
    repo: &Arc<dyn BookRepository>,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Highlight;
    use std::collections::HashSet;

    let books = repo.list_entries()?;
    
    // Fetch remote index once to avoid redundant 404 download round trips
    let remote_files: HashSet<String> = store
        .list("prose/highlights/")
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.path)
        .collect();

    for entry in &books {
        let book_id = &entry.book.id;
        let remote_path = format!("prose/highlights/{}.json", book_id.as_str());

        let local = repo.list_highlights(book_id)?;
        let remote: Vec<Highlight> = if remote_files.contains(&remote_path) {
            store
                .download(&remote_path)
                .ok()
                .and_then(|b| serde_json::from_slice(&b).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let merged = merge_by_id(&local, &remote);

        // Write locally only if different from local
        if merged != local {
            for hl in &merged {
                let _ = repo.add_highlight(hl);
            }
        }

        // Upload only if different from remote
        if merged != remote {
            let json = serde_json::to_vec(&merged).map_err(|e| crate::domain::error::DomainError::Storage(e.to_string()))?;
            store.upload(&remote_path, &json)?;
        }
    }
    Ok(())
}

fn sync_books(
    store: &WebDavRemoteStore,
    repo: &Arc<dyn BookRepository>,
    app_data: &std::path::Path,
    app: &AppHandle,
) -> Result<(), crate::domain::error::DomainError> {
    use crate::domain::model::Format;

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

    let local_books = repo.list_entries()?;

    for entry in &local_books {
        let book = &entry.book;
        let id_str = book.id.as_str().to_string();
        let ext = match book.format {
            Format::Epub => "epub",
            Format::Pdf => "pdf",
        };
        let remote_path = format!("prose/books/{}.{}", id_str, ext);

        // Upload if not present on remote. The local file lives at the path
        // the library import creates: {app_data}/books/{book_id}.{ext}.
        if !remote_ids.contains(&id_str) {
            let local_file = app_data.join("books").join(format!("{}.{}", id_str, ext));
            if let Ok(bytes) = std::fs::read(&local_file) {
                let _ = store.upload(&remote_path, &bytes);
                emit_progress(app, "uploading_book", 0.9);
            }
        }
    }

    // Books on remote but not local are exposed via sync_list_remote /
    // sync_download_book so the user can choose what to download.
    Ok(())
}

fn uuid_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
