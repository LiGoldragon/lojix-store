//! Index DB for arca.
//!
//! Maps `StoreEntryHash → { tree_path, byte_len, built_at_rev,
//! source_narhash, reachability_state }`. Backed by redb (or
//! similar); readers open it read-only.
//!
//! The index does not hold the file contents — just metadata
//! about where in the filesystem each entry's tree lives and
//! what else is known about it.

use crate::hash::StoreEntryHash;
use crate::{Error, Result};

/// Reachability-from-sema status for GC coordination. criome
/// updates these values via signal verbs to forge as record
/// graphs change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reachability {
    /// Referenced by at least one live sema record.
    Live,
    /// Not referenced; eligible for GC after a grace period.
    Unreferenced,
    /// Marked but not yet collected (sweep in progress).
    PendingGc,
}

/// One row of the index.
#[derive(Clone, Debug)]
pub struct IndexRow {
    pub hash: StoreEntryHash,
    pub byte_len: u64,
    pub built_at_rev: u64,
    pub source_narhash: Option<String>,
    pub reachability: Reachability,
}

/// Read-only handle to the index DB.
pub trait IndexReader {
    fn get(&self, hash: StoreEntryHash) -> Result<Option<IndexRow>>;
    fn iter(&self) -> Result<Box<dyn Iterator<Item = IndexRow> + '_>>;
}

/// Write-side handle; lives inside the privileged writer
/// (forge today; capability-gated for any future writer).
pub trait IndexWriter: Send {
    fn insert(&mut self, row: IndexRow) -> Result<()>;
    fn set_reachability(
        &mut self,
        hash: StoreEntryHash,
        r: Reachability,
    ) -> Result<()>;
    fn remove(&mut self, hash: StoreEntryHash) -> Result<()>;
}
