//! The `prose://` URI scheme that streams book resources to the renderer.
//!
//! Registered with `register_asynchronous_uri_scheme_protocol`, it reads from
//! the stored book file and honors `Range` headers, so foliate-js and pdf.js
//! fetch bytes directly and book content never travels through `invoke`.
//!
//! A request `prose://book/{book_id}/{resource_path}` is scoped to books the
//! library knows about: the id is looked up first, so an unknown or removed
//! book yields a 404 and nothing outside the catalog is reachable. An empty
//! resource path serves the whole stored file (the entry point for both
//! renderers); a non-empty one names a resource inside the container, resolved
//! by the format's reader adapter.

use std::path::{Path, PathBuf};

use percent_encoding::percent_decode_str;
use tauri::http::{header, Request, Response, StatusCode};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use crate::domain::model::{BookId, Format};
use crate::state::AppState;

/// The host segment every well-formed request carries, e.g.
/// `prose://book/{id}/{path}`.
const BOOK_HOST: &str = "book";

/// Handle one `prose://` request asynchronously, off the WebView thread.
pub fn handle<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    let uri = request.uri().to_string();
    let range = request
        .headers()
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    // File reading is blocking; keep it off the async executor.
    tauri::async_runtime::spawn_blocking(move || {
        let response = build_response(&app, &uri, range.as_deref());
        responder.respond(response);
    });
}

/// Resolve a request to a response, mapping every failure to a status code.
fn build_response<R: Runtime>(
    app: &tauri::AppHandle<R>,
    uri: &str,
    range: Option<&str>,
) -> Response<Vec<u8>> {
    let (book_id, resource_path) = match parse_request(uri) {
        Some(parts) => parts,
        None => return status(StatusCode::BAD_REQUEST),
    };

    let state = app.state::<AppState>();

    // Scope: only books in the catalog resolve.
    let book = match state.library.get_book(&book_id) {
        Ok(Some(book)) => book,
        Ok(None) => return status(StatusCode::NOT_FOUND),
        Err(_) => return status(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let app_data_dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(_) => return status(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let file_path = stored_file_path(&app_data_dir, &book_id, book.format);
    let bytes = match std::fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(_) => return status(StatusCode::NOT_FOUND),
    };

    let content = match state
        .library
        .read_resource(book.format, &bytes, &resource_path)
    {
        Ok(content) => content,
        Err(_) => return status(StatusCode::NOT_FOUND),
    };

    match range {
        Some(range) => range_response(content.bytes, &content.mime, range),
        None => full_response(content.bytes, &content.mime),
    }
}

/// Split `prose://book/{book_id}/{resource_path}` into its id and decoded
/// resource path. Returns `None` if the host segment is not `book` or the id is
/// missing. The id and each path segment are percent-decoded.
fn parse_request(uri: &str) -> Option<(BookId, String)> {
    let parsed: tauri::http::Uri = uri.parse().ok()?;

    // The scheme renders differently per platform (a `book` host on macOS, a
    // `prose.localhost` host with a `book` path prefix on Windows/Android), so
    // gather both into one segment list and require `book` to lead.
    let mut segments: Vec<String> = Vec::new();
    if let Some(host) = parsed.host() {
        if !host.is_empty() && host != "prose.localhost" {
            segments.push(host.to_string());
        }
    }
    segments.extend(
        parsed
            .path()
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(str::to_string),
    );

    let mut iter = segments.into_iter();
    if iter.next().as_deref() != Some(BOOK_HOST) {
        return None;
    }

    let raw_id = iter.next()?;
    let book_id = BookId::from_hash(decode(&raw_id));

    let resource_path = iter
        .map(|segment| decode(&segment))
        .collect::<Vec<_>>()
        .join("/");

    Some((book_id, resource_path))
}

fn decode(segment: &str) -> String {
    percent_decode_str(segment).decode_utf8_lossy().into_owned()
}

/// The on-disk path of a stored book: `books/{id}.{ext}`.
fn stored_file_path(app_data_dir: &Path, id: &BookId, format: Format) -> PathBuf {
    let ext = match format {
        Format::Epub => "epub",
        Format::Pdf => "pdf",
    };
    app_data_dir
        .join("books")
        .join(format!("{}.{}", id.as_str(), ext))
}

/// A complete `200 OK` body that advertises range support.
fn full_response(bytes: Vec<u8>, mime: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, bytes.len())
        .body(bytes)
        .unwrap_or_else(|_| status(StatusCode::INTERNAL_SERVER_ERROR))
}

/// A `206 Partial Content` slice for a single `bytes=start-end` range, falling
/// back to the full body when the range is unsatisfiable or malformed.
fn range_response(bytes: Vec<u8>, mime: &str, range: &str) -> Response<Vec<u8>> {
    let total = bytes.len() as u64;
    let (start, end) = match parse_range(range, total) {
        Some(bounds) => bounds,
        None => {
            return Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(header::CONTENT_RANGE, format!("bytes */{}", total))
                .body(Vec::new())
                .unwrap_or_else(|_| status(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    let slice = bytes[start as usize..=end as usize].to_vec();
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, mime)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(
            header::CONTENT_RANGE,
            format!("bytes {}-{}/{}", start, end, total),
        )
        .header(header::CONTENT_LENGTH, slice.len())
        .body(slice)
        .unwrap_or_else(|_| status(StatusCode::INTERNAL_SERVER_ERROR))
}

/// Parse a single-range `bytes=start-end` header into inclusive bounds clamped
/// to `total`. Only the first range of a set is honored. Returns `None` for a
/// malformed or unsatisfiable range.
fn parse_range(range: &str, total: u64) -> Option<(u64, u64)> {
    if total == 0 {
        return None;
    }
    let spec = range.strip_prefix("bytes=")?;
    let first = spec.split(',').next()?.trim();
    let (start_str, end_str) = first.split_once('-')?;

    let last = total - 1;
    let (start, end) = match (start_str.trim(), end_str.trim()) {
        // Suffix range: the final N bytes.
        ("", end) => {
            let suffix: u64 = end.parse().ok()?;
            if suffix == 0 {
                return None;
            }
            (total.saturating_sub(suffix), last)
        }
        // Open-ended range: from start to the end of the file.
        (start, "") => (start.parse().ok()?, last),
        // Closed range, clamped to the final byte.
        (start, end) => (start.parse().ok()?, end.parse::<u64>().ok()?.min(last)),
    };

    if start > end || start > last {
        return None;
    }
    Some((start, end))
}

/// An empty-bodied response carrying just a status code.
fn status(code: StatusCode) -> Response<Vec<u8>> {
    Response::builder()
        .status(code)
        .body(Vec::new())
        .expect("static status response is always valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_book_id_and_resource_path() {
        let (id, path) = parse_request("prose://book/abc123/OEBPS/ch1.xhtml").unwrap();
        assert_eq!(id.as_str(), "abc123");
        assert_eq!(path, "OEBPS/ch1.xhtml");
    }

    #[test]
    fn parses_empty_resource_path() {
        let (id, path) = parse_request("prose://book/abc123").unwrap();
        assert_eq!(id.as_str(), "abc123");
        assert_eq!(path, "");

        let (_, path) = parse_request("prose://book/abc123/").unwrap();
        assert_eq!(path, "");
    }

    #[test]
    fn parses_localhost_host_form() {
        let (id, path) = parse_request("http://prose.localhost/book/abc123/style.css").unwrap();
        assert_eq!(id.as_str(), "abc123");
        assert_eq!(path, "style.css");
    }

    #[test]
    fn percent_decodes_segments() {
        let (_, path) = parse_request("prose://book/abc123/a%20b/c%2Bd.css").unwrap();
        assert_eq!(path, "a b/c+d.css");
    }

    #[test]
    fn rejects_wrong_host() {
        assert!(parse_request("prose://other/abc123/x").is_none());
        assert!(parse_request("prose://book").is_none());
    }

    #[test]
    fn closed_range_is_clamped_and_inclusive() {
        assert_eq!(parse_range("bytes=0-99", 1000), Some((0, 99)));
        assert_eq!(parse_range("bytes=990-2000", 1000), Some((990, 999)));
    }

    #[test]
    fn open_and_suffix_ranges() {
        assert_eq!(parse_range("bytes=500-", 1000), Some((500, 999)));
        assert_eq!(parse_range("bytes=-100", 1000), Some((900, 999)));
    }

    #[test]
    fn only_the_first_range_is_honored() {
        assert_eq!(parse_range("bytes=0-9,20-29", 1000), Some((0, 9)));
    }

    #[test]
    fn rejects_malformed_or_unsatisfiable_ranges() {
        assert_eq!(parse_range("items=0-9", 1000), None);
        assert_eq!(parse_range("bytes=abc", 1000), None);
        assert_eq!(parse_range("bytes=500-499", 1000), None);
        assert_eq!(parse_range("bytes=1000-1001", 1000), None);
        assert_eq!(parse_range("bytes=0-0", 0), None);
    }
}
