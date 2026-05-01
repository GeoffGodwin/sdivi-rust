//! [`PatternFingerprint`] — keyed blake3 digest of a pattern's structural shape.

use serde::{Deserialize, Serialize};

/// Fixed 32-byte key for all pattern fingerprints.
///
/// This constant must never change within a `snapshot_version`. Changing it
/// invalidates all existing snapshot fingerprints and requires a snapshot
/// version bump per CLAUDE.md Rule 19.
pub const FINGERPRINT_KEY: [u8; 32] = *b"sdi-rust::patterns::fingerprint!";

/// A keyed blake3 digest representing the structural shape of a pattern.
///
/// Fingerprints are computed from the `node_kind` of a [`sdi_parsing::feature_record::PatternHint`],
/// producing a stable, content-independent shape identifier. Two pattern instances
/// with the same `node_kind` produce the same [`PatternFingerprint`].
///
/// Serialized as a 64-character lowercase hex string for JSON key compatibility.
///
/// # Examples
///
/// ```rust
/// use sdi_patterns::fingerprint::fingerprint_node_kind;
///
/// let fp1 = fingerprint_node_kind("try_expression");
/// let fp2 = fingerprint_node_kind("try_expression");
/// let fp3 = fingerprint_node_kind("match_expression");
///
/// assert_eq!(fp1, fp2);
/// assert_ne!(fp1, fp3);
/// assert_eq!(fp1.to_hex().len(), 64);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatternFingerprint([u8; 32]);

impl PatternFingerprint {
    /// Constructs a [`PatternFingerprint`] from raw digest bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        PatternFingerprint(bytes)
    }

    /// Parses a [`PatternFingerprint`] from a 64-character lowercase hex string.
    ///
    /// Returns `None` if the string is not exactly 64 hex characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sdi_patterns::fingerprint::{fingerprint_node_kind, PatternFingerprint};
    ///
    /// let fp = fingerprint_node_kind("try_expression");
    /// let hex = fp.to_hex();
    /// let parsed = PatternFingerprint::from_hex(&hex).unwrap();
    /// assert_eq!(fp, parsed);
    /// ```
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != 64 { return None; }
        let mut bytes = [0u8; 32];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(PatternFingerprint(bytes))
    }

    /// Returns the raw digest bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns a 64-character lowercase hex string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sdi_patterns::fingerprint::fingerprint_node_kind;
    ///
    /// let fp = fingerprint_node_kind("try_expression");
    /// assert_eq!(fp.to_hex().len(), 64);
    /// ```
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl Serialize for PatternFingerprint {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for PatternFingerprint {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let hex: String = Deserialize::deserialize(d)?;
        Self::from_hex(&hex)
            .ok_or_else(|| serde::de::Error::custom("expected 64-char ASCII hex fingerprint"))
    }
}

/// Computes a [`PatternFingerprint`] for the given `node_kind` using the fixed key.
///
/// Equivalent to `normalize_and_hash(node_kind, &[])` — the algorithm for a
/// leaf node (empty children) is byte-identical to this function.
///
/// # Examples
///
/// ```rust
/// use sdi_patterns::fingerprint::fingerprint_node_kind;
///
/// let fp = fingerprint_node_kind("await_expression");
/// assert_eq!(fp.to_hex().len(), 64);
/// ```
pub fn fingerprint_node_kind(node_kind: &str) -> PatternFingerprint {
    let hash = blake3::keyed_hash(&FINGERPRINT_KEY, node_kind.as_bytes());
    PatternFingerprint::from_bytes(*hash.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_node_kind_same_fingerprint() {
        assert_eq!(
            fingerprint_node_kind("try_expression"),
            fingerprint_node_kind("try_expression")
        );
    }

    #[test]
    fn different_node_kinds_different_fingerprints() {
        assert_ne!(
            fingerprint_node_kind("try_expression"),
            fingerprint_node_kind("match_expression")
        );
    }

    #[test]
    fn hex_is_64_chars() {
        assert_eq!(fingerprint_node_kind("closure_expression").to_hex().len(), 64);
    }

    #[test]
    fn serde_round_trip() {
        let fp = fingerprint_node_kind("await_expression");
        let json = serde_json::to_string(&fp).unwrap();
        let decoded: PatternFingerprint = serde_json::from_str(&json).unwrap();
        assert_eq!(fp, decoded);
    }

    #[test]
    fn fingerprint_key_is_32_bytes() {
        assert_eq!(FINGERPRINT_KEY.len(), 32);
    }

    #[test]
    fn from_hex_round_trips() {
        let fp = fingerprint_node_kind("try_expression");
        let parsed = PatternFingerprint::from_hex(&fp.to_hex()).unwrap();
        assert_eq!(fp, parsed);
    }

    #[test]
    fn from_hex_invalid_length_returns_none() {
        assert!(PatternFingerprint::from_hex("abc").is_none());
        assert!(PatternFingerprint::from_hex("").is_none());
    }

    #[test]
    fn serde_deserialize_non_ascii_returns_err() {
        // 64-byte non-ASCII UTF-8 string should be rejected during deserialization, not panic
        let s = "é".repeat(32); // é is 2 bytes each; 32 × 2 = 64 bytes but not 64 chars
        let json = serde_json::to_string(&s).unwrap();
        let result: Result<PatternFingerprint, _> = serde_json::from_str(&json);
        assert!(result.is_err());
    }
}
