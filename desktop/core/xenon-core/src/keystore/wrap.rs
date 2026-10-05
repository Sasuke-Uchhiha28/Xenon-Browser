//! Optional master-password wrap for the database key (Architecture.md
//! 6.1 `keystore`). The key is encrypted with AES-256-GCM under a key
//! derived from the master password with Argon2id (OWASP m=19456 KiB,
//! t=2, p=1). The wrapped blob is what the keystore stores when the user
//! opts into a master password; without one, the key is stored directly.

use crate::{CoreError, Result};
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// Argon2id cost parameters (OWASP recommended defaults for interactive
/// use): 19 MiB memory, 2 iterations, 1 lane, 32-byte output.
const ARGON_MEMORY_KIB: u32 = 19_456;
const ARGON_ITERATIONS: u32 = 2;
const ARGON_PARALLELISM: u32 = 1;
const ARGON_OUTPUT_BYTES: usize = 32;
const SALT_BYTES: usize = 16;
const NONCE_BYTES: usize = 12;

/// The stored form of a wrapped database key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WrappedKey {
    /// Base64 Argon2 salt.
    pub salt: String,
    /// Base64 AES-GCM nonce.
    pub nonce: String,
    /// Base64 AES-GCM ciphertext of the key.
    pub ciphertext: String,
}

fn derive_key(master_password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; ARGON_OUTPUT_BYTES]>> {
    let argon = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(
            ARGON_MEMORY_KIB,
            ARGON_ITERATIONS,
            ARGON_PARALLELISM,
            Some(ARGON_OUTPUT_BYTES),
        )
        .map_err(|err| CoreError::Crypto(format!("argon2 parameters rejected: {err}")))?,
    );
    let mut okm = Zeroizing::new([0u8; ARGON_OUTPUT_BYTES]);
    argon
        .hash_password_into(master_password.as_bytes(), salt, &mut *okm)
        .map_err(|err| CoreError::Crypto(format!("key derivation failed: {err}")))?;
    Ok(okm)
}

fn random_bytes(len: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; len];
    getrandom::fill(&mut bytes).map_err(|err| CoreError::Crypto(format!("rng failed: {err}")))?;
    Ok(bytes)
}

/// Wrap the database key under `master_password`.
pub fn wrap(db_key: &[u8], master_password: &str) -> Result<WrappedKey> {
    let salt = random_bytes(SALT_BYTES)?;
    let nonce = random_bytes(NONCE_BYTES)?;
    let derived = derive_key(master_password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(derived.as_slice())
        .map_err(|err| CoreError::Crypto(err.to_string()))?;
    let nonce_array =
        Nonce::try_from(nonce.as_slice()).map_err(|_| CoreError::Crypto("nonce length".into()))?;
    let ciphertext = cipher
        .encrypt(&nonce_array, db_key)
        .map_err(|err| CoreError::Crypto(format!("encryption failed: {err}")))?;
    Ok(WrappedKey {
        salt: BASE64.encode(salt),
        nonce: BASE64.encode(nonce),
        ciphertext: BASE64.encode(ciphertext),
    })
}

/// Recover the database key. Fails on a wrong password or a tampered
/// blob (AES-GCM authentication), without leaking which one it was.
pub fn unwrap(wrapped: &WrappedKey, master_password: &str) -> Result<Zeroizing<Vec<u8>>> {
    let salt = BASE64
        .decode(&wrapped.salt)
        .map_err(|_| CoreError::Crypto("wrapped key is corrupt".into()))?;
    let nonce = BASE64
        .decode(&wrapped.nonce)
        .map_err(|_| CoreError::Crypto("wrapped key is corrupt".into()))?;
    let ciphertext = BASE64
        .decode(&wrapped.ciphertext)
        .map_err(|_| CoreError::Crypto("wrapped key is corrupt".into()))?;
    let derived = derive_key(master_password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(derived.as_slice())
        .map_err(|err| CoreError::Crypto(err.to_string()))?;
    let nonce_array =
        Nonce::try_from(nonce.as_slice()).map_err(|_| CoreError::Crypto("nonce length".into()))?;
    let plaintext = cipher
        .decrypt(&nonce_array, ciphertext.as_slice())
        .map_err(|_| CoreError::Crypto("unwrap failed: wrong password or corrupt blob".into()))?;
    Ok(Zeroizing::new(plaintext))
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &[u8] = &[42u8; 32];

    #[test]
    fn wrap_unwrap_round_trip() {
        let wrapped = wrap(KEY, "correct horse battery staple").expect("wrap");
        let recovered = unwrap(&wrapped, "correct horse battery staple").expect("unwrap");
        assert_eq!(recovered.as_slice(), KEY);
    }

    #[test]
    fn wrong_password_fails() {
        let wrapped = wrap(KEY, "right password").expect("wrap");
        assert!(unwrap(&wrapped, "wrong password").is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let mut wrapped = wrap(KEY, "password").expect("wrap");
        let mut bytes = BASE64.decode(&wrapped.ciphertext).expect("decode");
        bytes[0] ^= 0x01;
        wrapped.ciphertext = BASE64.encode(bytes);
        assert!(unwrap(&wrapped, "password").is_err());
    }

    #[test]
    fn wraps_are_unique_per_call() {
        let first = wrap(KEY, "password").expect("wrap");
        let second = wrap(KEY, "password").expect("wrap");
        assert_ne!(first, second, "random salt and nonce must differ");
        // Both still unwrap to the same key.
        assert_eq!(unwrap(&first, "password").expect("unwrap").as_slice(), KEY);
    }

    #[test]
    fn blob_never_contains_plaintext_key() {
        let wrapped = wrap(KEY, "password").expect("wrap");
        let blob = format!("{}{}{}", wrapped.salt, wrapped.nonce, wrapped.ciphertext);
        let raw_key = BASE64.encode(KEY);
        assert!(!blob.contains(&raw_key), "the plain key must not be stored");
    }
}
