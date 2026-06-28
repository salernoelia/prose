use lopdf::{Document, Object};
use std::io::Cursor;
use std::path::PathBuf;

use crate::domain::error::DomainError;
use crate::domain::model::{BookId, BookMetadata, Format};
use crate::domain::ports::{ReaderAdapter, ResourceContent};

/// A reader adapter that parses PDF metadata and extracts a cover thumbnail.
pub struct PdfReader {
    app_data_dir: PathBuf,
}

impl PdfReader {
    pub fn new(app_data_dir: PathBuf) -> Self {
        PdfReader { app_data_dir }
    }
}

impl ReaderAdapter for PdfReader {
    fn supports(&self, format: Format) -> bool {
        format == Format::Pdf
    }

    fn probe(&self, bytes: &[u8]) -> Result<BookMetadata, DomainError> {
        let cursor = Cursor::new(bytes);
        let doc = Document::load_from(cursor).map_err(|_| DomainError::InvalidFormat)?;

        let mut title = None;
        let mut author = None;

        if let Ok(info_obj) = doc.trailer.get(b"Info") {
            if let Ok((_, obj)) = doc.dereference(info_obj) {
                if let Ok(info_dict) = obj.as_dict() {
                    if let Ok(title_obj) = info_dict.get(b"Title") {
                        if let Ok(decoded) = lopdf::decode_text_string(title_obj) {
                            title = Some(decoded);
                        }
                    }
                    if let Ok(author_obj) = info_dict.get(b"Author") {
                        if let Ok(decoded) = lopdf::decode_text_string(author_obj) {
                            author = Some(decoded);
                        }
                    }
                }
            }
        }

        let title = title.unwrap_or_else(|| "Unknown Title".to_string());

        // Probe page count just to verify we can access it
        let _page_count = doc.get_pages().len();

        // Try to extract first JPEG image as cover
        let mut cover = None;
        let id = BookId::from_content(bytes);

        for (_id, object) in doc.objects.iter() {
            if let Ok(stream) = object.as_stream() {
                let dict = &stream.dict;

                let is_image = dict
                    .get(b"Subtype")
                    .and_then(|obj| obj.as_name())
                    .map(|name| name == b"Image")
                    .unwrap_or(false);

                if is_image {
                    let is_jpeg = if let Ok(filter) = dict.get(b"Filter") {
                        match filter {
                            Object::Name(name) => name == b"DCTDecode",
                            Object::Array(arr) => arr.iter().any(|obj| {
                                obj.as_name()
                                    .map(|name| name == b"DCTDecode")
                                    .unwrap_or(false)
                            }),
                            _ => false,
                        }
                    } else {
                        false
                    };

                    if is_jpeg {
                        if let Ok(content) = stream.decompressed_content() {
                            let cover_filename = format!("{}.jpg", id.as_str());
                            let cover_path = self.app_data_dir.join("covers").join(&cover_filename);
                            if std::fs::write(&cover_path, content).is_ok() {
                                cover = Some(format!("covers/{}", cover_filename));
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(BookMetadata {
            title,
            author,
            cover,
        })
    }

    fn read_resource(
        &self,
        bytes: &[u8],
        _resource_path: &str,
    ) -> Result<ResourceContent, DomainError> {
        // A PDF is a single file with no addressable sub-resources; pdf.js
        // fetches the whole document and reads pages with Range requests.
        Ok(ResourceContent {
            bytes: bytes.to_vec(),
            mime: "application/pdf".to_string(),
        })
    }
}
