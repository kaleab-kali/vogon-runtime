use sha2::{Digest, Sha256};

/// Computes a lowercase SHA-256 hex digest for stable replay hashing.
pub fn stable_hash(input: impl AsRef<[u8]>) -> String {
    let digest = Sha256::digest(input.as_ref());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::stable_hash;

    #[test]
    fn stable_hash_is_repeatable() {
        assert_eq!(stable_hash("vogon"), stable_hash("vogon"));
    }
}
