# criome-store

> **⚠ Context: renamed in the canonical architecture**
>
> Per `mentci-next/docs/architecture.md` and
> `mentci-next/reports/019`, the MVP splits this slot into two:
>
> - **sema** — records database (redb-backed; logical code
>   records, owned by criomed).
> - **lojix-store** — opaque blob store (append-only file +
>   hash→offset index; compiled binaries, attachments; owned by
>   lojixd).
>
> This repo is the **predecessor** of `lojix-store`. It still
> contains the content-addressed-blob prototype; the
> `MemoryStore` and `ChunkStore` traits below are the seed for
> `lojix-store`'s reader library.
>
> It is **not** the single universal store the first paragraph
> below describes. Records and blobs have diverged.

Earlier framing (kept for historical reference):

The content-addressed blob store for the lojix family. Holds
opaque bytes — compiled binaries, user file attachments, any
large/unstructured payload — addressed by blake3. In the MVP
architecture, every record in sema that references large data
stores a `BlobRef` pointing here.

## Planned shape (renaming to lojix-store)

**Append-only file + rebuildable hash→offset index.** No kind
bytes in the MVP schema (per `mentci-next/reports/017`): blob
types are known only through the sema records that reference
them. The `kind_byte` idea below survives only as a historical
marker; new code should not depend on it.

## Two Traits (prototype)

**`Store`** — the prototype typed layer; see historical note
above. Kept for tests.

**`ChunkStore`** (from arbor) — the raw layer.
`put(hash, bytes)`, `get(hash) → bytes`, `contains(hash)`.

## Current Implementation

`MemoryStore` — in-memory `HashMap<ContentHash, (u8, Vec<u8>)>`. 9 tests.
Good for development; does not reflect the terminal architecture.

## Target: Append-Only File Store

```
~/.lojix/store/
  store.bin     append-only data file (all blobs)
  store.idx     hash→(offset, length) cache (rebuildable)
```

Directory renames from `~/.criome/store/` to `~/.lojix/store/`
when the lojix-store repo consolidates.

## Dependency on arbor

Shelved for MVP. When arbor returns (post-self-hosting), the
`ChunkStore` trait is how it plugs in.

## VCS

Jujutsu (`jj`) is mandatory. Always pass `-m`.
