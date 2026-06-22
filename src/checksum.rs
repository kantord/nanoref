use sha2::{Digest, Sha256};

// nanoid default URL-safe alphabet, 64 symbols — used for the check characters.
const ALPHABET: &[u8; 64] =
    b"_-0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub const SALT_V1: &[u8] = b"nref-version-1";

/// Compute the two check characters for `body` under `salt`.
///
/// check1 = SHA-256(salt + body)[0] % 64
/// check2 = SHA-256(salt + body)[1] % 64
///
/// Two characters (12 bits) give a 1/4096 collision probability between
/// versions, making version discrimination reliable across large codebases.
pub fn check_chars(salt: &[u8], body: &str) -> (char, char) {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(body.as_bytes());
    let hash = hasher.finalize();
    (
        ALPHABET[(hash[0] % 64) as usize] as char,
        ALPHABET[(hash[1] % 64) as usize] as char,
    )
}
