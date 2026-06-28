use epub::doc::EpubDoc;
use std::io::Cursor;
use std::path::PathBuf;

use crate::domain::error::DomainError;
use crate::domain::model::{BookId, BookMetadata, Format};
use crate::domain::ports::{ReaderAdapter, ResourceContent};

/// A reader adapter that parses ePub metadata and extracts cover images.
pub struct EpubReader {
    app_data_dir: PathBuf,
}

impl EpubReader {
    pub fn new(app_data_dir: PathBuf) -> Self {
        EpubReader { app_data_dir }
    }
}

impl ReaderAdapter for EpubReader {
    fn supports(&self, format: Format) -> bool {
        format == Format::Epub
    }

    fn probe(&self, bytes: &[u8]) -> Result<BookMetadata, DomainError> {
        let cursor = Cursor::new(bytes);
        let mut doc = EpubDoc::from_reader(cursor).map_err(|_| DomainError::InvalidFormat)?;

        let title = doc
            .mdata("title")
            .map(|val| val.value.clone())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author = doc.mdata("creator").map(|val| val.value.clone());

        let cover = if let Some((cover_bytes, mime_type)) = doc.get_cover() {
            let id = BookId::from_content(bytes);
            let ext = if mime_type.contains("jpeg") || mime_type.contains("jpg") {
                "jpg"
            } else {
                "png"
            };
            let cover_filename = format!("{}.{}", id.as_str(), ext);
            let covers_dir = self.app_data_dir.join("covers");

            std::fs::create_dir_all(&covers_dir).ok();

            let cover_path = covers_dir.join(&cover_filename);

            if let Err(e) = std::fs::write(&cover_path, cover_bytes) {
                eprintln!("Failed to write cover image to {:?}: {}", cover_path, e);
                None
            } else {
                Some(format!("covers/{}", cover_filename))
            }
        } else {
            None
        };

        Ok(BookMetadata {
            title,
            author,
            cover,
        })
    }

    fn read_resource(
        &self,
        bytes: &[u8],
        resource_path: &str,
    ) -> Result<ResourceContent, DomainError> {
        // An empty path means the container itself, which foliate-js unzips.
        if resource_path.is_empty() {
            return Ok(ResourceContent {
                bytes: bytes.to_vec(),
                mime: "application/epub+zip".to_string(),
            });
        }

        let cursor = Cursor::new(bytes);
        let mut doc = EpubDoc::from_reader(cursor).map_err(|_| DomainError::InvalidFormat)?;

        let data = doc
            .get_resource_by_path(resource_path)
            .ok_or_else(|| DomainError::ResourceNotFound(resource_path.to_string()))?;
        let mime = doc
            .get_resource_mime_by_path(resource_path)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        Ok(ResourceContent { bytes: data, mime })
    }
}
