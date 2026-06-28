use tauri::{AppHandle, Emitter, Manager};

use crate::domain::model::{BookId, Format};
use crate::ipc::dto::{BookDto, LibraryEntryDto, LibraryQueryDto};
use crate::ipc::error::AppError;
use crate::ipc::event::{ImportProgressPayload, IMPORT_PROGRESS, LIBRARY_CHANGED};
use crate::state::AppState;

#[tauri::command]
pub async fn library_import_book(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<BookDto, AppError> {
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

    let path_clone = path.clone();
    let (bytes, format, _, _) =
        tauri::async_runtime::spawn_blocking(move || -> Result<_, AppError> {
            let path_buf = std::path::PathBuf::from(&path_clone);
            if !path_buf.exists() {
                return Err(AppError {
                    code: "file_not_found".to_string(),
                    message: format!("File not found at: {}", path_clone),
                });
            }

            let bytes = std::fs::read(&path_buf)
                .map_err(|e| AppError::from_internal(format!("Failed to read file: {}", e)))?;

            let id = BookId::from_content(&bytes);

            let ext = path_buf
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .ok_or_else(|| AppError {
                    code: "invalid_format".to_string(),
                    message: "Missing file extension".to_string(),
                })?;

            let format = match ext.as_str() {
                "epub" => Format::Epub,
                "pdf" => Format::Pdf,
                _ => {
                    return Err(AppError {
                        code: "invalid_format".to_string(),
                        message: "Unsupported file format. Only ePub and PDF are supported."
                            .to_string(),
                    })
                }
            };

            // Copy file to app_data/books/{hash}.{ext}
            let dest_filename = format!("{}.{}", id.as_str(), ext);
            let dest_path = app_data_dir.join("books").join(&dest_filename);

            std::fs::create_dir_all(dest_path.parent().unwrap()).map_err(|e| {
                AppError::from_internal(format!("Failed to create books directory: {}", e))
            })?;

            if !dest_path.exists() {
                std::fs::copy(&path_buf, &dest_path).map_err(|e| {
                    AppError::from_internal(format!("Failed to copy file to library: {}", e))
                })?;
            }

            Ok((bytes, format, ext, id))
        })
        .await
        .map_err(|e| AppError::from_internal(e.to_string()))??;

    app.emit(
        IMPORT_PROGRESS,
        ImportProgressPayload {
            message: "Extracting metadata...".to_string(),
            fraction: 0.7,
        },
    )
    .map_err(|e| AppError::from_internal(e.to_string()))?;

    // Call domain service to probe metadata and store to database
    let book = state.library.import(&bytes, format)?;

    app.emit(
        IMPORT_PROGRESS,
        ImportProgressPayload {
            message: "Done".to_string(),
            fraction: 1.0,
        },
    )
    .map_err(|e| AppError::from_internal(e.to_string()))?;

    // Emit library changed event
    app.emit(LIBRARY_CHANGED, ())
        .map_err(|e| AppError::from_internal(e.to_string()))?;

    Ok(BookDto::from(book))
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
