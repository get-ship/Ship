//! Bundle attestation — verify primitive (slice 1 of registry distribution).
//!
//! A **bundle attestation** is a detached EdDSA JWT signed by the *publisher*
//! key. Its claim binds a package coordinate (`path`, `version`) to the
//! content hash of the bundle tarball (`sha256`, the same hashing as
//! [`crate::registry::hash`]). Verification proves the tarball came from a
//! trusted publisher and was not tampered with in transit.
//!
//! This module is the pure, offline-testable core: given the computed tarball
//! hash, the attestation JWT, and a set of trusted public keys, it returns the
//! verified claim or a typed error. It is intentionally UNWIRED in slice 1 —
//! no install/transport path calls it yet.
//!
//! Crypto mirrors [`crate::auth::attest`] verbatim — one EdDSA/JWT story, no
//! new dependency: `jsonwebtoken` with `DecodingKey::from_ed_components` and
//! `Validation::new(Algorithm::EdDSA)`. Bundles do not expire, so expiry
//! validation is disabled and no aud/iss is required in this slice.

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};

/// Baked-in first-party publisher key id.
///
/// PLACEHOLDER — the real first-party signing key is generated and custodied by
/// release-engineering (see design doc, "Out of scope"). This constant exists
/// so the offline/bundled-supplemental verify path has a stable `kid` to select
/// against once the real key is baked in. It is an obviously-fake value and
/// MUST be replaced before any first-party bundle is gated on it.
// TODO(release-eng): replace with the real first-party Ed25519 pubkey kid + x.
pub const FIRST_PARTY_KID: &str = "ship-first-party-PLACEHOLDER-do-not-trust";

/// Verified claim carried by a bundle attestation JWT.
///
/// `sha256` is the content hash of the *unsigned* tarball (detached signature);
/// the verify step requires it to equal the independently computed hash.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BundleClaim {
    pub path: String,
    pub version: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
}

/// An Ed25519 public key trusted to sign bundle attestations.
///
/// `x_base64url` is the JWK `x` parameter: the raw 32-byte Ed25519 public key,
/// base64url-encoded with no padding (same encoding as the JWKS `x` field in
/// [`crate::auth::attest`]).
#[derive(Debug, Clone)]
pub struct TrustedKey {
    pub kid: String,
    pub x_base64url: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BundleVerifyError {
    /// JWT header/body could not be parsed, or the key material was unusable.
    Malformed,
    /// The JWT's `kid` did not match any trusted key.
    UntrustedKey,
    /// The signature did not verify against the selected trusted key.
    SignatureInvalid,
    /// The claimed `sha256` did not match the computed tarball hash.
    HashMismatch,
}

impl std::fmt::Display for BundleVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleVerifyError::Malformed => write!(f, "attestation malformed"),
            BundleVerifyError::UntrustedKey => write!(f, "attestation signed by an untrusted key"),
            BundleVerifyError::SignatureInvalid => write!(f, "attestation signature invalid"),
            BundleVerifyError::HashMismatch => {
                write!(f, "attestation hash does not match tarball")
            }
        }
    }
}

impl std::error::Error for BundleVerifyError {}

/// Verify a detached bundle attestation against the computed tarball hash.
///
/// Steps (mirrors `auth::attest::verify_with_cache`):
/// 1. Parse the JWT header; select the [`TrustedKey`] by `kid` (or the first
///    trusted key when the header carries no `kid`). No match → `UntrustedKey`.
/// 2. Build the EdDSA [`DecodingKey`] from the trusted key's raw `x`.
/// 3. Decode + verify with `Algorithm::EdDSA`, expiry disabled (bundles don't
///    expire); no aud/iss requirement in this slice.
/// 4. Require `claim.sha256 == tarball_sha256` → else `HashMismatch`.
pub fn verify_bundle_attestation(
    tarball_sha256: &str,
    jwt: &str,
    trusted_keys: &[TrustedKey],
) -> Result<BundleClaim, BundleVerifyError> {
    let header = decode_header(jwt).map_err(|_| BundleVerifyError::Malformed)?;

    // Select the trusted key by kid, or the first when the header has none.
    let trusted = match &header.kid {
        Some(kid) => trusted_keys.iter().find(|k| k.kid == *kid),
        None => trusted_keys.first(),
    }
    .ok_or(BundleVerifyError::UntrustedKey)?;

    let decoding_key = DecodingKey::from_ed_components(&trusted.x_base64url)
        .map_err(|_| BundleVerifyError::Malformed)?;

    let mut validation = Validation::new(Algorithm::EdDSA);
    // Bundles don't expire — the claim carries no `exp`. Keep verification
    // minimal for slice 1: signature + hash binding only.
    validation.validate_exp = false;
    validation.required_spec_claims.clear();

    let token = decode::<BundleClaim>(jwt, &decoding_key, &validation).map_err(|e| {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                BundleVerifyError::SignatureInvalid
            }
            _ => BundleVerifyError::Malformed,
        }
    })?;
    let claim = token.claims;

    if claim.sha256 != tarball_sha256 {
        return Err(BundleVerifyError::HashMismatch);
    }

    Ok(claim)
}

/// Verify a downloaded bundle tarball against its attestation — the daemon's
/// trust gate before a fetched bundle enters the cache.
///
/// The attestation binds the **transport artifact** (the tarball bytes,
/// npm-style integrity), so this recomputes the content hash from the *actual
/// bytes received* and requires the attestation to match. A registry/CDN that
/// swaps or corrupts the bytes can't produce a matching publisher signature, so
/// tampering surfaces as `HashMismatch` (bytes changed) or `SignatureInvalid`
/// (forged attestation). The caller downloads from the gateway's signed
/// `download_url`, then calls this BEFORE trusting or caching the bytes.
pub fn verify_downloaded_bundle(
    bytes: &[u8],
    attestation_jwt: &str,
    trusted_keys: &[TrustedKey],
) -> Result<BundleClaim, BundleVerifyError> {
    let computed = crate::registry::hash::content_hash_bytes(bytes);
    verify_bundle_attestation(&computed, attestation_jwt, trusted_keys)
}

/// Map a JWKS document to the trusted bundle-publisher key set.
///
/// Parses `{ "keys": [ { "kty": "OKP", "crv": "Ed25519", "x": "...", "kid":
/// "..." } ] }` and keeps only Ed25519 keys carrying a `kid`. The daemon
/// assembles its full trust anchor as the baked-in first-party key
/// ([`FIRST_PARTY_KID`]) UNION the publisher keys this returns (fetched from the
/// gateway JWKS, the same source `auth::attest` uses). Malformed JWKS → empty.
pub fn trusted_keys_from_jwks(jwks_json: &str) -> Vec<TrustedKey> {
    #[derive(serde::Deserialize)]
    struct Jwk {
        kid: Option<String>,
        kty: Option<String>,
        crv: Option<String>,
        x: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct Jwks {
        #[serde(default)]
        keys: Vec<Jwk>,
    }
    let Ok(parsed) = serde_json::from_str::<Jwks>(jwks_json) else {
        return Vec::new();
    };
    parsed
        .keys
        .into_iter()
        .filter_map(|k| {
            let is_ed = k.kty.as_deref() == Some("OKP") && k.crv.as_deref() == Some("Ed25519");
            match (is_ed, k.kid, k.x) {
                (true, Some(kid), Some(x)) => Some(TrustedKey { kid, x_base64url: x }),
                _ => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header};

    // Test-only Ed25519 keypairs, generated offline with
    // `openssl genpkey -algorithm ed25519`. The trusted-key `x` (raw 32-byte
    // pubkey, base64url-no-pad) was derived from the PEM via:
    //   openssl pkey -pubout -outform DER | tail -c 32 | basenc --base64url
    // (the trailing 32 DER bytes are the raw Ed25519 point; the leading 12
    // bytes are the fixed SPKI prefix). These live only in this test module.
    const TEST_KID: &str = "test-publisher-1";
    const TEST_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
        MC4CAQAwBQYDK2VwBCIEIAFISiAspm6fXMD6Gg3dlW8qtiRFxoiHbiCjCuys/rg5\n\
        -----END PRIVATE KEY-----\n";
    const TEST_PUB_X: &str = "mZAmshQk4uLsAab8Rhqqt9kJGbP9F6hZZ39eyfUncAQ";

    // A second, unrelated keypair — used to sign with a key that is NOT in the
    // trusted set (same kid → signature must fail to verify).
    const OTHER_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
        MC4CAQAwBQYDK2VwBCIEIOKTdLdQ37vqn0+6MYF8iMq3vrfGWl/d9nZWeYoMfIww\n\
        -----END PRIVATE KEY-----\n";

    fn trusted() -> Vec<TrustedKey> {
        vec![TrustedKey { kid: TEST_KID.to_string(), x_base64url: TEST_PUB_X.to_string() }]
    }

    /// Sign a `BundleClaim` JWT with the given Ed25519 PKCS8 PEM and `kid`.
    fn sign(claim: &BundleClaim, priv_pem: &str, kid: &str) -> String {
        let key = EncodingKey::from_ed_pem(priv_pem.as_bytes()).expect("valid ed25519 pem");
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(kid.to_string());
        jsonwebtoken::encode(&header, claim, &key).expect("sign jwt")
    }

    fn sample_claim(sha256: &str) -> BundleClaim {
        BundleClaim {
            path: "ship/themes/midnight".to_string(),
            version: "1.2.0".to_string(),
            sha256: sha256.to_string(),
            kid: Some(TEST_KID.to_string()),
            iat: Some(1_700_000_000),
        }
    }

    #[test]
    fn valid_attestation_with_matching_hash_verifies() {
        let hash = "a".repeat(64);
        let claim = sample_claim(&hash);
        let jwt = sign(&claim, TEST_PRIV_PEM, TEST_KID);

        let verified = verify_bundle_attestation(&hash, &jwt, &trusted()).expect("verify ok");
        assert_eq!(verified, claim);
        assert_eq!(verified.path, "ship/themes/midnight");
        assert_eq!(verified.version, "1.2.0");
    }

    #[test]
    fn tampered_tarball_hash_mismatch() {
        let signed_hash = "a".repeat(64);
        let claim = sample_claim(&signed_hash);
        let jwt = sign(&claim, TEST_PRIV_PEM, TEST_KID);

        // Verify against a DIFFERENT tarball hash — signature is valid but the
        // content binding is broken.
        let actual_hash = "b".repeat(64);
        let err = verify_bundle_attestation(&actual_hash, &jwt, &trusted()).unwrap_err();
        assert_eq!(err, BundleVerifyError::HashMismatch);
    }

    #[test]
    fn signature_by_untrusted_key_rejected() {
        let hash = "c".repeat(64);
        let claim = sample_claim(&hash);
        // Signed by OTHER key but carrying the trusted kid → key is selected,
        // signature fails to verify.
        let jwt = sign(&claim, OTHER_PRIV_PEM, TEST_KID);

        let err = verify_bundle_attestation(&hash, &jwt, &trusted()).unwrap_err();
        assert_eq!(err, BundleVerifyError::SignatureInvalid);
    }

    #[test]
    fn unknown_kid_rejected() {
        let hash = "d".repeat(64);
        let claim = sample_claim(&hash);
        let jwt = sign(&claim, TEST_PRIV_PEM, "some-unknown-kid");

        let err = verify_bundle_attestation(&hash, &jwt, &trusted()).unwrap_err();
        assert_eq!(err, BundleVerifyError::UntrustedKey);
    }

    #[test]
    fn jwks_maps_ed25519_keys_and_skips_others() {
        let jwks = format!(
            r#"{{"keys":[
                {{"kty":"OKP","crv":"Ed25519","kid":"{TEST_KID}","x":"{TEST_PUB_X}"}},
                {{"kty":"RSA","kid":"rsa-1","n":"...","e":"AQAB"}},
                {{"kty":"OKP","crv":"X25519","kid":"x-1","x":"abc"}}
            ]}}"#
        );
        let keys = trusted_keys_from_jwks(&jwks);
        assert_eq!(keys.len(), 1, "only the Ed25519 signing key is trusted");
        assert_eq!(keys[0].kid, TEST_KID);
        assert_eq!(keys[0].x_base64url, TEST_PUB_X);
        // And a key from JWKS actually verifies a real attestation.
        let hash = "e".repeat(64);
        let jwt = sign(&sample_claim(&hash), TEST_PRIV_PEM, TEST_KID);
        assert!(verify_bundle_attestation(&hash, &jwt, &keys).is_ok());
    }

    #[test]
    fn malformed_jwks_yields_no_keys() {
        assert!(trusted_keys_from_jwks("not json").is_empty());
        assert!(trusted_keys_from_jwks(r#"{"keys":[]}"#).is_empty());
    }

    #[test]
    fn downloaded_bundle_verifies_against_its_bytes() {
        // The attestation is signed over the hash of the ACTUAL bytes — the
        // daemon recomputes from what it received and the binding holds.
        let bytes = b"a pretend bundle tarball's bytes";
        let hash = crate::registry::hash::content_hash_bytes(bytes);
        let claim = sample_claim(hash.trim_start_matches("sha256:"));
        // sample_claim stores the bare hex; verify recomputes the `sha256:` form
        // — keep the claim's sha256 in the same shape as content_hash_bytes.
        let claim = BundleClaim { sha256: hash.clone(), ..claim };
        let jwt = sign(&claim, TEST_PRIV_PEM, TEST_KID);

        let verified =
            verify_downloaded_bundle(bytes, &jwt, &trusted()).expect("downloaded bundle verifies");
        assert_eq!(verified.sha256, hash);
    }

    #[test]
    fn downloaded_bundle_tampered_bytes_rejected() {
        // Attestation signed over the original bytes; the daemon receives
        // DIFFERENT bytes (a swap/corruption) → the recomputed hash won't match.
        let original = b"the genuine bundle bytes";
        let hash = crate::registry::hash::content_hash_bytes(original);
        let claim = BundleClaim { sha256: hash, ..sample_claim("") };
        let jwt = sign(&claim, TEST_PRIV_PEM, TEST_KID);

        let tampered = b"swapped malicious bytes";
        let err = verify_downloaded_bundle(tampered, &jwt, &trusted()).unwrap_err();
        assert_eq!(err, BundleVerifyError::HashMismatch);
    }
}
