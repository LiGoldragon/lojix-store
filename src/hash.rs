use std::fmt;

/// A blake3 content hash — the identity of a blob.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Hash raw bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(*blake3::hash(data).as_bytes())
    }

    /// Parse from hex string.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let bytes = blake3::Hash::from_hex(hex).ok()?;
        Some(Self(*bytes.as_bytes()))
    }

    /// The raw 32-byte hash.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Hex-encoded string.
    pub fn to_hex(&self) -> String {
        blake3::Hash::from(self.0).to_hex().to_string()
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}
