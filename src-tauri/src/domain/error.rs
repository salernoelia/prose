//! The domain error type shared across the core services.

use thiserror::Error;

/// Errors produced by the domain core. Variants are added as the services land;
/// the boundary maps each one to a serializable `AppError` in `ipc/error.rs`.
#[derive(Debug, Error)]
pub enum DomainError {}
