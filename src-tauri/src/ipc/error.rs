//! `AppError`: the serializable error that crosses the IPC boundary.
//!
//! The domain speaks in [`DomainError`] (a `thiserror` enum); the boundary never
//! leaks that type or a bare string. Every command returns `Result<_, AppError>`,
//! a `{ code, message }` pair the TypeScript side can branch on by `code`
//! (architecture section 4.4).

use serde::Serialize;

use crate::domain::error::DomainError;

/// The boundary error: a stable machine-readable `code` plus a human-readable
/// `message`. Mirrored by the `AppError` interface in `src/ipc/types.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    fn new(code: &str, message: String) -> Self {
        AppError {
            code: code.to_string(),
            message,
        }
    }

    pub fn from_internal(message: String) -> Self {
        AppError::new("internal", message)
    }
}

impl From<DomainError> for AppError {
    fn from(error: DomainError) -> Self {
        let code = match error {
            DomainError::BookNotFound(_) => "book_not_found",
            DomainError::InvalidFormat => "invalid_format",
            DomainError::NoReaderForFormat(_) => "no_reader_for_format",
            DomainError::ResourceNotFound(_) => "resource_not_found",
            DomainError::InvalidInput(_) => "invalid_input",
            DomainError::Conflict(_) => "conflict",
            DomainError::Storage(_) => "storage_error",
            DomainError::Remote(_) => "remote_error",
            DomainError::Credential(_) => "credential_error",
        };
        AppError::new(code, error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_each_domain_error_to_a_stable_code() {
        let mapped = AppError::from(DomainError::BookNotFound("abc".to_string()));
        assert_eq!(mapped.code, "book_not_found");
        assert_eq!(mapped.message, "book not found: abc");

        assert_eq!(
            AppError::from(DomainError::InvalidInput("bad".to_string())).code,
            "invalid_input"
        );
        assert_eq!(
            AppError::from(DomainError::Storage("disk".to_string())).code,
            "storage_error"
        );
    }

    #[test]
    fn serializes_to_camel_case_code_and_message() {
        let json = serde_json::to_string(&AppError::new("conflict", "clash".to_string())).unwrap();
        assert_eq!(json, r#"{"code":"conflict","message":"clash"}"#);
    }
}
