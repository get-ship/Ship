//! Install a fetched bundle — the verify-THEN-extract gate (slice 2c).
//!
//! Verification gates extraction: a downloaded bundle's attestation is checked
//! against the actual bytes BEFORE anything is written to disk, so unverified
//! or tampered bytes never reach the cache. The daemon's fetch op downloads the
//! tarball from the gateway's signed `download_url`, calls
//! [`verify_and_extract_bundle`], then hands the extracted tree to
//! [`crate::registry::cache::PackageCache::store`] (which content-addresses it
//! by tree hash). Bundles are gzip tarballs (`.tar.gz`), reusing the same
//! extraction path as `fetch.rs`.

use std::path::Path;

use super::bundle_sig::{verify_downloaded_bundle, BundleClaim, BundleVerifyError, TrustedKey};

#[derive(Debug)]
pub enum InstallBundleError {
    /// The bundle's attestation did not verify against its bytes.
    Verify(BundleVerifyError),
    /// The (verified) tarball could not be extracted.
    Extract(String),
}

impl std::fmt::Display for InstallBundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallBundleError::Verify(e) => write!(f, "bundle verification failed: {e}"),
            InstallBundleError::Extract(e) => write!(f, "bundle extraction failed: {e}"),
        }
    }
}

impl std::error::Error for InstallBundleError {}

/// Verify `bytes` against `attestation_jwt`, then extract the gzip tarball into
/// `dest`. Returns the verified [`BundleClaim`].
///
/// Verification runs FIRST: on any verify failure nothing is extracted, so the
/// caller's `dest` stays empty and no unverified bytes touch the cache. Only
/// after the publisher attestation is confirmed over the exact bytes received
/// is the archive unpacked.
pub fn verify_and_extract_bundle(
    bytes: &[u8],
    attestation_jwt: &str,
    trusted_keys: &[TrustedKey],
    dest: &Path,
) -> Result<BundleClaim, InstallBundleError> {
    let claim = verify_downloaded_bundle(bytes, attestation_jwt, trusted_keys)
        .map_err(InstallBundleError::Verify)?;

    // Verified — safe to extract.
    let gz = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(gz);
    archive
        .unpack(dest)
        .map_err(|e| InstallBundleError::Extract(e.to_string()))?;

    Ok(claim)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::hash::content_hash_bytes;
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

    /// Build an in-memory gzip tarball containing `views/x/page.sui` so the
    /// extracted tree is inspectable.
    fn make_bundle() -> Vec<u8> {
        let mut tar_buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_buf);
            let contents = b"<p>hi from a dep view</p>";
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "views/x/page.sui", &contents[..])
                .unwrap();
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

    fn sign_for(bytes: &[u8]) -> String {
        let claim = BundleClaim {
            path: "ship/apps/notes".to_string(),
            version: "1.0.0".to_string(),
            sha256: content_hash_bytes(bytes),
            kid: Some(TEST_KID.to_string()),
            iat: Some(1_700_000_000),
        };
        let key = EncodingKey::from_ed_pem(TEST_PRIV_PEM.as_bytes()).unwrap();
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(TEST_KID.to_string());
        jsonwebtoken::encode(&header, &claim, &key).unwrap()
    }

    #[test]
    fn verified_bundle_extracts() {
        let bundle = make_bundle();
        let jwt = sign_for(&bundle);
        let dest = tempfile::tempdir().unwrap();

        let claim = verify_and_extract_bundle(&bundle, &jwt, &trusted(), dest.path())
            .expect("verified bundle extracts");
        assert_eq!(claim.path, "ship/apps/notes");
        assert!(dest.path().join("views/x/page.sui").is_file(), "tree extracted");
    }

    #[test]
    fn tampered_bundle_is_not_extracted() {
        let bundle = make_bundle();
        let jwt = sign_for(&bundle); // attestation over the genuine bytes
        let dest = tempfile::tempdir().unwrap();

        // Append a byte → different bytes than the attestation signed.
        let mut tampered = bundle.clone();
        tampered.push(0x00);

        let err = verify_and_extract_bundle(&tampered, &jwt, &trusted(), dest.path())
            .expect_err("tampered bundle rejected");
        assert!(matches!(err, InstallBundleError::Verify(BundleVerifyError::HashMismatch)));
        // Nothing was written — verification gated extraction.
        assert!(
            std::fs::read_dir(dest.path()).unwrap().next().is_none(),
            "no files extracted from an unverified bundle"
        );
    }
}
