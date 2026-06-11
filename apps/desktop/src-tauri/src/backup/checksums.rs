use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Computes the SHA-256 hex digest of a byte slice.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// A map from archive entry path → SHA-256 hex digest.
pub type ChecksumMap = BTreeMap<String, String>;

/// Serializes a `ChecksumMap` to a canonical JSON bytes representation.
pub fn checksums_to_json(map: &ChecksumMap) -> Vec<u8> {
    serde_json::to_vec_pretty(map).expect("checksum map is always serializable")
}

/// Deserializes a `ChecksumMap` from JSON bytes.
pub fn checksums_from_json(data: &[u8]) -> Result<ChecksumMap, String> {
    serde_json::from_slice(data).map_err(|e| format!("checksum JSON parse error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_empty_bytes_produces_known_hash() {
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_known_input_produces_known_hash() {
        // SHA-256("abc") = ba7816bf...
        let hash = sha256_hex(b"abc");
        assert!(hash.starts_with("ba7816bf"));
    }

    #[test]
    fn sha256_hash_is_64_hex_chars() {
        let hash = sha256_hex(b"test data");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn sha256_different_inputs_produce_different_hashes() {
        let h1 = sha256_hex(b"hello");
        let h2 = sha256_hex(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn checksum_map_roundtrip() {
        let mut map = ChecksumMap::new();
        map.insert(
            "manifest.json".to_string(),
            sha256_hex(b"synthetic manifest"),
        );
        map.insert("base.json".to_string(), sha256_hex(b"synthetic base"));
        let json = checksums_to_json(&map);
        let map2 = checksums_from_json(&json).expect("deserialize");
        assert_eq!(map, map2);
    }

    #[test]
    fn checksum_map_is_sorted_by_key() {
        let mut map = ChecksumMap::new();
        map.insert("z.json".to_string(), sha256_hex(b"z"));
        map.insert("a.json".to_string(), sha256_hex(b"a"));
        map.insert("m.json".to_string(), sha256_hex(b"m"));
        let keys: Vec<&String> = map.keys().collect();
        assert_eq!(keys[0].as_str(), "a.json");
        assert_eq!(keys[2].as_str(), "z.json");
    }

    #[test]
    fn checksums_from_invalid_json_returns_error() {
        let result = checksums_from_json(b"not json {{{");
        assert!(result.is_err());
    }
}
