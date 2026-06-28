//! The domain core: platform-independent library, reading, annotation,
//! settings, and sync logic.
//!
//! Nothing here touches Tauri, reqwest, rusqlite, or the filesystem. Everything
//! external is reached through a port (see `ports.rs` as it lands), so the core
//! compiles once and is unit-testable without a UI, disk, or network.

pub mod error;

pub use error::DomainError;

#[cfg(test)]
mod tests {
    #[test]
    fn core_builds() {
        let sum = 1 + 1;
        assert_eq!(sum, 2);
    }
}
