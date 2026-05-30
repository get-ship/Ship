//! Install a fetched bundle — the extract-verify-or-discard gate (slice 2c).
//!
//! The attestation binds the bundle's **tree hash** (`compute_tree_hash` over
//! the extracted content) — the SAME hash the cache content-addresses by and
//! the registry stores as `contentHash`. One canonical content hash across
//! registry ↔ attestation ↔ cache; the daemon never trusts a second hash.
//!
//! Because the attested identity is the *tree* (not the transport tarball),
//! verification runs after extraction into an ISOLATED directory: extract →
//! `compute_tree_hash` → verify the publisher attestation over that hash. On
//! any failure the directory is wiped, so unverified content never reaches the
//! cache. The caller (the daemon fetch op) extracts into a temp dir, calls
//! this, then on success hands the dir to [`crate::registry::cache::PackageCache::store`]
//! (which re-hashes to the identical tree hash).

use std::path::Path;

use super::bundle_sig::{verify_bundle_attestation, BundleClaim, BundleVerifyError, TrustedKey};
use super::hash::compute_tree_hash;

#[derive(Debug)]
pub enum InstallBundleError {
    /// The (verified-pending) tarball could not be extracted.
    Extract(String),
    /// The tree hash of the extracted content could not be computed.
    Hash(String),
    /// The bundle's attestation did not verify against its tree hash.
    Verify(BundleVerifyError),
}

impl std::fmt::Display for InstallBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallBundleError::Extract(e) => write!(f, "bundle extraction failed: {e}"),
            InstallBundleError::Hash(e) => write!(f, "bundle tree-hash failed: {e}"),
            InstallBundleError::Verify(e) => write!(f, "bundle verification failed: {e}"),
        }
    }
}

impl std::error::Error for InstallBundleError {}

/// Extract the gzip-tarball `bytes` into `dest`, then verify the publisher
/// attestation against the extracted tree's hash. Returns the verified
/// [`BundleClaim`].
///
/// `dest` must be an isolated, caller-owned directory (e.g. a temp dir). On ANY
/// failure — extraction, hashing, or a bad/forged attestation — `dest`'s
/// contents are removed before returning, so a verify failure never leaves
/// unverified files behind for the caller to mistakenly cache.
pub fn extract_and_verify_bundle(
    bytes: &[u8],
    attestation_jwt: &str,
    trusted_keys: &[TrustedKey],
    dest: &Path,
) -> Result<BundleClaim, InstallBundleError> {
    let result = extract_and_verify_inner(bytes, attestation_jwt, trusted_keys, dest);
    if result.is_err() {
        // Wipe any partially-extracted / unverified content. Best-effort: the
        // caller owns `dest` and discards it on Err regardless.
        if let Ok(entries) = std::fs::read_dir(dest) {
            for entry in entries.flatten() {
                let p = entry.path();
                let _ = if p.is_dir() {
                    std::fs::remove_dir_all(&p)
                } else {
                    std::fs::remove_file(&p)
                };
            }
        }
    }
    result
}

fn extract_and_verify_inner(
    bytes: &[u8],
    attestation_jwt: &str,
    trusted_keys: &[TrustedKey],
    dest: &Path,
) -> Result<BundleClaim, InstallBundleError> {
    // Extract into the isolated dir. `tar`'s `unpack` rejects `..` and absolute
    // paths, so a malicious archive can't escape `dest`; anything it does write
    // is discarded on the verify failure below.
    let gz = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(gz);
    archive
        .unpack(dest)
        .map_err(|e| InstallBundleError::Extract(e.to_string()))?;

    // Hash the extracted tree — the canonical content identity.
    let tree_hash = compute_tree_hash(dest).map_err(|e| InstallBundleError::Hash(e.to_string()))?;

    // The attestation must bind exactly this tree hash.
    verify_bundle_attestation(&tree_hash, attestation_jwt, trusted_keys)
        .map_err(InstallBundleError::Verify)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{Algorithm, EncodingKey, Header};

    // Reuse the slice-1 test publisher key (offline-generated Ed25519).
    const TEST_KID: &str = "test-publisher-1";
    const TEST_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
        MC4CAQAwBQYDK2VwBCIEIAFISiAspm6fXMD6Gg3dlW8qtiRFxoiHbiCjCuys/rg5\n\
        -----END PRIVATE KEY-----\n";
    const TEST_PUB_X: &str = "mZAmshQk4uLsAab8Rhqqt9kJGbP9F6hZZ39eyfUncAQ";

    fn trusted() -> Vec<TrustedKey> {
        vec![TrustedKey { kid: TEST_KID.to_string(), x_base64url: TEST_PUB_X.to_string() }]
    }

    /// Build an in-memory gzip tarball with one file, returning the bytes.
    fn make_bundle(body: &[u8]) -> Vec<u8> {
        let mut tar_buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_buf);
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, "views/x/page.sui", body).unwrap();
            builder.finish().unwrap();
        }
        let mut gz = Vec::new();
        {
            use std::io::Write;
            let mut enc = flate2::write::GzEncoder::new(&mut gz, flate2::Compression::default());
            enc.write_all(&tar_buf).unwrap();
            enc.finish().unwrap();
        }
        gz
    }

    /// The tree hash a bundle WILL produce once extracted — computed by
    /// extracting into a throwaway dir (the publisher/gateway does the same to
    /// know what to attest).
    fn tree_hash_of(bytes: &[u8]) -> String {
        let tmp = tempfile::tempdir().unwrap();
        let gz = flate2::read::GzDecoder::new(bytes);
        tar::Archive::new(gz).unpack(tmp.path()).unwrap();
        compute_tree_hash(tmp.path()).unwrap()
    }

    fn sign_tree_hash(tree_hash: &str) -> String {
        let claim = BundleClaim {
            path: "ship/apps/notes".to_string(),
            version: "1.0.0".to_string(),
            sha256: tree_hash.to_string(),
            kid: Some(TEST_KID.to_string()),
            iat: Some(1_700_000_000),
        };
        let key = EncodingKey::from_ed_pem(TEST_PRIV_PEM.as_bytes()).unwrap();
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(TEST_KID.to_string());
        jsonwebtoken::encode(&header, &claim, &key).unwrap()
    }

    #[test]
    fn verified_bundle_extracts_and_returns_claim() {
        let bundle = make_bundle(b"<p>hi from a dep view</p>");
        let jwt = sign_tree_hash(&tree_hash_of(&bundle));
        let dest = tempfile::tempdir().unwrap();

        let claim = extract_and_verify_bundle(&bundle, &jwt, &trusted(), dest.path())
            .expect("verified bundle");
        assert_eq!(claim.path, "ship/apps/notes");
        assert!(dest.path().join("views/x/page.sui").is_file(), "tree extracted + kept");
    }

    #[test]
    fn tampered_bundle_rejected_and_wiped() {
        // Attestation is over the ORIGINAL tree; the daemon receives a bundle
        // whose content differs → the extracted tree hashes differently.
        let original = make_bundle(b"the genuine view");
        let jwt = sign_tree_hash(&tree_hash_of(&original));
        let tampered = make_bundle(b"malicious swapped view");
        let dest = tempfile::tempdir().unwrap();

        let err = extract_and_verify_bundle(&tampered, &jwt, &trusted(), dest.path())
            .expect_err("tampered bundle rejected");
        assert!(matches!(err, InstallBundleError::Verify(BundleVerifyError::HashMismatch)));
        assert!(
            std::fs::read_dir(dest.path()).unwrap().next().is_none(),
            "unverified content wiped — nothing left to cache"
        );
    }
}
