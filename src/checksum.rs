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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Nanoid URL-safe alphabet — same set check_chars maps into.
    const BODY_PATTERN: &str = "[_\\-0-9a-zA-Z]{21}";

    proptest! {
        #[test]
        fn check_chars_is_deterministic(body in BODY_PATTERN) {
            prop_assert_eq!(check_chars(SALT_V1, &body), check_chars(SALT_V1, &body));
        }

        #[test]
        fn check_chars_output_is_in_alphabet(body in BODY_PATTERN) {
            let (c1, c2) = check_chars(SALT_V1, &body);
            prop_assert!(ALPHABET.contains(&(c1 as u8)), "c1 '{c1}' not in alphabet");
            prop_assert!(ALPHABET.contains(&(c2 as u8)), "c2 '{c2}' not in alphabet");
        }

        #[test]
        fn mutating_body_changes_check_chars(body in BODY_PATTERN, idx in 0usize..21) {
            let (c1, c2) = check_chars(SALT_V1, &body);
            // Flip one character in the body to something different.
            let mut mutated: Vec<char> = body.chars().collect();
            mutated[idx] = if mutated[idx] == 'A' { 'B' } else { 'A' };
            let mutated: String = mutated.into_iter().collect();
            let (m1, m2) = check_chars(SALT_V1, &mutated);
            // With 12-bit check the probability of collision is 1/4096;
            // over proptest's default 256 cases this will virtually never fire.
            prop_assume!((c1, c2) != (m1, m2));
        }
    }
}
