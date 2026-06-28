//! The driving adapter: the only Rust that knows about Tauri commands.
//!
//! Thin `#[tauri::command]` wrappers call a domain service and translate the
//! domain error into the serializable boundary error. Payloads are flat DTOs,
//! never domain types. Submodules are added per command group.

pub mod dto;
pub mod error;
pub mod event;
pub mod library;
pub mod settings;
