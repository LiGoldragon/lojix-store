//! Reader-side API for arca.
//!
//! Public — any process can link this and read the store. Reads
//! are mmap-friendly and don't require a daemon round-trip. The
//! index DB is opened read-only; the tree contents are plain
//! filesystem paths.
//!
//! Nix's equivalent: any user can read `/nix/store` without
//! talking to `nix-daemon`. Same pattern here.

use crate::hash::StoreEntryHash;
use crate::layout::{StorePath, StoreRoot};
use crate::{Error, Result};

/// Read-only handle to an arca directory.
pub trait StoreReader {
    /// Does the given hash exist in this store?
    fn contains(&self, hash: StoreEntryHash) -> Result<bool>;

    /// Resolve a store-entry hash to its top-level tree path.
    /// Error if the hash is unknown.
    fn resolve(&self, hash: StoreEntryHash) -> Result<StorePath>;

    /// Iterate every entry currently in the store.
    fn entries(&self) -> Result<Box<dyn Iterator<Item = StoreEntryHash> + '_>>;

    /// Metadata for a given entry.
    fn metadata(&self, hash: StoreEntryHash) -> Result<EntryMetadata>;
}

/// Metadata stored in the index per entry.
#[derive(Clone, Debug)]
pub struct EntryMetadata {
    pub hash: StoreEntryHash,
    pub byte_len: u64,
    pub built_at_rev: u64,
    pub source_narhash: Option<String>, // nix narhash, for provenance
}

/// Concrete reader opening a store at `root`.
pub struct StoreReaderHandle {
    root: StoreRoot,
}

impl StoreReaderHandle {
    pub fn open(_root: StoreRoot) -> Result<Self> {
        todo!()
    }
}

impl StoreReader for StoreReaderHandle {
    fn contains(&self, _hash: StoreEntryHash) -> Result<bool> {
        todo!()
    }

    fn resolve(&self, _hash: StoreEntryHash) -> Result<StorePath> {
        todo!()
    }

    fn entries(&self) -> Result<Box<dyn Iterator<Item = StoreEntryHash> + '_>> {
        todo!()
    }

    fn metadata(&self, _hash: StoreEntryHash) -> Result<EntryMetadata> {
        todo!()
    }
}
