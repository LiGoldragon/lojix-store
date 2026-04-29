# lojix-store

Content-addressed filesystem — a blake3-hashed analogue to the
nix-store. Holds real unix files and directory trees; a
separate index DB tracks `hash → path + metadata + reachability`.

## Role in the sema ecosystem

Per `criome/ARCHITECTURE.md §5`:

- **sema** (records DB, redb-backed) holds logical records —
  owned by criome.
- **lojix-store** (this repo) holds **real-file artifacts** —
  compiled binary trees, user attachments — owned by lojix.
- **sema records reference lojix-store hashes** as canonical
  artifact identity.

During the bootstrap era, nix builds into `/nix/store`; lojix's
StoreWriter copies the closure into
`~/.lojix/store/<blake3>/` with RPATH rewrite; the
`StoreEntryHash` is what sema sees. `/nix/store` is a
transient build-intermediate, not a destination.

## Status

**Skeleton-as-design.** Types and trait signatures are pinned;
function bodies are `todo!()`. `cargo check` passes; `cargo
build` fails (intentional — nothing's implemented). The
skeleton **is** the design doc; modifying the interface means
modifying this code.

Real implementation lands alongside lojix scaffolding.

## Module layout

```
src/
  lib.rs        — crate-level invariants + module re-exports
  hash.rs       — StoreEntryHash newtype (blake3)
  layout.rs     — StoreRoot, StorePath; ~/.lojix/store/<hex>/...
  reader.rs     — StoreReader trait; public read-side API
  writer.rs     — StoreWriter trait; in-process (lojix only)
  bundle.rs    — BundleFromNix trait; /nix/store → lojix-store
  index.rs      — IndexReader / IndexWriter; metadata+reachability
  error.rs      — Error + Result
```

Read `src/lib.rs` for the overview.

## Design invariants (enforced by types)

- Store-entry identity is `StoreEntryHash`, which is blake3 of
  the canonical tree encoding.
- Reader API is public (`StoreReader` trait); any process can
  link it and read.
- Writer API is in-process only (`StoreWriter` trait in
  lojix); writes require a criome-signed capability (checked
  upstream of the handle).
- Paths are distinct types (`StorePath`) from bare `PathBuf`.
- `BundlePolicy` makes the determinism controls explicit —
  `normalise_timestamps`, `strip_build_id`, `rewrite_rpath`.

## Heritage

## VCS

Jujutsu (`jj`) is mandatory. Always pass `-m`.
