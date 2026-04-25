//! Filesystem layout conventions for a lojix-store directory.
//!
//! Default root: `$HOME/.lojix/store/`.
//!
//! Layout:
//!
//! ```text
//! ~/.lojix/store/
//!   <hex-hash>/                  # one subdirectory per entry
//!     bin/<name>                 # executables (rpath into sibling /lib)
//!     lib/<libX>.so              # shared libs (rpath absolute into lojix-store)
//!     share/...                  # data files
//!   index.redb                   # hash → { path, metadata, reachability }
//! ```
//!
//! Paths inside an entry are normal unix; the entry as a whole
//! is addressed by its blake3. Cross-entry RPATHs use absolute
//! paths into the store, so artifacts work regardless of cwd.

use std::path::{Path, PathBuf};

use crate::hash::StoreEntryHash;

/// Root of a lojix-store directory.
#[derive(Clone, Debug)]
pub struct StoreRoot(pub PathBuf);

impl StoreRoot {
    /// The default root: `$HOME/.lojix/store/`.
    ///
    /// Falls back to `./.lojix/store` if `$HOME` is unset.
    pub fn default_for_user() -> Self {
        let base = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        Self(base.join(".lojix").join("store"))
    }

    /// Path to the subdirectory that holds a given entry's tree.
    pub fn entry_tree(&self, hash: StoreEntryHash) -> PathBuf {
        self.0.join(hash.to_hex())
    }

    /// Path to the index DB file.
    pub fn index_db_path(&self) -> PathBuf {
        self.0.join("index.redb")
    }

    /// Does this store root exist and look valid?
    ///
    /// Valid = the root directory exists AND the index DB file
    /// exists. A store with no entries but an empty index is still
    /// valid; a store with entries but no index is not.
    pub fn exists(&self) -> bool {
        self.0.is_dir() && self.index_db_path().is_file()
    }
}

/// A resolved filesystem path inside the store (entry root, bin,
/// lib, or leaf). Kept distinct from bare `PathBuf` so the type
/// surface distinguishes store-resolved paths from arbitrary ones.
#[derive(Clone, Debug)]
pub struct StorePath(pub PathBuf);

impl StorePath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::HASH_LEN;

    #[test]
    fn default_path_ends_with_store_dir() {
        let root = StoreRoot::default_for_user();
        let s = root.0.to_string_lossy();
        assert!(
            s.ends_with(".lojix/store"),
            "expected path ending in .lojix/store, got {s}"
        );
    }

    #[test]
    fn entry_tree_appends_hex() {
        let root = StoreRoot(PathBuf::from("/tmp/lojix-store-test"));
        let hash = StoreEntryHash([0u8; HASH_LEN]);
        let path = root.entry_tree(hash);
        let expected = format!("/tmp/lojix-store-test/{}", "0".repeat(HASH_LEN * 2));
        assert_eq!(path.to_string_lossy(), expected);
    }

    #[test]
    fn index_db_path_is_inside_root() {
        let root = StoreRoot(PathBuf::from("/tmp/lojix-store-test"));
        let idx = root.index_db_path();
        assert_eq!(idx.to_string_lossy(), "/tmp/lojix-store-test/index.redb");
    }
}
