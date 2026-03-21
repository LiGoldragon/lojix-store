use miette::Diagnostic;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    #[error("I/O error: {detail}")]
    Io { detail: String },

    #[error("store error: {detail}")]
    Store { detail: String },

    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            detail: e.to_string(),
        }
    }
}

impl From<redb::Error> for Error {
    fn from(e: redb::Error) -> Self {
        Self::Store {
            detail: e.to_string(),
        }
    }
}
