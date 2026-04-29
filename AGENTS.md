# arca

Content-addressed filesystem — a blake3-hashed analogue to the
nix-store. Holds real unix files and directory trees; a
separate index DB tracks `hash → path + metadata + reachability`
per store.

General-purpose: any data that doesn't fit in sema's record
shape lives in arca. forge is the most active writer today;
future writers (uploads, document store, others) get
capabilities the same way.

**One library + one daemon.** The library (`arca`) is the
public reader API + on-disk layout. The daemon (`arca-daemon`)
is the privileged writer — owns a write-only staging directory,
manages multiple stores, verifies criome-signed capability
tokens, computes blake3, atomically moves deposits into the
right store.

## Role in the sema ecosystem

Per `criome/ARCHITECTURE.md §5`:

- **sema** (records DB, redb-backed) holds logical records —
  owned by criome.
- **arca** (this repo) holds **real-file artifacts** —
  compiled binary trees, user attachments, documents, anything
  blob-shaped — content-addressed by blake3.
- **sema records reference arca hashes** as canonical
  artifact identity.

During the bootstrap era, nix builds into `/nix/store`; the
writer (forge today) bundles the closure with RPATH rewrite +
deterministic timestamps and deposits the canonicalised tree
into `~/.arca/_staging/`. arca-daemon picks it up, verifies the
capability token, computes blake3, and moves into
`~/.arca/<store>/<blake3>/`. `/nix/store` is a transient build-
intermediate, not a destination.

## Multi-store + access control

arca-daemon manages multiple stores (one directory per store
under `~/.arca/`). Capability tokens specify the target store.
Stores are filesystem-read-only to consumers; only arca-daemon
has write permission. nix-store has one global read-only store;
arca's multi-store shape is the access-control layer nix-store
can't express.

## Write-only staging

The staging directory is **write-only** to writers. Once a
writer drops content there, it can't be edited. arca-daemon
reads, validates, hashes, and moves — atomically. The hash
arca computes is the hash of exactly what gets stored.

## Status

**Skeleton-as-design.** Types and trait signatures are pinned;
function bodies are `todo!()`. `cargo check` passes; `cargo
build` of the daemon fails (intentional — nothing's
implemented). The skeleton **is** the design doc; modifying
the interface means modifying this code.

Real implementation lands alongside forge scaffolding.

## Module layout

```
src/
  lib.rs        — crate-level invariants + module re-exports
  hash.rs       — StoreEntryHash newtype (blake3)
  layout.rs     — StoreRoot, StorePath; ~/.arca/<store>/<hex>/...
  reader.rs     — StoreReader trait; public read-side API
  writer.rs     — StoreWriter trait; in-process inside arca-daemon only
  bundle.rs     — BundleFromNix trait; /nix/store → arca
  index.rs      — IndexReader / IndexWriter; per-store redb
  deposit.rs    — write-only staging + atomic move
  token.rs      — capability-token verification
  error.rs      — Error + Result
  main.rs       — arca-daemon binary entry
```

Read `src/lib.rs` for the overview.

## Design invariants (enforced by types)

- Store-entry identity is `StoreEntryHash`, which is blake3 of
  the canonical tree encoding.
- Reader API is public (`StoreReader` trait); any process can
  link it and read.
- Writer API is in-process inside arca-daemon only
  (`StoreWriter` trait); writes require a criome-signed
  capability token referencing a sema authz record + target
  store.
- Paths are distinct types (`StorePath`) from bare `PathBuf`.
- `BundlePolicy` makes the determinism controls explicit —
  `normalise_timestamps`, `strip_build_id`, `rewrite_rpath`.

## VCS

Jujutsu (`jj`) is mandatory. Always pass `-m`.
