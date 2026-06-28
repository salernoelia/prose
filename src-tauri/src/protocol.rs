//! The `prose://` URI scheme that streams book resources to the renderer.
//!
//! Registered with `register_asynchronous_uri_scheme_protocol`, it reads from
//! the stored book file and honors `Range` headers, so foliate-js and pdf.js
//! fetch bytes directly and book content never travels through `invoke`.
