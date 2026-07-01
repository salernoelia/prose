use tauri::{AppHandle, Emitter, Manager};

use crate::domain::model::{BookId, Format};
use crate::ipc::dto::{BookDto, LibraryEntryDto, LibraryQueryDto};
use crate::ipc::error::AppError;
use crate::ipc::event::{ImportProgressPayload, IMPORT_PROGRESS, LIBRARY_CHANGED};
use crate::state::AppState;

/// Resolve an incoming location into a real filesystem path.
///
/// Desktop dialogs hand back plain paths, but iOS (and `file://` open-in-place
/// URLs) return a percent-encoded `file://` URL. `PathBuf::from` does not decode
/// those, so parse them as a URL and convert to a path first.
fn resolve_file_path(path: &str) -> Result<std::path::PathBuf, AppError> {
    if path.starts_with("file://") {
        return tauri::Url::parse(path)
            .ok()
            .and_then(|url| url.to_file_path().ok())
            .ok_or_else(|| AppError {
                code: "invalid_path".to_string(),
                message: format!("Could not resolve file URL: {}", path),
            });
    }

    Ok(std::path::PathBuf::from(path))
}

/// Read the picked file into memory along with its detected format.
///
/// Desktop and iOS dialogs return real paths (or `file://` URLs) that `std::fs`
/// can open directly. Android's Storage Access Framework instead returns a
/// `content://` URI, which `std::fs` cannot open: it must be resolved through
/// the Android content resolver. The fs plugin's `open` handles both, resolving
/// `content://` URIs to a file descriptor, so route those through it.
async fn read_book_bytes(app: &AppHandle, path: &str) -> Result<(Vec<u8>, Format), AppError> {
    // `content://` URIs (Android SAF) have no usable filesystem path or
    // extension, so read them via the fs plugin and detect the format by
    // sniffing the file's magic bytes.
    if path.starts_with("content://") {
        use std::io::Read;
        use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

        let file_path = path
            .parse::<FilePath>()
            .map_err(|e| AppError::from_internal(e.to_string()))?;
        // Resolving the content URI to a file descriptor is a quick platform
        // call; the actual read is blocking, so hand the descriptor to a
        // blocking task like the filesystem path below.
        let file = app
            .fs()
            .open(file_path, OpenOptions::new().read(true).clone())
            .map_err(|e| AppError::from_internal(format!("Failed to open file: {}", e)))?;

        let bytes = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<u8>, AppError> {
            let mut file = file;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)
                .map_err(|e| AppError::from_internal(format!("Failed to read file: {}", e)))?;
            Ok(bytes)
        })
        .await
        .map_err(|e| AppError::from_internal(e.to_string()))??;

        let format = detect_format(&bytes, None)?;
        return Ok((bytes, format));
    }

    let path_buf = resolve_file_path(path)?;
    let ext = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    tauri::async_runtime::spawn_blocking(move || -> Result<(Vec<u8>, Format), AppError> {
        let bytes = read_file_bytes(&path_buf)?;
        let format = detect_format(&bytes, ext.as_deref())?;
        Ok((bytes, format))
    })
    .await
    .map_err(|e| AppError::from_internal(e.to_string()))?
}

/// Read a filesystem path into memory.
///
/// On most platforms this is a plain `std::fs::read`. iOS is the exception:
/// files handed to us through Share or "Open in place" live outside our sandbox
/// behind a security-scoped URL. `std::fs::read` fails with a permission error
/// until the scope is opened, so route iOS reads through Foundation, which knows
/// how to claim that scope. (Android never reaches here; its `content://` URIs
/// are read earlier through the fs plugin.)
#[cfg(not(target_os = "ios"))]
fn read_file_bytes(path: &std::path::Path) -> Result<Vec<u8>, AppError> {
    if !path.exists() {
        return Err(AppError {
            code: "file_not_found".to_string(),
            message: format!("File not found at: {}", path.display()),
        });
    }

    std::fs::read(path).map_err(|e| AppError::from_internal(format!("Failed to read file: {}", e)))
}

#[cfg(target_os = "ios")]
fn read_file_bytes(path: &std::path::Path) -> Result<Vec<u8>, AppError> {
    use objc2_foundation::{NSData, NSString, NSURL};

    objc2::rc::autoreleasepool(|_| {
        let ns_path = NSString::from_str(&path.to_string_lossy());
        let url = NSURL::fileURLWithPath(&ns_path);

        // Claim the security scope before reading. It returns `false` for files
        // already inside our sandbox (e.g. copied into Documents/Inbox), which
        // read fine without it, so a `false` is not itself an error: only stop
        // accessing when we actually started.
        let scoped = unsafe { url.startAccessingSecurityScopedResource() };
        let data = NSData::dataWithContentsOfURL(&url);
        if scoped {
            unsafe { url.stopAccessingSecurityScopedResource() };
        }

        match data {
            Some(data) => Ok(data.to_vec()),
            None => Err(AppError {
                code: "file_not_found".to_string(),
                message: format!("Could not read file at: {}", path.display()),
            }),
        }
    })
}

/// Determine the book format from the file extension when available, falling
/// back to the file's magic bytes (needed for Android `content://` URIs, which
/// carry no extension): PDFs begin with `%PDF`, ePubs are ZIP archives (`PK\x03\x04`).
fn detect_format(bytes: &[u8], ext: Option<&str>) -> Result<Format, AppError> {
    match ext {
        Some("epub") => return Ok(Format::Epub),
        Some("pdf") => return Ok(Format::Pdf),
        _ => {}
    }

    if bytes.starts_with(b"%PDF") {
        Ok(Format::Pdf)
    } else if bytes.starts_with(b"PK\x03\x04") {
        Ok(Format::Epub)
    } else {
        Err(AppError {
            code: "invalid_format".to_string(),
            message: "Unsupported file format. Only ePub and PDF are supported.".to_string(),
        })
    }
}

/// On-disk extension for a stored book, mirroring `protocol::stored_file_path`.
fn stored_extension(format: Format) -> &'static str {
    match format {
        Format::Epub => "epub",
        Format::Pdf => "pdf",
    }
}

pub async fn import_book_from_path(app: &AppHandle, path: String) -> Result<BookDto, AppError> {
    let state = app.state::<AppState>();

    app.emit(
        IMPORT_PROGRESS,
        ImportProgressPayload {
            message: "Reading file...".to_string(),
            fraction: 0.1,
        },
    )
    .map_err(|e| AppError::from_internal(e.to_string()))?;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::from_internal(e.to_string()))?;

    // Desktop and iOS hand back real paths or `file://` URLs; Android's Storage
    // Access Framework hands back a `content://` URI that std::fs cannot open.
    // `read_book_bytes` reads the file into memory through the right channel for
    // each platform and detects the format.
    let (bytes, format) = read_book_bytes(app, &path).await?;

    // Persist the bytes into the library store. The on-disk name is derived from
    // the content hash and format so `protocol::stored_file_path` can locate it
    // later. We write the in-memory bytes rather than copy the source, which also
    // works for `content://` URIs that have no plain filesystem path.
    {
        let bytes = bytes.clone();
        let id = BookId::from_content(&bytes);
        let dest_path = app_data_dir.join("books").join(format!(
            "{}.{}",
            id.as_str(),
            stored_extension(format)
        ));

        tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
            std::fs::create_dir_all(dest_path.parent().unwrap()).map_err(|e| {
                AppError::from_internal(format!("Failed to create books directory: {}", e))
            })?;
            if !dest_path.exists() {
                std::fs::write(&dest_path, &bytes).map_err(|e| {
                    AppError::from_internal(format!("Failed to write file to library: {}", e))
                })?;
            }
            Ok(())
        })
        .await
        .map_err(|e| AppError::from_internal(e.to_string()))??;
    }

    app.emit(
        IMPORT_PROGRESS,
        ImportProgressPayload {
            message: "Extracting metadata...".to_string(),
            fraction: 0.7,
        },
    )
    .map_err(|e| AppError::from_internal(e.to_string()))?;

    let book = state.library.import(&bytes, format)?;

    app.emit(
        IMPORT_PROGRESS,
        ImportProgressPayload {
            message: "Done".to_string(),
            fraction: 1.0,
        },
    )
    .map_err(|e| AppError::from_internal(e.to_string()))?;

    app.emit(LIBRARY_CHANGED, ())
        .map_err(|e| AppError::from_internal(e.to_string()))?;

    Ok(BookDto::from(book))
}

#[tauri::command]
pub async fn library_import_book(app: AppHandle, path: String) -> Result<BookDto, AppError> {
    import_book_from_path(&app, path).await
}

#[tauri::command]
pub async fn library_list(
    state: tauri::State<'_, AppState>,
    query: LibraryQueryDto,
) -> Result<Vec<LibraryEntryDto>, AppError> {
    let domain_query = query.try_into()?;
    let entries = state.library.list(&domain_query)?;
    Ok(entries.into_iter().map(LibraryEntryDto::from).collect())
}

#[tauri::command]
pub async fn library_remove(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::from_internal(e.to_string()))?;

    let book_id = BookId::from_hash(&id);

    // Deletes book record from the DB
    state.library.remove(&book_id)?;

    // Clean up files in books/ and covers/
    let id_clone = id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        for ext in &["epub", "pdf"] {
            let file_path = app_data_dir
                .join("books")
                .join(format!("{}.{}", id_clone, ext));
            if file_path.exists() {
                let _ = std::fs::remove_file(file_path);
            }
        }
        for ext in &["png", "jpg"] {
            let cover_path = app_data_dir
                .join("covers")
                .join(format!("{}.{}", id_clone, ext));
            if cover_path.exists() {
                let _ = std::fs::remove_file(cover_path);
            }
        }
    })
    .await
    .map_err(|e| AppError::from_internal(e.to_string()))?;

    // Emit library changed event
    app.emit(LIBRARY_CHANGED, ())
        .map_err(|e| AppError::from_internal(e.to_string()))?;

    Ok(())
}
