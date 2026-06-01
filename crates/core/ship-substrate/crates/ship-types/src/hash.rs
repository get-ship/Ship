use sha2::{Digest, Sha256};

/// Compute SHA-256 of bytes, returning a lowercase hex string.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    format!("{hash:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_known_vector() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
