//! Bundle a `/nix/store` output into arca.
//!
//! During the bootstrap era, most artifacts enter arca via
//! this path:
//!
//! 1. forge invoked `nix build` — output landed in
//!    `/nix/store/<narhash>-<name>/`.
//! 2. The `BundleFromNix` step walks the output closure.
//! 3. For each binary or shared-lib in the closure, patchelf
//!    rewrites RPATH from `/nix/store/.../lib` to
//!    `~/.arca/<blake3-of-dep>/lib` (absolute arca paths —
//!    artifacts work regardless of cwd on the host).
//! 4. Timestamps and other non-determinism are stripped so the
//!    bundle is bit-reproducible.
//! 5. The canonicalised tree is handed to `StoreWriter::put_tree`
//!    which computes the final `StoreEntryHash` and places it.
//!
//! Eventually (post-nix-replacement) this step disappears —
//! the privileged writer drives rustc directly and writes into
//! arca. The trait stays; only the input changes.

use std::path::Path;

use crate::hash::StoreEntryHash;
use crate::{Error, Result};

/// A closure of nix-store paths to be bundled in together. The
/// root path is typically the opus's build output; dependency
/// paths are the transitive closure that RPATHs point at.
#[derive(Clone, Debug)]
pub struct NixClosure<'a> {
    pub root: &'a Path,
    pub deps: Vec<&'a Path>,
    pub source_narhash: Option<String>,
}

/// Bundle step — takes a nix-store closure, produces an
/// arca entry. Implemented by the privileged writer (forge).
pub trait BundleFromNix {
    fn bundle(&mut self, closure: NixClosure<'_>) -> Result<StoreEntryHash>;
}

/// Deterministic-bundling policy — what to normalise before
/// hashing.
#[derive(Clone, Copy, Debug, Default)]
pub struct BundlePolicy {
    /// Reset atime/mtime/ctime to a fixed epoch.
    pub normalise_timestamps: bool,
    /// Strip build-id from ELF `.note.gnu.build-id`.
    pub strip_build_id: bool,
    /// Rewrite RPATHs to absolute arca paths.
    pub rewrite_rpath: bool,
}

impl BundlePolicy {
    /// Sensible defaults for Linux ELF bundling.
    pub fn linux_default() -> Self {
        Self {
            normalise_timestamps: true,
            strip_build_id: true,
            rewrite_rpath: true,
        }
    }
}
