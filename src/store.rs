use crate::Error;

/// blake3 hash — the content address
pub type ContentHash = [u8; 32];

/// Content-addressed store. Immutable: same bytes always produce the same
/// hash. Writes are idempotent. Values are never updated or deleted.
///
/// The `kind` byte sorts objects into typed namespaces:
/// strings, sema objects per struct type, arbor nodes, manifests, commits.
/// All live in one store, content-addressed within each kind.
pub trait Store {
    /// Store bytes, return their content hash. Skips if already present.
    fn put(&mut self, kind: u8, data: &[u8]) -> Result<ContentHash, Error>;

    /// Retrieve bytes by content hash.
    fn get(&self, hash: &ContentHash) -> Result<&[u8], Error>;

    /// Retrieve bytes and kind by content hash.
    fn get_typed(&self, hash: &ContentHash) -> Result<(u8, &[u8]), Error>;

    /// Check if a hash exists without loading the data.
    fn contains(&self, hash: &ContentHash) -> bool;

    /// Iterate all entries of a given kind.
    fn scan(&self, kind: u8) -> Vec<(ContentHash, &[u8])>;
}

/// Hash bytes with blake3, returning the content address.
pub fn content_hash(data: &[u8]) -> ContentHash {
    *blake3::hash(data).as_bytes()
}

/// In-memory store for tests and bootstrap.
pub struct MemoryStore {
    /// hash → (kind, data)
    entries: std::collections::HashMap<ContentHash, (u8, Vec<u8>)>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for MemoryStore {
    fn put(&mut self, kind: u8, data: &[u8]) -> Result<ContentHash, Error> {
        let hash = content_hash(data);
        self.entries
            .entry(hash)
            .or_insert_with(|| (kind, data.to_vec()));
        Ok(hash)
    }

    fn get(&self, hash: &ContentHash) -> Result<&[u8], Error> {
        self.entries
            .get(hash)
            .map(|(_, data)| data.as_slice())
            .ok_or(Error::NotFound(*hash))
    }

    fn get_typed(&self, hash: &ContentHash) -> Result<(u8, &[u8]), Error> {
        self.entries
            .get(hash)
            .map(|(kind, data)| (*kind, data.as_slice()))
            .ok_or(Error::NotFound(*hash))
    }

    fn contains(&self, hash: &ContentHash) -> bool {
        self.entries.contains_key(hash)
    }

    fn scan(&self, kind: u8) -> Vec<(ContentHash, &[u8])> {
        self.entries
            .iter()
            .filter(|(_, (k, _))| *k == kind)
            .map(|(hash, (_, data))| (*hash, data.as_slice()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_round_trip() {
        let mut store = MemoryStore::new();
        let data = b"hello world";
        let hash = store.put(0, data).unwrap();
        let retrieved = store.get(&hash).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn content_addressing_is_deterministic() {
        let mut store = MemoryStore::new();
        let h1 = store.put(0, b"same bytes").unwrap();
        let h2 = store.put(0, b"same bytes").unwrap();
        assert_eq!(h1, h2);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn different_content_different_hash() {
        let mut store = MemoryStore::new();
        let h1 = store.put(0, b"alpha").unwrap();
        let h2 = store.put(0, b"beta").unwrap();
        assert_ne!(h1, h2);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn not_found() {
        let store = MemoryStore::new();
        let hash = content_hash(b"missing");
        assert!(!store.contains(&hash));
        assert!(store.get(&hash).is_err());
    }

    #[test]
    fn idempotent_put() {
        let mut store = MemoryStore::new();
        let h1 = store.put(0, b"data").unwrap();
        let h2 = store.put(1, b"data").unwrap();
        assert_eq!(h1, h2);
        // First write wins — kind stays 0
        let (_, retrieved) = store.entries.get(&h1).unwrap();
        assert_eq!(retrieved, b"data");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn hash_matches_blake3() {
        let data = b"verify hash";
        let expected = *blake3::hash(data).as_bytes();
        let actual = content_hash(data);
        assert_eq!(expected, actual);
    }

    #[test]
    fn get_typed_returns_kind() {
        let mut store = MemoryStore::new();
        let hash = store.put(7, b"typed data").unwrap();
        let (kind, data) = store.get_typed(&hash).unwrap();
        assert_eq!(kind, 7);
        assert_eq!(data, b"typed data");
    }

    #[test]
    fn scan_filters_by_kind() {
        let mut store = MemoryStore::new();
        store.put(1, b"thought alpha").unwrap();
        store.put(1, b"thought beta").unwrap();
        store.put(2, b"rule gamma").unwrap();
        store.put(0, b"string delta").unwrap();

        let thoughts = store.scan(1);
        assert_eq!(thoughts.len(), 2);

        let rules = store.scan(2);
        assert_eq!(rules.len(), 1);

        let strings = store.scan(0);
        assert_eq!(strings.len(), 1);

        let empty = store.scan(99);
        assert!(empty.is_empty());
    }
}
