//! Content-hash type used to identify store entries.
//!
//! A `StoreEntryHash` is the blake3 hash of the canonical
//! encoding of a store entry's tree contents. Canonical
//! encoding is deterministic (sorted filenames, normalised
//! timestamps, stable RPATHs); two identical trees hash to
//! the same value regardless of machine or build order.

/// blake3 output width, in bytes.
pub const HASH_LEN: usize = 32;

/// Identity of a store entry: the blake3 of its canonical tree.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StoreEntryHash(pub [u8; HASH_LEN]);

impl StoreEntryHash {
    /// Render as lowercase hex (the on-disk directory-name form).
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(HASH_LEN * 2);
        for b in &self.0 {
            s.push(hex_nibble(b >> 4));
            s.push(hex_nibble(b & 0xf));
        }
        s
    }

    /// Parse from hex.
    pub fn from_hex(hex: &str) -> Result<Self, HashParseError> {
        if hex.len() != HASH_LEN * 2 {
            return Err(HashParseError::WrongLength);
        }
        let mut bytes = [0u8; HASH_LEN];
        let raw = hex.as_bytes();
        for i in 0..HASH_LEN {
            let hi = nibble_value(raw[i * 2]).ok_or(HashParseError::InvalidHex)?;
            let lo = nibble_value(raw[i * 2 + 1]).ok_or(HashParseError::InvalidHex)?;
            bytes[i] = (hi << 4) | lo;
        }
        Ok(Self(bytes))
    }
}

fn hex_nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + n - 10) as char,
        _ => unreachable!(),
    }
}

fn nibble_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

impl From<blake3::Hash> for StoreEntryHash {
    fn from(h: blake3::Hash) -> Self {
        Self(*h.as_bytes())
    }
}

/// Errors parsing a hex-encoded hash.
#[derive(Debug, thiserror::Error)]
pub enum HashParseError {
    #[error("hex decode failed")]
    InvalidHex,
    #[error("wrong length; expected {HASH_LEN} bytes")]
    WrongLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bytes() -> [u8; HASH_LEN] {
        let mut b = [0u8; HASH_LEN];
        for (i, slot) in b.iter_mut().enumerate() {
            *slot = i as u8;
        }
        b
    }

    #[test]
    fn roundtrip_preserves_identity() {
        let bytes = sample_bytes();
        let hash = StoreEntryHash(bytes);
        let hex = hash.to_hex();
        assert_eq!(hex.len(), HASH_LEN * 2);
        let recovered = StoreEntryHash::from_hex(&hex).expect("parse");
        assert_eq!(recovered, hash);
    }

    #[test]
    fn from_hex_rejects_wrong_length() {
        assert!(matches!(
            StoreEntryHash::from_hex("00"),
            Err(HashParseError::WrongLength)
        ));
        assert!(matches!(
            StoreEntryHash::from_hex(&"a".repeat(63)),
            Err(HashParseError::WrongLength)
        ));
    }

    #[test]
    fn from_hex_rejects_non_hex_chars() {
        let bad = "z".repeat(HASH_LEN * 2);
        assert!(matches!(
            StoreEntryHash::from_hex(&bad),
            Err(HashParseError::InvalidHex)
        ));
    }

    #[test]
    fn from_hex_accepts_mixed_case() {
        let upper = "DEADBEEF".repeat(8);
        let lower = "deadbeef".repeat(8);
        let h_upper = StoreEntryHash::from_hex(&upper).expect("uppercase parses");
        let h_lower = StoreEntryHash::from_hex(&lower).expect("lowercase parses");
        assert_eq!(h_upper, h_lower);
    }
}
