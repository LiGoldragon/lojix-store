//! arca-daemon — binary entry point.
//!
//! The privileged writer for arca. Owns a write-only staging
//! directory; manages multiple stores under `~/.arca/<store>/`;
//! verifies criome-signed capability tokens on every deposit;
//! computes blake3 of staged content; atomically moves into
//! the target store; updates the per-store index DB.
//!
//! Reads are direct filesystem access (no daemon round-trip);
//! writes go through this daemon.

use arca::Result;

#[tokio::main]
async fn main() -> Result<()> {
    todo!(
        "arca-daemon entry: \
         (1) load store registry from sema (one-time at startup), \
         (2) bind UDS for signal-arca verbs, \
         (3) watch ~/.arca/_staging/ for new deposits, \
         (4) per-deposit: verify capability token, compute blake3, \
             validate target store, atomic move into ~/.arca/<store>/<blake3>/, \
             update per-store redb index, reply with hash"
    )
}
