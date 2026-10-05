//! xenon-core: the browser core service library.
//!
//! Owns all persistent state (ENG-04): the SQLCipher database, the OS
//! keystore integration, the JSON-RPC surface the engine hosts talk to,
//! and the PRIV-05-safe logging module. The `xenon-core` binary wraps
//! this library with a stdin/stdout JSON-RPC loop (Architecture.md 4.2).

pub mod chunked;
pub mod db;
pub mod keystore;
pub mod logging;
pub mod rpc;
pub mod settings;

/// Error for every failure mode in core. Variants never carry secret
/// material (key bytes, passwords) — only descriptions (PRIV-05).
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("keystore error: {0}")]
    Keystore(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("rpc error {code}: {message}")]
    Rpc { code: i32, message: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

/// Convenient result alias used across the crate.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Encode bytes as lowercase hex. Used for the SQLCipher key literal and
/// the test/dev key override; never for logging secrets.
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from_digit((byte >> 4) as u32, 16).unwrap_or('0'));
        out.push(char::from_digit((byte & 0x0f) as u32, 16).unwrap_or('0'));
    }
    out
}

/// Decode lowercase or uppercase hex into bytes. Errors on odd length or
/// non-hex characters instead of silently truncating.
pub fn hex_decode(text: &str) -> Result<Vec<u8>> {
    let chars: Vec<char> = text.chars().collect();
    if !chars.len().is_multiple_of(2) {
        return Err(CoreError::Other("hex string has odd length".into()));
    }
    let mut out = Vec::with_capacity(chars.len() / 2);
    for pair in chars.chunks(2) {
        let hi = pair[0]
            .to_digit(16)
            .ok_or_else(|| CoreError::Other("invalid hex character".into()))?;
        let lo = pair[1]
            .to_digit(16)
            .ok_or_else(|| CoreError::Other("invalid hex character".into()))?;
        out.push(((hi << 4) | lo) as u8);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip() {
        let bytes = [0x00, 0x0f, 0xa5, 0xff];
        assert_eq!(hex_encode(&bytes), "000fa5ff");
        assert_eq!(hex_decode("000FA5FF").unwrap_or_default(), bytes);
    }

    #[test]
    fn hex_decode_rejects_bad_input() {
        assert!(hex_decode("abc").is_err());
        assert!(hex_decode("zz").is_err());
    }
}
