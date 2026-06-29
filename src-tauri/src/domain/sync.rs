//! The sync core: conflict resolution and the outbox, as pure types.
//!
//! This module owns the synchronization *rules*, not the engine. The background
//! task that drives them against a live server lands later (Phase 10); here the
//! logic is written against the [`RemoteStore`] port and exercised with a fake,
//! so every conflict branch is unit-tested without a network.
//!
//! Two rules resolve conflicts (FR-SYNC-04):
//! - reading position keeps the furthest progression,
//! - everything else is last-write-wins by timestamp.
//!
//! Resumability (NFR-R-03) rests on the [`Outbox`]: pending changes are keyed by
//! a stable id, so re-running an interrupted sync replays each change exactly
//! once, without loss or duplication.

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::model::{Bookmark, Highlight, Progress};
use crate::domain::ports::{RemoteEntry, RemoteStore};

/// A record that can be synchronized: it carries a stable id for idempotent
/// keying and a last-modified timestamp for last-write-wins resolution.
pub trait Syncable {
    fn sync_id(&self) -> &str;
    fn last_modified(&self) -> i64;
}

impl Syncable for Bookmark {
    fn sync_id(&self) -> &str {
        &self.id
    }
    fn last_modified(&self) -> i64 {
        self.created_at
    }
}

impl Syncable for Highlight {
    fn sync_id(&self) -> &str {
        &self.id
    }
    fn last_modified(&self) -> i64 {
        self.created_at
    }
}

/// Resolve a reading-position conflict: keep the position with the newest
/// timestamp (last-write-wins) to avoid jumps when page lengths or font sizes
/// differ across devices (FR-SYNC-04). On an exact tie the local copy is kept.
pub fn resolve_progress(local: &Progress, remote: &Progress) -> Progress {
    if remote.updated_at > local.updated_at {
        remote.clone()
    } else {
        local.clone()
    }
}

/// Resolve a conflict for any timestamped record: the most recently modified
/// wins (FR-SYNC-04). On an exact tie the local copy is kept, so a sync that
/// finds nothing newer is a no-op.
pub fn resolve_last_write<T: Syncable + Clone>(local: &T, remote: &T) -> T {
    if remote.last_modified() > local.last_modified() {
        remote.clone()
    } else {
        local.clone()
    }
}

/// Merge two collections of the same syncable kind by id, resolving each per-id
/// conflict by last-write-wins. The result holds one record per id, sorted by id
/// for a deterministic order. This is the additive merge; deletions travel as
/// explicit [`SyncOp`] entries, never inferred from a record's absence, so a
/// device that has not yet synced cannot resurrect a deleted record.
pub fn merge_by_id<T: Syncable + Clone>(local: &[T], remote: &[T]) -> Vec<T> {
    let mut by_id: HashMap<&str, &T> = HashMap::new();
    for item in local.iter().chain(remote) {
        by_id
            .entry(item.sync_id())
            .and_modify(|current| {
                if item.last_modified() > current.last_modified() {
                    *current = item;
                }
            })
            .or_insert(item);
    }
    let mut merged: Vec<T> = by_id.into_values().cloned().collect();
    merged.sort_by(|a, b| a.sync_id().cmp(b.sync_id()));
    merged
}

/// A change waiting to be uploaded to the remote.
#[derive(Debug, Clone, PartialEq)]
pub enum SyncOp {
    /// Upload the latest reading position for a book.
    PutProgress { book_id: String, progress: Progress },
    /// Upload a created bookmark.
    PutBookmark(Bookmark),
    /// Remove a bookmark from the remote.
    DeleteBookmark { id: String },
    /// Upload a created highlight.
    PutHighlight(Highlight),
    /// Remove a highlight from the remote.
    DeleteHighlight { id: String },
}

impl SyncOp {
    /// The idempotency key: the unit of work this op represents. A put and a
    /// later delete of the same record share a key, so the delete supersedes the
    /// not-yet-uploaded put rather than racing it.
    pub fn key(&self) -> String {
        match self {
            SyncOp::PutProgress { book_id, .. } => format!("progress:{book_id}"),
            SyncOp::PutBookmark(bookmark) => format!("bookmark:{}", bookmark.id),
            SyncOp::DeleteBookmark { id } => format!("bookmark:{id}"),
            SyncOp::PutHighlight(highlight) => format!("highlight:{}", highlight.id),
            SyncOp::DeleteHighlight { id } => format!("highlight:{id}"),
        }
    }
}

/// One pending operation in the outbox, paired with its idempotency key.
#[derive(Debug, Clone, PartialEq)]
pub struct OutboxEntry {
    pub key: String,
    pub op: SyncOp,
}

/// The local queue of changes not yet confirmed by the remote. Enqueue is
/// idempotent by key: re-recording the same logical change replaces the pending
/// entry rather than adding a second, so an interrupted, replayed sync neither
/// loses nor duplicates work (NFR-R-03).
#[derive(Debug, Clone, Default)]
pub struct Outbox {
    entries: Vec<OutboxEntry>,
}

impl Outbox {
    pub fn new() -> Self {
        Outbox::default()
    }

    /// Record an operation. If one with the same key is already pending it is
    /// replaced, so the newest intent for each record wins.
    pub fn enqueue(&mut self, op: SyncOp) {
        let key = op.key();
        if let Some(existing) = self.entries.iter_mut().find(|entry| entry.key == key) {
            existing.op = op;
        } else {
            self.entries.push(OutboxEntry { key, op });
        }
    }

    /// The pending entries, in enqueue order.
    pub fn pending(&self) -> &[OutboxEntry] {
        &self.entries
    }

    /// Drop an entry once the remote has confirmed it.
    pub fn ack(&mut self, key: &str) {
        self.entries.retain(|entry| entry.key != key);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Drives synchronization against a [`RemoteStore`]. In this phase it holds the
/// port and exposes the primitives the rules build on; the background engine
/// that schedules them arrives in Phase 10.
pub struct SyncService {
    remote: Arc<dyn RemoteStore>,
}

impl SyncService {
    pub fn new(remote: Arc<dyn RemoteStore>) -> Self {
        SyncService { remote }
    }

    /// List what the remote holds under `dir`, the starting point for deciding
    /// which records to pull and merge.
    pub fn remote_index(&self, dir: &str) -> Result<Vec<RemoteEntry>, DomainError> {
        self.remote.list(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{BookId, Locator};
    use crate::domain::testing::InMemoryRemoteStore;

    fn progress(progression: f32, updated_at: i64) -> Progress {
        Progress {
            locator: Locator::new("p", progression),
            updated_at,
        }
    }

    fn bookmark(id: &str, created_at: i64) -> Bookmark {
        Bookmark {
            id: id.to_string(),
            book_id: BookId::from_content(b"book"),
            locator: Locator::new("p", 0.0),
            created_at,
        }
    }

    #[test]
    fn progress_conflict_keeps_the_newest_timestamp() {
        let local = progress(0.3, 200);
        let remote = progress(0.7, 100);
        assert_eq!(resolve_progress(&local, &remote), local);
        assert_eq!(resolve_progress(&remote, &local), local);
    }

    #[test]
    fn progress_conflict_tie_keeps_local() {
        let local = progress(0.3, 100);
        let remote = progress(0.5, 100);
        assert_eq!(resolve_progress(&local, &remote), local);
    }

    #[test]
    fn last_write_keeps_the_most_recent() {
        let older = bookmark("a", 100);
        let newer = bookmark("a", 200);
        assert_eq!(resolve_last_write(&older, &newer), newer);
        assert_eq!(resolve_last_write(&newer, &older), newer);
    }

    #[test]
    fn last_write_tie_keeps_local() {
        let local = bookmark("a", 100);
        let remote = bookmark("a", 100);
        assert_eq!(resolve_last_write(&local, &remote), local);
    }

    #[test]
    fn merge_unions_disjoint_ids_and_resolves_overlaps() {
        let local = vec![bookmark("a", 100), bookmark("b", 100)];
        let remote = vec![bookmark("b", 200), bookmark("c", 100)];

        let merged = merge_by_id(&local, &remote);
        let ids: Vec<&str> = merged.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
        // The newer "b" from the remote won.
        let b = merged.iter().find(|b| b.id == "b").unwrap();
        assert_eq!(b.created_at, 200);
    }

    #[test]
    fn op_keys_pair_a_put_and_its_delete() {
        assert_eq!(
            SyncOp::PutBookmark(bookmark("x", 0)).key(),
            SyncOp::DeleteBookmark {
                id: "x".to_string()
            }
            .key()
        );
    }

    #[test]
    fn outbox_enqueue_is_idempotent_by_key() {
        let mut outbox = Outbox::new();
        outbox.enqueue(SyncOp::PutBookmark(bookmark("x", 1)));
        outbox.enqueue(SyncOp::PutBookmark(bookmark("x", 2)));
        assert_eq!(outbox.len(), 1);

        // A later delete of the same record replaces the pending put.
        outbox.enqueue(SyncOp::DeleteBookmark {
            id: "x".to_string(),
        });
        assert_eq!(outbox.len(), 1);
        assert_eq!(
            outbox.pending()[0].op,
            SyncOp::DeleteBookmark {
                id: "x".to_string()
            }
        );
    }

    #[test]
    fn outbox_ack_removes_the_confirmed_entry() {
        let mut outbox = Outbox::new();
        outbox.enqueue(SyncOp::PutBookmark(bookmark("x", 1)));
        outbox.enqueue(SyncOp::PutBookmark(bookmark("y", 1)));

        outbox.ack(&SyncOp::PutBookmark(bookmark("x", 1)).key());
        assert_eq!(outbox.len(), 1);
        assert_eq!(
            outbox.pending()[0].op,
            SyncOp::PutBookmark(bookmark("y", 1))
        );

        outbox.ack(&SyncOp::PutBookmark(bookmark("y", 1)).key());
        assert!(outbox.is_empty());
    }

    #[test]
    fn remote_index_lists_through_the_port() {
        let remote = Arc::new(InMemoryRemoteStore::new());
        remote.upload("books/a.epub", b"data").unwrap();
        let service = SyncService::new(remote);

        let listed = service.remote_index("books/").unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].path, "books/a.epub");
    }
}
