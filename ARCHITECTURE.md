# ARCHITECTURE — arca

A content-addressed filesystem — nix-store analogue, hashed by
blake3 of canonical encoding. Holds **actual unix files and
directory trees**, not blobs. A compiled binary lives at a
hash-derived path; you `exec` it directly.

General-purpose. Any data that doesn't fit in sema's record
shape lives here: forge build outputs, user attachments,
documents, anything blob-shaped. forge happens to be the most
active writer today; future writers earn the same write
capability the same way.

## Two pieces

arca is **one library + one daemon**:

- **arca library** — the reader API + on-disk layout types.
  Public; any process can link it and read.
- **arca-daemon** — the privileged writer. Owns a write-only
  staging directory; computes blake3 of deposited content;
  moves into one of the stores it manages; updates the index
  DB.

Reads are direct filesystem access (no daemon round-trip).
Writes go through arca-daemon, gated by a criome-signed
capability token.

## Multi-store

arca-daemon manages **multiple stores** for access control.
Each store is a directory under `~/.arca/<store-name>/`:

```
~/.arca/
├── system/                     # system-level artifacts
│   ├── <blake3>/               # one entry per content hash
│   ├── ...
│   └── index.redb              # per-store index
├── user-foo/                   # per-user store
│   ├── <blake3>/
│   └── index.redb
├── project-bar/                # per-project store
│   ├── <blake3>/
│   └── index.redb
└── _staging/                   # write-only deposit area
    └── ...
```

Capability tokens reference a target store. arca-daemon
verifies the token, validates the deposit lands in the right
store. nix-store has one global read-only store; arca's
multi-store shape adds the access-control layer nix-store can't
express.

**Stores are read-only to consumers at the filesystem level.**
Only arca-daemon has write permission on store directories.
Reads need no token — anyone with filesystem access reads.
Writes need a token.

## Write-only staging

The staging directory is **write-only** to writers. Once a
writer deposits content, it can't edit, list, or read what's
there. arca-daemon then:

1. Reads the deposit (only arca-daemon has read permission on
   staging).
2. Computes the blake3 of the canonical encoding.
3. Validates the writer's capability token specifies the right
   store + permits the operation.
4. Atomically moves the tree into the canonical location
   `~/.arca/<store>/<blake3>/`.
5. Updates the index DB with the entry's metadata.
6. Replies with the computed hash.

Why write-only: the writer can't modify the bytes between
deposit and arca-daemon's hash check. The hash arca computes
is the hash of exactly what arca moved into the store. No
TOCTOU race.

## Role in the sema-ecosystem

```
   sema records (referencing arca by hash)
            │
            ▼
   criome ── signs capability tokens for arca writes ──┐
            │                                            │
            │ signal verbs                                │
            ▼                                            │
       writers (forge today, future writers)             │
            │                                            │
            │ deposits content into write-only staging   │
            │ with capability token                      ▼
            ▼
   ~/.arca/_staging/   ── arca-daemon ──▶ blake3 + token check
                                          │
                                          │ atomic move
                                          ▼
                              ~/.arca/<store>/<blake3>/
                              (canonical, sema-referenced,
                               read-only to consumers)
```

## Boundaries

Owns:

- The on-disk layout (`~/.arca/<store>/<blake3>/...`).
- The write-only staging directory (`~/.arca/_staging/`).
- The redb index DB per store (hash → metadata + reachability).
- The arca-daemon binary.
- The reader library (open public, read-only).
- Capability-token verification on incoming deposit verbs
  (tokens signed by criome).

Does not own:

- The artifact files' content semantics — those are known only
  through the sema record that references the hash.
- Garbage collection policy — criome maintains the
  reachability view via sema records; arca is GC's executor,
  not its policy author.
- Capability-token signing — that's criome's authorization
  layer; arca verifies but doesn't issue.
- Source-of-truth for which stores exist — criome holds the
  authoritative store-list as sema records; arca-daemon reads
  this on startup and at config-update events.

## Day-one canonical

arca ships **alongside forge from day one**, not after some
migration phase. Reasoning: dogfooding the real interface now
reveals what it actually needs; deferred implementations rot.
The gradualist path "nix builds; arca stores; loosen dep on
nix over time" is strictly safer than "nix forever until Big
Bang replace."

## Code map

```
arca/
├── src/
│   ├── lib.rs        — library entry; reader API + types
│   ├── hash.rs       — blake3 helpers, hex encoding
│   ├── layout.rs     — path conventions per store
│   ├── reader.rs     — StoreReader trait; public read API
│   ├── writer.rs     — StoreWriter trait; in-process inside
│   │                   arca-daemon only
│   ├── bundle.rs     — BundleFromNix trait; /nix/store → arca
│   ├── index.rs      — IndexReader / IndexWriter; per-store redb
│   ├── deposit.rs    — write-only staging + atomic move (NEW)
│   ├── token.rs      — capability-token verification (NEW)
│   ├── error.rs      — Error + Result
│   └── main.rs       — arca-daemon binary entry (NEW)
```

The reader / hash / layout helpers have real implementations
and tests. The writer + bundle + deposit + token + daemon
bodies are `todo!()` skeleton-as-design; bodies land alongside
forge scaffolding.

## Invariants

- **Hash is canonical identity.** Renames or path changes do
  not change the hash; only content does.
- **No typing.** arca does not know what a hash points at;
  sema records own that knowledge.
- **Atomic moves.** Deposits move into the canonical location
  by rename — no half-written entries.
- **Write-only deposit.** Writers cannot edit their content
  after dropping it in staging; the hash arca computes is the
  hash of exactly what gets stored.
- **Capability-tokened writes.** Every deposit verb carries a
  criome-signed token referencing a sema authz record + target
  store. arca-daemon verifies signature; rejects expired or
  malformed tokens.
- **Stores are read-only at the filesystem level.** Only
  arca-daemon has write permission on store directories.

## Cross-cutting context

- Two-stores model:
  [criome/ARCHITECTURE.md §5](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
- Compile + self-host loop:
  [criome/ARCHITECTURE.md §7](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
- Wire protocol layered atop signal:
  [signal-forge/ARCHITECTURE.md](https://github.com/LiGoldragon/signal-forge/blob/main/ARCHITECTURE.md)
  (and a parallel signal-arca crate for writer ↔ arca-daemon
  verbs lands when deposits are wired)

## Status

**CANON, day-one skeleton.** Hash and layout filled; reader /
writer / deposit / token / daemon body fills land alongside
forge scaffolding.
