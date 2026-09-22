//! Shared percent-encoder for registry URL paths.
//!
//! Encodes everything except unreserved characters (RFC 3986):
//! ALPHA / DIGIT / "-" / "." / "_" / "~".
//!
//! Used by `tracking::track_install`, `resolver::try_registry_api_alias`, and
//! `verify::fetch_registry_hash` for the `/api/registry/...` URL surface.
//!
//! Note: this is NOT the same as `cache::url_encode`, which omits `~` (encoding
//! it as `%7E`). The cache variant builds on-disk index keys whose byte shape
//! must remain stable across releases — its omission of `~` is intentional and
//! changing it would invalidate existing caches.

/// Percent-encode a string for use in a registry URL path.
///
/// Preserves the RFC 3986 unreserved set (`A-Z a-z 0-9 - _ . ~`) verbatim and
/// encodes every other byte as `%XX`. There is no special-case for `/`: paths
/// passed to this function are expected to be already-path-shaped values like
/// `github.com/owner/repo`, where `/` is part of the path the server expects to
/// receive encoded as `%2F`.
pub fn percent_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encode_simple_path() {
        assert_eq!(
            percent_encode("github.com/owner/repo"),
            "github.com%2Fowner%2Frepo"
        );
    }

    #[test]
    fn percent_encode_preserves_unreserved() {
        assert_eq!(percent_encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn percent_encode_encodes_special_chars() {
        assert_eq!(percent_encode("a b@c"), "a%20b%40c");
    }
}
