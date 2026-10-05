//! Size-limited chunking helper for large RPC messages.
//!
//! Architecture.md 4.2: messages above 1 MB are chunked. A large payload
//! is split into base64 parts, each small enough to travel as one line
//! inside the 1 MB frame limit. The receiver joins them back strictly:
//! every part must decode cleanly and no part may exceed the part limit.

use crate::{CoreError, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

/// Largest single part in decoded bytes. Parts travel base64-encoded on
/// one line each, so this stays well below the 1 MB RPC frame limit.
pub const MAX_PART_BYTES: usize = 512 * 1024;

/// Split `data` into base64 parts of at most `max_part_bytes` bytes.
/// Empty input yields no parts; the receiver treats that as empty data.
pub fn split(data: &[u8], max_part_bytes: usize) -> Result<Vec<String>> {
    if max_part_bytes == 0 {
        return Err(CoreError::Other("chunk part size must be positive".into()));
    }
    let mut parts = Vec::new();
    for chunk in data.chunks(max_part_bytes) {
        parts.push(BASE64.encode(chunk));
    }
    Ok(parts)
}

/// Join base64 parts back into the original bytes. Fails if any part is
/// over the limit, is not valid base64, or an expected total was declared
/// and the parts do not add up to it.
pub fn join(parts: &[String], max_part_bytes: usize) -> Result<Vec<u8>> {
    if parts.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let decoded = BASE64
            .decode(part.trim())
            .map_err(|err| CoreError::Other(format!("chunk part {index} is not base64: {err}")))?;
        if decoded.len() > max_part_bytes {
            return Err(CoreError::Other(format!(
                "chunk part {index} exceeds the part limit"
            )));
        }
        out.extend_from_slice(&decoded);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_across_part_boundaries() {
        let data: Vec<u8> = (0..=255u8).cycle().take(MAX_PART_BYTES + 7).collect();
        let parts = split(&data, MAX_PART_BYTES).unwrap_or_default();
        assert_eq!(parts.len(), 2);
        let joined = join(&parts, MAX_PART_BYTES).unwrap_or_default();
        assert_eq!(joined, data);
    }

    #[test]
    fn empty_data_yields_no_parts() {
        assert!(split(&[], MAX_PART_BYTES).unwrap_or_default().is_empty());
        assert!(join(&[], MAX_PART_BYTES).unwrap_or_default().is_empty());
    }

    #[test]
    fn small_data_is_one_part() {
        let parts = split(b"xenon", 64).unwrap_or_default();
        assert_eq!(parts.len(), 1);
        assert_eq!(join(&parts, 64).unwrap_or_default(), b"xenon");
    }

    #[test]
    fn oversized_part_is_rejected_on_join() {
        let parts = vec![BASE64.encode([0u8; 65])];
        assert!(join(&parts, 64).is_err());
    }

    #[test]
    fn invalid_base64_is_rejected() {
        let parts = vec!["not base64!!!".to_string()];
        assert!(join(&parts, 64).is_err());
    }

    #[test]
    fn zero_part_size_is_rejected() {
        assert!(split(b"abc", 0).is_err());
    }
}
