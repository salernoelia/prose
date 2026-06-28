use std::fs;
use std::path::{Path, PathBuf};

use lopdf::{Dictionary, Document, Object};
use prose_lib::adapters::readers::epub::EpubReader;
use prose_lib::adapters::readers::pdf::PdfReader;
use prose_lib::domain::error::DomainError;
use prose_lib::domain::model::Format;
use prose_lib::domain::ports::ReaderAdapter;

fn fixtures_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = manifest_dir.join("tests").join("fixtures");
    fs::create_dir_all(&fixtures_dir).expect("failed to create fixtures dir");
    fixtures_dir
}

fn ensure_pdf_fixture(fixtures_dir: &Path) -> PathBuf {
    let pdf_path = fixtures_dir.join("sample.pdf");
    if !pdf_path.exists() {
        let mut doc = Document::new();

        let pages_id = doc.add_object(Dictionary::new());

        let mut catalog = Dictionary::new();
        catalog.set(b"Type", Object::Name(b"Catalog".to_vec()));
        catalog.set(b"Pages", Object::Reference(pages_id));
        let catalog_id = doc.add_object(catalog);

        let mut info = Dictionary::new();
        info.set(
            b"Title",
            Object::String(
                "Test Title".as_bytes().to_vec(),
                lopdf::StringFormat::Literal,
            ),
        );
        info.set(
            b"Author",
            Object::String(
                "Test Author".as_bytes().to_vec(),
                lopdf::StringFormat::Literal,
            ),
        );
        let info_id = doc.add_object(info);

        doc.trailer.set(b"Root", Object::Reference(catalog_id));
        doc.trailer.set(b"Info", Object::Reference(info_id));

        doc.save(&pdf_path).unwrap();
    }

    pdf_path
}

#[test]
fn test_epub_metadata_extraction() {
    let fixtures_dir = fixtures_dir();
    let epub_path = fixtures_dir.join("book_sample.epub");
    assert!(
        epub_path.exists(),
        "missing fixture: tests/fixtures/book_sample.epub"
    );
    let epub_bytes = fs::read(&epub_path).unwrap();

    let mut temp_app_data = std::env::temp_dir();
    temp_app_data.push("prose_test_epub");
    let _ = fs::remove_dir_all(&temp_app_data);
    fs::create_dir_all(temp_app_data.join("covers")).unwrap();

    let reader = EpubReader::new(temp_app_data.clone());
    assert!(reader.supports(Format::Epub));
    assert!(!reader.supports(Format::Pdf));

    let meta = reader.probe(&epub_bytes).unwrap();
    assert!(!meta.title.trim().is_empty());

    let _ = fs::remove_dir_all(&temp_app_data);
}

#[test]
fn test_pdf_metadata_extraction() {
    let fixtures_dir = fixtures_dir();
    let pdf_path = ensure_pdf_fixture(&fixtures_dir);
    let pdf_bytes = fs::read(&pdf_path).unwrap();

    let mut temp_app_data = std::env::temp_dir();
    temp_app_data.push("prose_test_pdf");
    let _ = fs::remove_dir_all(&temp_app_data);
    fs::create_dir_all(temp_app_data.join("covers")).unwrap();

    let reader = PdfReader::new(temp_app_data.clone());
    assert!(reader.supports(Format::Pdf));
    assert!(!reader.supports(Format::Epub));

    let meta = reader.probe(&pdf_bytes).unwrap();
    assert_eq!(meta.title, "Test Title");
    assert_eq!(meta.author, Some("Test Author".to_string()));

    let _ = fs::remove_dir_all(&temp_app_data);
}

#[test]
fn test_epub_read_resource_whole_file_and_missing() {
    let epub_path = fixtures_dir().join("book_sample.epub");
    assert!(
        epub_path.exists(),
        "missing fixture: tests/fixtures/book_sample.epub"
    );
    let epub_bytes = fs::read(&epub_path).unwrap();
    let reader = EpubReader::new(std::env::temp_dir());

    // An empty path serves the container itself, what foliate-js loads.
    let whole = reader.read_resource(&epub_bytes, "").unwrap();
    assert_eq!(whole.mime, "application/epub+zip");
    assert_eq!(whole.bytes, epub_bytes);

    // A path the container does not hold is a clean not-found.
    let missing = reader.read_resource(&epub_bytes, "does/not/exist.xhtml");
    assert!(matches!(missing, Err(DomainError::ResourceNotFound(_))));
}

#[test]
fn test_pdf_read_resource_serves_whole_document() {
    let pdf_path = ensure_pdf_fixture(&fixtures_dir());
    let pdf_bytes = fs::read(&pdf_path).unwrap();
    let reader = PdfReader::new(std::env::temp_dir());

    let content = reader.read_resource(&pdf_bytes, "").unwrap();
    assert_eq!(content.mime, "application/pdf");
    assert_eq!(content.bytes, pdf_bytes);
}
