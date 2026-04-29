//! Writer-side API for arca.
//!
//! **In-process only** — lives inside the privileged writer.
//! No public writer trait; arca is single-writer under criome-
//! signed capability tokens, and those tokens are validated
//! inside the writer before a `StoreWriter` is handed out.
//!
//! Nix's equivalent: `nix-daemon` is the only process that
//! writes to `/nix/store`; everything else goes through its
//! protocol. Same pattern here — forge holds the writer today;
//! future privileged writers earn the capability the same way.

use std::path::Path;

use crate::hash::StoreEntryHash;
use crate::layout::StoreRoot;
use crate::{Error, Result};

/// Write-side handle to an arca directory.
pub trait StoreWriter: Send {
    /// Place a tree under a newly-computed blake3 hash.
    ///
    /// `source_tree` is a directory on disk (typically a
    /// materialised nix-store output that has already been
    /// RPATH-rewritten). This function:
    ///
    /// 1. Canonicalises the tree (normalised timestamps, stable
    ///    orderings) so the blake3 is deterministic.
    /// 2. Hashes it into a `StoreEntryHash`.
    /// 3. Moves/copies into `<root>/<hex>/`.
    /// 4. Updates the index DB with metadata.
    /// 5. Returns the hash.
    ///
    /// If the hash already exists, dedup wins and the input tree
    /// is discarded.
    fn put_tree(
        &mut self,
        source_tree: &Path,
        source_narhash: Option<String>,
    ) -> Result<StoreEntryHash>;

    /// Delete an entry. Typically criome-driven GC; requires a
    /// capability token (validated upstream of this call).
    fn delete(&mut self, hash: StoreEntryHash) -> Result<()>;
}

/// Concrete writer opening a store at `root`.
///
/// NB: only one `StoreWriterHandle` should be alive per store
/// root at a time. Enforcement is the writer process's
/// responsibility.
pub struct StoreWriterHandle {
    root: StoreRoot,
}

impl StoreWriterHandle {
    pub fn open(_root: StoreRoot) -> Result<Self> {
        todo!()
    }
}

impl StoreWriter for StoreWriterHandle {
    fn put_tree(
        &mut self,
        _source_tree: &Path,
        _source_narhash: Option<String>,
    ) -> Result<StoreEntryHash> {
        todo!()
    }

    fn delete(&mut self, _hash: StoreEntryHash) -> Result<()> {
        todo!()
    }
}
