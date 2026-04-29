//! arca — content-addressed filesystem for artifact trees.
//!
//! An analogue to the nix-store, hashed by blake3. Holds real
//! unix files and directory trees under hash-derived paths,
//! organised into multiple stores under `~/.arca/<store>/`.
//! A compiled binary lives at
//! `~/.arca/<store>/<blake3>/bin/<name>` and is directly
//! executable.
//!
//! Two pieces:
//!
//! - **Library** (this crate, [`arca`]) — public reader API +
//!   on-disk layout types. Any process can link and read.
//! - **Daemon** ([`arca-daemon`] binary, `src/main.rs`) — the
//!   privileged writer. Owns a write-only staging directory;
//!   verifies criome-signed capability tokens; computes
//!   blake3; atomically moves deposits into the target store.
//!
//! General-purpose. forge writes build outputs into arca;
//! future writers (uploads, document store, others) write
//! into arca too via arca-daemon's deposit flow. arca itself
//! doesn't know what's in any entry — sema records own that
//! knowledge.
//!
//! # Design invariants
//!
//! - Store entries are real files on disk, not packed blobs.
//! - Identity is blake3 of the tree's canonical content.
//! - The index DB maps `StoreEntryHash → { path, metadata,
//!   reachability }`. The index does not contain the files.
//! - Writes go through `StoreWriter` (in-process; capability-
//!   gated).
//! - Reads go through `StoreReader` (mmap-friendly; no daemon
//!   round-trip for path resolution).
//! - During the bootstrap era, most writes arrive via
//!   `BundleFromNix` — copy an existing `/nix/store/...` closure
//!   into arca with RPATH rewrite.
//!
//! # Skeleton-as-design
//!
//! This crate is types + trait signatures + module layout.
//! Bodies are `todo!()`. Real implementation lands alongside
//! forge scaffolding.

pub mod hash;
pub mod layout;
pub mod reader;
pub mod writer;
pub mod bundle;
pub mod index;
pub mod error;

pub use error::{Error, Result};
pub use hash::StoreEntryHash;
pub use reader::StoreReader;
pub use writer::StoreWriter;
