use epub::doc::EpubDoc;
use std::io::Cursor;
use std::path::PathBuf;

use crate::domain::error::DomainError;
use crate::domain::model::{BookId, BookMetadata, Format};
use crate::domain::ports::ReaderAdapter;

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
            let cover_path = self.app_data_dir.join("covers").join(&cover_filename);

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
}
