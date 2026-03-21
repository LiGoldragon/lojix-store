use std::path::{Path, PathBuf};
use std::sync::Arc;

use redb::{Database, TableDefinition};

use crate::error::Error;
use crate::hash::ContentHash;

/// Index table: blake3 hash (32 bytes) → blob metadata (cbor or raw bytes).
const BLOBS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("blobs");

/// Metadata stored alongside each blob in the index.
#[derive(Debug, Clone)]
pub struct BlobMeta {
    /// Original size in bytes (before compression).
    pub size: u64,
    /// Compressed size on disk.
    pub compressed_size: u64,
    /// Whether the blob is stored zstd-compressed.
    pub compressed: bool,
}

impl BlobMeta {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(17);
        buf.extend_from_slice(&self.size.to_le_bytes());
        buf.extend_from_slice(&self.compressed_size.to_le_bytes());
        buf.push(u8::from(self.compressed));
        buf
    }

    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 17 {
            return None;
        }
        Some(Self {
            size: u64::from_le_bytes(data[0..8].try_into().ok()?),
            compressed_size: u64::from_le_bytes(data[8..16].try_into().ok()?),
            compressed: data[16] != 0,
        })
    }
}

/// Content-addressed blob store.
///
/// Blobs are identified by their blake3 hash. The store is opaque —
/// nothing outside this module ever sees a filesystem path.
pub struct Store {
    /// Root directory for blob files.
    root: PathBuf,
    /// Index database.
    db: Arc<Database>,
}

impl Store {
    /// Open or create a store at the given root directory.
    pub fn open(root: &Path) -> Result<Self, Error> {
        std::fs::create_dir_all(root)?;
        let db_path = root.join("index.redb");
        let db = Database::create(&db_path).map_err(|e| Error::Store {
            detail: format!("failed to open index: {e}"),
        })?;

        // Ensure table exists.
        let txn = db.begin_write().map_err(|e| Error::Store {
            detail: e.to_string(),
        })?;
        txn.open_table(BLOBS).map_err(|e| Error::Store {
            detail: e.to_string(),
        })?;
        txn.commit().map_err(|e| Error::Store {
            detail: e.to_string(),
        })?;

        Ok(Self {
            root: root.to_path_buf(),
            db: Arc::new(db),
        })
    }

    /// Store a blob. Returns its content hash.
    ///
    /// If the blob already exists (same hash), this is a no-op.
    pub fn put(&self, data: &[u8]) -> Result<ContentHash, Error> {
        let hash = ContentHash::from_bytes(data);

        // Check if already stored.
        if self.exists(&hash)? {
            return Ok(hash);
        }

        // Decide whether to compress: try zstd, keep if >10% savings.
        let (blob_bytes, compressed) = try_compress(data);
        let blob_path = self.blob_path(&hash);

        // Write: tmp file → fsync → rename (crash-safe).
        let tmp_path = blob_path.with_extension("tmp");
        if let Some(parent) = blob_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&tmp_path, &blob_bytes)?;
        // fsync the file.
        let f = std::fs::File::open(&tmp_path)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp_path, &blob_path)?;

        // Write index entry.
        let meta = BlobMeta {
            size: data.len() as u64,
            compressed_size: blob_bytes.len() as u64,
            compressed,
        };
        let txn = self.db.begin_write().map_err(|e| Error::Store {
            detail: e.to_string(),
        })?;
        {
            let mut table = txn.open_table(BLOBS).map_err(|e| Error::Store {
                detail: e.to_string(),
            })?;
            table
                .insert(hash.as_bytes().as_slice(), meta.to_bytes().as_slice())
                .map_err(|e| Error::Store {
                    detail: e.to_string(),
                })?;
        }
        txn.commit().map_err(|e| Error::Store {
            detail: e.to_string(),
        })?;

        Ok(hash)
    }

    /// Retrieve a blob by its content hash.
    pub fn get(&self, hash: &ContentHash) -> Result<Vec<u8>, Error> {
        let meta = self.meta(hash)?.ok_or_else(|| Error::Store {
            detail: format!("blob not found: {hash}"),
        })?;

        let blob_path = self.blob_path(hash);
        let raw = std::fs::read(&blob_path)?;

        let data = if meta.compressed {
            zstd::decode_all(raw.as_slice()).map_err(|e| Error::Store {
                detail: format!("decompression failed: {e}"),
            })?
        } else {
            raw
        };

        // Verify content hash.
        let actual = ContentHash::from_bytes(&data);
        if actual != *hash {
            return Err(Error::HashMismatch {
                expected: hash.to_hex(),
                actual: actual.to_hex(),
            });
        }

        Ok(data)
    }

    /// Check if a blob exists in the store.
    pub fn exists(&self, hash: &ContentHash) -> Result<bool, Error> {
        let txn = self.db.begin_read().map_err(|e| Error::Store {
            detail: e.to_string(),
        })?;
        let table = txn.open_table(BLOBS).map_err(|e| Error::Store {
            detail: e.to_string(),
        })?;
        let found = table
            .get(hash.as_bytes().as_slice())
            .map_err(|e| Error::Store {
                detail: e.to_string(),
            })?
            .is_some();
        Ok(found)
    }

    /// Get metadata for a blob without reading the blob data.
    pub fn meta(&self, hash: &ContentHash) -> Result<Option<BlobMeta>, Error> {
        let txn = self.db.begin_read().map_err(|e| Error::Store {
            detail: e.to_string(),
        })?;
        let table = txn.open_table(BLOBS).map_err(|e| Error::Store {
            detail: e.to_string(),
        })?;
        let result = table
            .get(hash.as_bytes().as_slice())
            .map_err(|e| Error::Store {
                detail: e.to_string(),
            })?;
        Ok(result.and_then(|v| BlobMeta::from_bytes(v.value())))
    }

    /// Opaque internal path for a blob. Two-level fan-out.
    fn blob_path(&self, hash: &ContentHash) -> PathBuf {
        let hex = hash.to_hex();
        self.root
            .join("blobs")
            .join(&hex[..2])
            .join(&hex[2..4])
            .join(&hex)
    }
}

/// Try zstd compression. Return compressed bytes if >10% savings,
/// otherwise return original bytes uncompressed.
fn try_compress(data: &[u8]) -> (Vec<u8>, bool) {
    match zstd::encode_all(data, 3) {
        Ok(compressed) if compressed.len() < data.len() * 9 / 10 => (compressed, true),
        _ => (data.to_vec(), false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();

        let data = b"the zodiac is tropical";
        let hash = store.put(data).unwrap();

        assert!(store.exists(&hash).unwrap());

        let retrieved = store.get(&hash).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn dedup() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();

        let data = b"same content twice";
        let h1 = store.put(data).unwrap();
        let h2 = store.put(data).unwrap();

        assert_eq!(h1, h2);
    }
}
