# ARCHITECTURE — lojix-store

A content-addressed filesystem — nix-store analogue, hashed by
blake3 of canonical encoding. Holds **actual unix files and
directory trees**, not blobs. A compiled binary lives at a
hash-derived path; you `exec` it directly.

## Role

The two-stores model: **sema** holds records; **lojix-store**
holds artifact files referenced from sema by hash. Sema records
carry `StoreEntryHash` fields pointing into lojix-store.

```
nix builds → /nix/store/<hash>-<name>/ (transient)
              │
              │ lojixd's BundleIntoLojixStore actor
              │  • copy closure with RPATH rewrite (patchelf)
              │  • blake3 hash of canonical layout
              │
              ▼
~/.lojix/store/<blake3>/  (canonical, sema-referenced)
```

Sema records reference lojix-store hashes as canonical
identity; `/nix/store` is a transient build-intermediate.

## Boundaries

Owns:

- The `~/.lojix/store/` directory layout (hash-keyed
  subdirectories, close to `/nix/store/<hash>-<name>/`).
- The redb index DB mapping
  `blake3 → { path, metadata, reachability }`.
- Reader and writer types.

Does not own:

- The artifact files' content semantics — those are known only
  through the sema record that references the hash.
- Garbage collection — criomed maintains the reachability view
  via sema records; lojix-store is GC's executor, not its
  policy author.
- Capability-token verification — that's criomed's
  authorization layer; lojix-store trusts criomed-signed
  tokens.

## Day-one canonical

lojix-store ships **alongside lojixd from day one**, not after
some migration phase. Reasoning: dogfooding the real interface
now reveals what it actually needs; deferred implementations
rot. The gradualist path "nix builds; lojix-store stores;
loosen dep on nix over time" is strictly safer than "nix
forever until Big Bang replace."

## Code map

```
src/
├── lib.rs    — module entry + re-exports
├── hash.rs   — blake3 helpers, hex encoding (no extra dep)
├── layout.rs — path conventions (default_for_user, entry_tree)
├── reader.rs — StoreReaderHandle, StoreReader trait
└── writer.rs — StoreWriterHandle, StoreWriter trait
```

The hash + layout helpers have real implementations and tests.
The reader/writer trait methods are `todo!()` skeleton; bodies
land in lojixd's StoreWriter / StoreReaderPool actors.

## Invariants

- **No typing.** lojix-store does not know what a hash points
  at; sema records own that knowledge.
- **Hash is canonical identity.** Renames or path changes do
  not change the hash; only content does.
- **Atomic writes.** Writers create a temp directory, then
  rename — no half-written entries.
- **Capability-tokened access.** Writes require a criomed-
  signed token referencing a sema authz record.

## Cross-cutting context

- Two-stores model:
  [mentci-next/docs/architecture.md §5](https://github.com/LiGoldragon/mentci-next/blob/main/docs/architecture.md)
- Compile + self-host loop:
  [mentci-next/docs/architecture.md §7](https://github.com/LiGoldragon/mentci-next/blob/main/docs/architecture.md)

## Status

**CANON, day-one skeleton.** Hash and layout filled; reader /
writer body fills land alongside lojixd scaffolding (Phase C
per [lojix repo's ARCHITECTURE.md](https://github.com/LiGoldragon/lojix/blob/main/ARCHITECTURE.md)).
