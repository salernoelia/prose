//! Driven adapters: concrete implementations of the domain ports.
//!
//! SQLite and filesystem storage, the per-format reader adapters, the WebDAV
//! remote, and the OS credential store live here. Each implements a trait from
//! `domain::ports`, so the core never depends on a concrete technology.
//! Submodules are added as each adapter lands.

pub mod memory;
pub mod readers;
pub mod storage;
