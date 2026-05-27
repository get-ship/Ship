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
