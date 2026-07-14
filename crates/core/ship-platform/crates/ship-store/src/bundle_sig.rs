//! Bundle attestation — verify primitive (slice 1 of registry distribution).
//!
//! A **bundle attestation** is a detached EdDSA JWT signed by the *publisher*
//! key. Its claim binds a package coordinate (`path`, `version`) to the
//! content hash of the bundle tarball (`sha256`, the same hashing as
//! [`crate::hash`]). Verification proves the tarball came from a
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

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, decode_header, encode,
};
use serde::{Deserialize, Serialize};

/// Baked-in first-party publisher key id.
///
/// PLACEHOLDER — the real first-party signing key is generated and custodied by
/// release-engineering (see design doc, "Out of scope"). This constant exists
/// so the offline/bundled-supplemental verify path has a stable `kid` to select
/// against once the real key is baked in. It is an obviously-fake value and
/// MUST be replaced before any first-party bundle is gated on it.
///
/// # Trust set at verify time
///
/// The install-side verifier
/// ([`crate::install_bundle::extract_and_verify_bundle`]) accepts a bundle if
/// its attestation JWT was signed by ANY key in the `TrustedKey` set the
/// caller passes in. The daemon assembles that set as the UNION of:
///
/// 1. The baked-in first-party public key, identified by this constant
///    ([`FIRST_PARTY_KID`]). When the real key is provisioned, the matching
///    `x_base64url` is baked into the same release that hard-codes the kid —
///    they ship together. Today the constant is a PLACEHOLDER, so this entry
///    is intentionally untrusted: the runtime guard in
///    [`verify_bundle_attestation`] rejects attestations that select it.
/// 2. Per-publisher keys fetched from the gateway JWKS
///    ([`trusted_keys_from_jwks`] — same source `auth::attest` uses).
///
/// In other words, a first-party-signed bundle (one whose JWT carries a `kid`
/// equal to [`FIRST_PARTY_KID`]) is trusted only when:
///   - this constant is no longer the PLACEHOLDER, AND
///   - the matching `x_base64url` is also baked into the install-time trust
///     set alongside it.
///
/// Until release-engineering provisions both halves, first-party bundles
/// minted by [`crate::publish_first_party::publish_first_party`] (or
/// [`sign_bundle_attestation`] directly) only verify when the verifier is
/// given a corresponding `TrustedKey` explicitly — i.e. through the per-call
/// `trusted_keys` argument, not through the baked-in anchor. The integration
/// test suite uses this path today.
///
/// # Release gate
///
/// When building with `--features release` (required for official release cuts),
/// a compile-time assertion fires if this constant still contains `"PLACEHOLDER"`.
/// Dev builds (no feature) compile without restriction.
///
/// Replace this string with the real Ed25519 pubkey kid before enabling `release`.
// TODO(release-eng): replace with the real first-party Ed25519 pubkey kid + x,
// and add the matching public `x_base64url` to a corresponding
// `FIRST_PARTY_PUB_X` constant co-located with this one.
pub const FIRST_PARTY_KID: &str = "ship-first-party-PLACEHOLDER-do-not-trust";

/// Compile-time guard: fail if FIRST_PARTY_KID still carries the PLACEHOLDER
/// stub when the `release` feature is enabled. The loop is a const-stable
/// byte-level substring check (str::contains is not const).
///
/// To trigger: `cargo build -p ship-store --features release`
/// To fix: replace FIRST_PARTY_KID with the real Ed25519 pubkey kid.
#[cfg(feature = "release")]
const _: () = {
    const KID: &[u8] = FIRST_PARTY_KID.as_bytes();
    const NEEDLE: &[u8] = b"PLACEHOLDER";
    let mut i = 0usize;
    let mut found = false;
    while i + NEEDLE.len() <= KID.len() {
        let mut j = 0usize;
        let mut matched = true;
        while j < NEEDLE.len() {
            if KID[i + j] != NEEDLE[j] {
                matched = false;
                break;
            }
            j += 1;
        }
        if matched {
            found = true;
            break;
        }
        i += 1;
    }
    if found {
        panic!(
            "FIRST_PARTY_KID still contains PLACEHOLDER; \
             replace with the real Ed25519 pubkey kid before cutting a release \
             (see TODO(release-eng) in bundle_sig.rs)"
        );
    }
};

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

    // Runtime fail-closed guard: refuse to trust the placeholder first-party
    // kid even if it somehow made it into the trusted_keys set (e.g. a stale
    // config or a malformed build that bypassed the compile-time check above).
    // The placeholder kid is, by definition, an untrusted key.
    if FIRST_PARTY_KID.contains("PLACEHOLDER") && trusted.kid == FIRST_PARTY_KID {
        return Err(BundleVerifyError::UntrustedKey);
    }

    let decoding_key = DecodingKey::from_ed_components(&trusted.x_base64url)
        .map_err(|_| BundleVerifyError::Malformed)?;

    let mut validation = Validation::new(Algorithm::EdDSA);
    // Bundles don't expire — the claim carries no `exp`. Keep verification
    // minimal for slice 1: signature + hash binding only.
    validation.validate_exp = false;
    validation.required_spec_claims.clear();

    let token =
        decode::<BundleClaim>(jwt, &decoding_key, &validation).map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                BundleVerifyError::SignatureInvalid
            }
            _ => BundleVerifyError::Malformed,
        })?;
    let claim = token.claims;

    if claim.sha256 != tarball_sha256 {
        return Err(BundleVerifyError::HashMismatch);
    }

    Ok(claim)
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
                (true, Some(kid), Some(x)) => Some(TrustedKey {
                    kid,
                    x_base64url: x,
                }),
                _ => None,
            }
        })
        .collect()
}

/// Mint a detached bundle attestation JWT.
///
/// Mirrors the test `sign` helper at the bottom of this file but is a real,
/// non-`#[cfg(test)]` entry point used by the first-party publish path
/// ([`crate::publish_first_party::publish_first_party`]). Encodes the claim
/// as a JWS with `alg = EdDSA` and a `kid` header selecting the
/// publisher/first-party key the verifier will resolve against.
///
/// Pair with [`verify_bundle_attestation`] on the install side: the claim's
/// `sha256` is the canonical tree hash of the bundle tarball; the verifier
/// computes the same hash on the extracted content and rejects any mismatch.
///
/// The PEM is the PKCS8 private key (`-----BEGIN PRIVATE KEY-----`); the
/// matching public `x` lives in the JWK published alongside the kid. Test
/// fixtures under `tests/` use the same encoding.
pub fn sign_bundle_attestation(claim: &BundleClaim, signing_key_pem: &str, kid: &str) -> String {
    let key = EncodingKey::from_ed_pem(signing_key_pem.as_bytes())
        .expect("signing_key_pem is a valid PKCS8 Ed25519 private key");
    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(kid.to_string());
    encode(&header, claim, &key).expect("signing EdDSA JWT")
}

#[cfg(test)]
mod tests {
    use super::*;

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
        vec![TrustedKey {
            kid: TEST_KID.to_string(),
            x_base64url: TEST_PUB_X.to_string(),
        }]
    }

    /// Sign a `BundleClaim` JWT with the given Ed25519 PKCS8 PEM and `kid`.
    /// Thin wrapper over the production [`sign_bundle_attestation`] so tests
    /// keep their compact call sites but exercise the real entry point.
    fn sign(claim: &BundleClaim, priv_pem: &str, kid: &str) -> String {
        sign_bundle_attestation(claim, priv_pem, kid)
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
    fn placeholder_first_party_kid_is_rejected_at_verify_time() {
        // FIRST_PARTY_KID still carries the PLACEHOLDER stub in this build
        // (the compile-time guard only fires under `--features release`).
        // The runtime guard in `verify_bundle_attestation` must refuse to
        // verify an attestation that selects the placeholder kid, even when
        // it is the only entry in `trusted_keys`.
        assert!(FIRST_PARTY_KID.contains("PLACEHOLDER"));

        let hash = "f".repeat(64);
        let claim = sample_claim(&hash);
        // Sign with the test key but claim the placeholder kid in the header,
        // so selection resolves to a TrustedKey whose kid == FIRST_PARTY_KID.
        let jwt = sign(&claim, TEST_PRIV_PEM, FIRST_PARTY_KID);

        let keys = vec![TrustedKey {
            kid: FIRST_PARTY_KID.to_string(),
            x_base64url: TEST_PUB_X.to_string(),
        }];

        let err = verify_bundle_attestation(&hash, &jwt, &keys).unwrap_err();
        assert_eq!(err, BundleVerifyError::UntrustedKey);
    }

    #[test]
    fn placeholder_kid_does_not_shadow_a_real_trusted_kid() {
        // When the trusted set contains BOTH the placeholder first-party kid
        // and a real publisher kid, the real kid must still verify — only the
        // placeholder kid is rejected by the runtime guard.
        let hash = "1".repeat(64);
        let claim = sample_claim(&hash);
        let jwt = sign(&claim, TEST_PRIV_PEM, TEST_KID);

        let keys = vec![
            TrustedKey {
                kid: FIRST_PARTY_KID.to_string(),
                x_base64url: TEST_PUB_X.to_string(),
            },
            TrustedKey {
                kid: TEST_KID.to_string(),
                x_base64url: TEST_PUB_X.to_string(),
            },
        ];

        let verified =
            verify_bundle_attestation(&hash, &jwt, &keys).expect("real kid still verifies");
        assert_eq!(verified, claim);

        // And conversely, selecting the placeholder kid is still rejected.
        let placeholder_jwt = sign(&claim, TEST_PRIV_PEM, FIRST_PARTY_KID);
        let err = verify_bundle_attestation(&hash, &placeholder_jwt, &keys).unwrap_err();
        assert_eq!(err, BundleVerifyError::UntrustedKey);
    }
}
