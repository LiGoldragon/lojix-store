//! Error type for arca operations.

use std::io;
use std::path::PathBuf;

use crate::hash::{HashParseError, StoreEntryHash};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io at {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("unknown store-entry hash: {0:?}")]
    UnknownHash(StoreEntryHash),

    #[error("store root is not initialised: {0:?}")]
    UninitialisedRoot(PathBuf),

    #[error("bundle failed: {reason}")]
    Bundle { reason: String },

    #[error("index-db error: {0}")]
    Index(String),

    #[error("hash parse: {0}")]
    HashParse(#[from] HashParseError),
}
