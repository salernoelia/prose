//! WebDAV implementation of the [`RemoteStore`] port.
//!
//! Uses `reqwest` (blocking) with HTTP Basic auth and TLS. All network I/O
//! is sync and must be called from within `spawn_blocking` on the IPC layer.
//! TLS 1.2+ is enforced by the underlying `rustls` stack in reqwest.

use quick_xml::events::Event;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;

use crate::domain::error::DomainError;
use crate::domain::ports::{RemoteEntry, RemoteStore};

/// WebDAV remote store using HTTP Basic authentication.
pub struct WebDavRemoteStore {
    base_url: String,
    username: String,
    password: String,
    client: std::sync::Mutex<Option<Client>>,
}

impl WebDavRemoteStore {
    pub fn new(
        base_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let client = Client::builder()
            .build()
            .map_err(|e| DomainError::Remote(e.to_string()))?;
        Ok(WebDavRemoteStore {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            username: username.into(),
            password: password.into(),
            client: std::sync::Mutex::new(Some(client)),
        })
    }

    fn client(&self) -> Result<Client, DomainError> {
        self.client
            .lock()
            .map_err(|e| DomainError::Remote(format!("Mutex error: {e}")))?
            .clone()
            .ok_or_else(|| DomainError::Remote("Client dropped".into()))
    }

    fn resolve_safe_url(&self, path: &str) -> Result<String, DomainError> {
        let normalized = path.replace('\\', "/");

        // 1. Prevent path traversal
        if normalized.contains("..")
            || normalized.contains("%2e%2e")
            || normalized.contains("%2E%2E")
        {
            return Err(DomainError::Remote(
                "Access denied: path traversal detected".into(),
            ));
        }

        // 2. Parse path part. If it starts with scheme, check it's within base_url.
        let path_part = if normalized.starts_with("http://") || normalized.starts_with("https://") {
            if !normalized.starts_with(&self.base_url) {
                return Err(DomainError::Remote(
                    "Access denied: URL is outside the configured base URL".into(),
                ));
            }
            let parsed_url = reqwest::Url::parse(&normalized)
                .map_err(|e| DomainError::Remote(format!("Invalid URL: {e}")))?;
            parsed_url.path().to_string()
        } else {
            normalized
        };

        // 3. Split into segments
        let segments: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();

        // 4. Find the "prose" segment
        let prose_index = segments.iter().position(|&s| s == "prose").ok_or_else(|| {
            DomainError::Remote("Access denied: path must be inside the prose/ folder".into())
        })?;

        // 5. Reconstruct relative path starting with prose
        let mut safe_relative_path = segments[prose_index..].join("/");
        if path.ends_with('/') && !safe_relative_path.is_empty() {
            safe_relative_path.push('/');
        }

        // 6. Strip trailing "prose" from base_url to avoid doubling
        let mut base = self.base_url.clone();
        if base.ends_with("/prose") {
            base.truncate(base.len() - 6);
        } else if base.ends_with("prose") {
            base.truncate(base.len() - 5);
        }
        let base = base.trim_end_matches('/');

        Ok(format!("{base}/{safe_relative_path}"))
    }

    /// Ensure a collection (directory) exists, creating it if needed.
    pub fn ensure_collection(&self, path: &str) -> Result<(), DomainError> {
        let url = self.resolve_safe_url(path)?;
        let resp = self
            .client()?
            .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .map_err(|e| DomainError::Remote(e.to_string()))?;
        // 201 = created, 405 = already exists - both are fine.
        if resp.status().is_success() || resp.status() == 405 {
            Ok(())
        } else {
            Err(DomainError::Remote(format!(
                "MKCOL {path} failed: {}",
                resp.status()
            )))
        }
    }
}

impl Drop for WebDavRemoteStore {
    fn drop(&mut self) {
        if let Ok(mut lock) = self.client.lock() {
            if let Some(client) = lock.take() {
                // Drop reqwest blocking Client in standard background thread to avoid tokio runtime dropping panics
                let _ = std::thread::spawn(move || {
                    drop(client);
                });
            }
        }
    }
}

impl WebDavRemoteStore {
    /// Run a PROPFIND at the given `Depth` and parse the multistatus body.
    /// A 404 means the collection does not exist yet, reported as empty.
    fn propfind(&self, dir: &str, depth: &str) -> Result<Vec<RemoteEntry>, DomainError> {
        let url = self.resolve_safe_url(dir)?;
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:href/>
    <D:getetag/>
    <D:getcontenttype/>
  </D:prop>
</D:propfind>"#;

        let resp = self
            .client()?
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .header(CONTENT_TYPE, "application/xml")
            .header("Depth", depth)
            .body(body)
            .send()
            .map_err(|e| DomainError::Remote(e.to_string()))?;

        if !resp.status().is_success() {
            // 404 on the collection means it does not exist yet - return empty.
            if resp.status() == 404 {
                return Ok(vec![]);
            }
            return Err(DomainError::Remote(format!(
                "PROPFIND {dir} failed: {}",
                resp.status()
            )));
        }

        let xml = resp
            .text()
            .map_err(|e| DomainError::Remote(e.to_string()))?;
        Ok(parse_propfind(&xml, dir))
    }
}

impl RemoteStore for WebDavRemoteStore {
    fn list(&self, dir: &str) -> Result<Vec<RemoteEntry>, DomainError> {
        self.propfind(dir, "1")
    }

    /// One recursive PROPFIND returns every file etag under `dir`. Servers that
    /// forbid infinite depth answer with an error status, surfaced here as
    /// `Err` so the sync layer can fall back to per-folder listings.
    fn list_tree(&self, dir: &str) -> Result<Vec<RemoteEntry>, DomainError> {
        self.propfind(dir, "infinity")
    }

    fn download(&self, path: &str) -> Result<Vec<u8>, DomainError> {
        let url = self.resolve_safe_url(path)?;
        let resp = self
            .client()?
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .map_err(|e| DomainError::Remote(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(DomainError::Remote(format!(
                "GET {path} failed: {}",
                resp.status()
            )));
        }

        resp.bytes()
            .map(|b| b.to_vec())
            .map_err(|e| DomainError::Remote(e.to_string()))
    }

    fn upload(&self, path: &str, bytes: &[u8]) -> Result<(), DomainError> {
        let url = self.resolve_safe_url(path)?;
        let resp = self
            .client()?
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .body(bytes.to_vec())
            .send()
            .map_err(|e| DomainError::Remote(e.to_string()))?;

        if resp.status().is_success() || resp.status() == 201 || resp.status() == 204 {
            Ok(())
        } else {
            Err(DomainError::Remote(format!(
                "PUT {path} failed: {}",
                resp.status()
            )))
        }
    }

    fn delete(&self, path: &str) -> Result<(), DomainError> {
        let url = self.resolve_safe_url(path)?;
        let resp = self
            .client()?
            .request(reqwest::Method::DELETE, &url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .map_err(|e| DomainError::Remote(e.to_string()))?;

        if resp.status().is_success() || resp.status() == 404 {
            Ok(())
        } else {
            Err(DomainError::Remote(format!(
                "DELETE {path} failed: {}",
                resp.status()
            )))
        }
    }
}

/// Parse a PROPFIND multistatus XML body and return one [`RemoteEntry`] per
/// non-collection response. The `dir` prefix is used to skip the collection
/// entry itself (servers include it as the first response).
fn parse_propfind(xml: &str, dir: &str) -> Vec<RemoteEntry> {
    let mut entries = Vec::new();
    let mut reader = quick_xml::Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut current_href: Option<String> = None;
    let mut current_etag: Option<String> = None;
    let mut in_href = false;
    let mut in_etag = false;
    let mut is_collection = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = std::str::from_utf8(e.local_name().into_inner()).unwrap_or("");
                match local {
                    "href" => in_href = true,
                    "getetag" => in_etag = true,
                    "collection" => is_collection = true,
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().into_owned();
                if in_href {
                    current_href = Some(text);
                } else if in_etag {
                    current_etag = Some(text.trim_matches('"').to_string());
                }
            }
            Ok(Event::End(ref e)) => {
                let local = std::str::from_utf8(e.local_name().into_inner()).unwrap_or("");
                match local {
                    "href" => in_href = false,
                    "getetag" => in_etag = false,
                    "response" => {
                        if let Some(href) = current_href.take() {
                            // Skip the directory entry itself and sub-collections.
                            let normalized = href.trim_end_matches('/');
                            let dir_norm = dir.trim_end_matches('/');
                            if !is_collection && !normalized.ends_with(dir_norm) {
                                entries.push(RemoteEntry {
                                    path: href,
                                    etag: current_etag.take(),
                                });
                            }
                        }
                        current_etag = None;
                        is_collection = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_propfind_extracts_file_entries() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:">
  <D:response>
    <D:href>/prose/progress/</D:href>
    <D:propstat>
      <D:prop><D:resourcetype><D:collection/></D:resourcetype></D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
  <D:response>
    <D:href>/prose/progress/abc123.json</D:href>
    <D:propstat>
      <D:prop>
        <D:getetag>"etag-abc"</D:getetag>
        <D:resourcetype/>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>"#;

        let entries = parse_propfind(xml, "prose/progress/");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "/prose/progress/abc123.json");
        assert_eq!(entries[0].etag.as_deref(), Some("etag-abc"));
    }

    #[test]
    fn resolve_safe_url_enforces_boundaries_and_detects_traversal() {
        let store = WebDavRemoteStore::new("https://example.com/webdav", "user", "pass").unwrap();

        // 1. Simple relative path
        assert_eq!(
            store.resolve_safe_url("prose/settings.json").unwrap(),
            "https://example.com/webdav/prose/settings.json"
        );

        // 2. Relative path with trailing slash
        assert_eq!(
            store.resolve_safe_url("prose/books/").unwrap(),
            "https://example.com/webdav/prose/books/"
        );

        // 3. Absolute path on same server/endpoint
        assert_eq!(
            store
                .resolve_safe_url("/webdav/prose/books/123.epub")
                .unwrap(),
            "https://example.com/webdav/prose/books/123.epub"
        );

        // 4. Absolute URL matching base URL
        assert_eq!(
            store
                .resolve_safe_url("https://example.com/webdav/prose/books/123.epub")
                .unwrap(),
            "https://example.com/webdav/prose/books/123.epub"
        );

        // 5. Base URL ending with prose/ itself
        let store_prose =
            WebDavRemoteStore::new("https://example.com/webdav/prose", "user", "pass").unwrap();
        assert_eq!(
            store_prose.resolve_safe_url("prose/settings.json").unwrap(),
            "https://example.com/webdav/prose/settings.json"
        );

        // 6. Path traversal rejection
        assert!(store.resolve_safe_url("prose/../../etc/passwd").is_err());
        assert!(store
            .resolve_safe_url("prose/%2e%2e/%2e%2e/etc/passwd")
            .is_err());

        // 7. Path outside prose folder rejection
        assert!(store.resolve_safe_url("outside/settings.json").is_err());

        // 8. Attacker URL rejection
        assert!(store
            .resolve_safe_url("https://attacker.com/prose/settings.json")
            .is_err());
    }
}
