//! Platform bundles — registry wave 2b: PLATFORM-tier CDN distribution.
//!
//! Space/hub scaffolds (themes, hub shells, on-ship content packs) are
//! delivered as versioned bundles from a CDN, NOT as install-pipeline packages
//! and NOT as stacks. They are mechanically a third distribution class:
//! product-delivered, signed by the **product key** (a distinct trust anchor
//! from publisher keys), and addressed by content hash.
//!
//! # Trust model
//!
//! Two trust anchors live in this slice; they MUST stay distinct.
//!
//! - **Publisher key** (see [`crate::bundle_sig`]) — per-package. Each
//!   `ship install` calls a publisher, which signs the package's tree hash.
//!   Many publishers; publishers can rotate; ships verify per fetch.
//! - **Product key** (this module, [`PRODUCT_PUBKEY`]) — single, baked-in.
//!   The ship product team signs every entry in every channel manifest. A
//!   compromised publisher key MUST NOT be able to forge a channel-manifest
//!   entry pointing at a malicious URL/hash. A single rotating key (with JWKS
//!   out of scope for this slice) is the whole control plane for "what
//!   bundles does this version of ship offer?"
//!
//! This module imports [`crate::cache`] ONLY to delegate storage of the
//! extracted tree to the existing content-addressed cache; the module
//! does not fork or extend the cache's identity scheme. It also never
//! imports `ship-space`: the platform-bundle fetch flow does no
//! registration, library-row update, or space membership change. Those
//! happen in a later slice when a stack decides to consume a bundle.
//!
//! # Flow
//!
//! 1. Resolve a [`ChannelManifest`] entry for a target bundle name + daemon
//!    version via [`resolve_platform_bundle`] (pure, deterministic).
//! 2. Fetch the bundle tarball via the [`BundleFetcher`] trait (HTTP in
//!    production; mocked in tests so no network in CI).
//! 3. Verify the entry's signature against [`product_key`] (or an explicit
//!    override for tests).
//! 4. Verify the fetched bytes' SHA-256 equals the manifest's `content_hash`.
//! 5. Extract into an isolated temp dir and hand it to
//!    [`crate::cache::PackageCache::store`] under a `platform/<channel>/<name>`
//!    dep path. Returns the [`crate::cache::CachedPackage`] for the local tree.

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::bundle_sig::{TrustedKey, trusted_keys_from_jwks};
use super::cache::{CachedPackage, PackageCache};

// ---------------------------------------------------------------------------
// PRODUCT key — build-time-baked Ed25519 pubkey.
//
// Distinct trust anchor from publisher keys. See the module-level docs for why
// the two anchors must stay separate.
//
// The const below is intentionally a STUB for this slice — the real key is
// generated and custodied by the ship product team out of band. Any attempt
// to verify against this stub MUST fail closed: a runtime check below refuses
// to honor any signature whose `kid` matches `PRODUCT_KID` while the public
// key bytes still carry the PLACEHOLDER marker. This mirrors the guard
// already in place for `bundle_sig::FIRST_PARTY_KID`.
// ---------------------------------------------------------------------------

/// Baked-in product key id (kid). Distinct from `FIRST_PARTY_KID` (publisher
/// anchor) — see module docs.
pub const PRODUCT_KID: &str = "ship-product-PLACEHOLDER-do-not-trust";

/// Baked-in product public key (JWK `x`, base64url-no-pad). Stub for this
/// slice; release engineering replaces before any release cut ships a real
/// channel manifest gated on it.
pub const PRODUCT_PUB_X: &str = "PLACEHOLDER-product-pubkey-replace-before-release";

/// The trust anchor [`fetch_platform_bundle`] uses by default. Tests pass an
/// explicit [`TrustedKey`] to avoid the runtime PLACEHOLDER guard.
pub fn product_key() -> TrustedKey {
    TrustedKey {
        kid: PRODUCT_KID.to_string(),
        x_base64url: PRODUCT_PUB_X.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Manifest types
// ---------------------------------------------------------------------------

/// A single platform-bundle entry within a channel.
///
/// The `signature` field is a detached EdDSA JWT over the other fields,
/// signed by the product key. The verifier MUST check both:
///   1. signature (proves the manifest itself was issued by the product team)
///   2. SHA-256 of the fetched bytes (proves the bytes match what was signed)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformBundleEntry {
    pub name: String,
    pub version: String,
    /// Semver requirement the bundle declares it is compatible with.
    /// Example: `"^0.1.0"`, `">=0.2.0, <0.3.0"`.
    #[serde(rename = "daemon_compat")]
    pub daemon_compat: String,
    /// HTTPS URL the tarball is fetched from.
    pub url: String,
    /// `"sha256:<lowercase-hex>"` of the tarball bytes.
    #[serde(rename = "content_hash")]
    pub content_hash: String,
    /// Detached EdDSA JWT issued by the product key.
    pub signature: String,
}

/// Top-level channel manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelManifest {
    pub channel: String,
    pub bundles: Vec<PlatformBundleEntry>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by [`fetch_platform_bundle`].
#[derive(Debug)]
pub enum FetchError {
    /// HTTP fetch failed (network, non-2xx, body too large, etc.).
    Fetch(String),
    /// Manifest entry's signature could not be parsed or failed verification.
    Signature(BadSignature),
    /// Fetched bytes' SHA-256 did not match the manifest's `content_hash`.
    ContentHash { expected: String, actual: String },
    /// Could not extract the tarball into a temp dir.
    Extract(String),
    /// Underlying cache error.
    Cache(String),
    /// Spec-violating entry (bad version string, malformed hash, etc.).
    Invalid(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BadSignature {
    /// JWT could not be parsed or the kid did not match the trusted key.
    Untrusted,
    /// The signature itself failed to verify.
    Invalid,
    /// The JWT was well-formed but the claims did not match the manifest.
    ClaimMismatch,
}

impl std::fmt::Display for BadSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BadSignature::Untrusted => write!(f, "platform bundle signature from untrusted key"),
            BadSignature::Invalid => write!(f, "platform bundle signature did not verify"),
            BadSignature::ClaimMismatch => write!(
                f,
                "platform bundle signature claims did not match manifest entry"
            ),
        }
    }
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Fetch(msg) => write!(f, "fetch failed: {msg}"),
            FetchError::Signature(b) => write!(f, "{b}"),
            FetchError::ContentHash { expected, actual } => write!(
                f,
                "platform bundle content hash mismatch: expected {expected}, got {actual}"
            ),
            FetchError::Extract(msg) => write!(f, "extraction failed: {msg}"),
            FetchError::Cache(msg) => write!(f, "cache error: {msg}"),
            FetchError::Invalid(msg) => write!(f, "invalid entry: {msg}"),
        }
    }
}

impl std::error::Error for FetchError {}

// ---------------------------------------------------------------------------
// Fetcher abstraction
// ---------------------------------------------------------------------------

/// HTTP fetch contract for a platform-bundle URL.
///
/// Production wires [`ReqwestFetcher`]; tests pass a closure-backed mock so no
/// network is involved.
pub trait BundleFetcher {
    /// Fetch `url` and return the raw response body. Errors are non-retryable
    /// from the perspective of the caller — the resolver decides what to do.
    fn fetch(&self, url: &str) -> anyhow::Result<Vec<u8>>;
}

/// `reqwest::blocking`-backed [`BundleFetcher`].
pub struct ReqwestFetcher {
    client: reqwest::blocking::Client,
    user_agent: String,
}

impl ReqwestFetcher {
    /// Default fetcher — 30 s total timeout, mirrors the existing
    /// `ship-store` fetch style (`User-Agent: ship-pkg/0.1`).
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client builds with default config"),
            user_agent: "ship-pkg/0.1".to_string(),
        }
    }

    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }
}

impl Default for ReqwestFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl BundleFetcher for ReqwestFetcher {
    fn fetch(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        let mut resp = self
            .client
            .get(url)
            .header("User-Agent", &self.user_agent)
            .send()
            .with_context(|| format!("requesting {url}"))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("HTTP {status} fetching {url}"));
        }
        // Early reject when the server advertises an oversized body.
        if let Some(declared) = resp.content_length()
            && declared > bundle_limits::MAX_BUNDLE_DOWNLOAD
        {
            return Err(anyhow!(
                "platform bundle download from {url} declared {declared} bytes, \
                 exceeds MAX_BUNDLE_DOWNLOAD ({} bytes)",
                bundle_limits::MAX_BUNDLE_DOWNLOAD
            ));
        }
        // Stream the body so a hostile or oversized response cannot OOM
        // the daemon. `resp` impls `Read`; the streaming helper caps the
        // allocation at the limit. content-length is a hint, not a
        // guarantee, so the cap is the binding check.
        let mut out = Vec::new();
        bundle_limits::copy_capped(&mut resp, &mut out, bundle_limits::MAX_BUNDLE_DOWNLOAD)
            .with_context(|| format!("reading body of {url}"))?;
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Verification helpers
// ---------------------------------------------------------------------------

/// JWT claim type for a single platform-bundle entry. Mirrors the shape of
/// [`crate::bundle_sig::BundleClaim`] but for a different trust anchor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EntryClaim {
    name: String,
    version: String,
    url: String,
    content_hash: String,
    #[serde(default)]
    kid: Option<String>,
    #[serde(default)]
    iat: Option<i64>,
}

/// Verify the manifest entry's signature against `trusted_key`, and confirm
/// the JWT's claims match the rest of the entry byte-for-byte.
///
/// This is split out so tests can exercise the verify path independently of
/// the byte-fetch path.
pub fn verify_entry_signature(
    entry: &PlatformBundleEntry,
    trusted_key: &TrustedKey,
) -> Result<(), BadSignature> {
    // Runtime fail-closed guard: a STUB PRODUCT key (the slice-1 default)
    // MUST NOT verify anything, even if it appears in the trusted set.
    // Mirrors the FIRST_PARTY_KID guard in bundle_sig.
    if PRODUCT_KID.contains("PLACEHOLDER") && trusted_key.kid == PRODUCT_KID {
        return Err(BadSignature::Untrusted);
    }

    let decoding_key = DecodingKey::from_ed_components(&trusted_key.x_base64url)
        .map_err(|_| BadSignature::Untrusted)?;

    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.validate_exp = false;
    validation.required_spec_claims.clear();

    let token = decode::<EntryClaim>(&entry.signature, &decoding_key, &validation)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSignature => BadSignature::Invalid,
            _ => BadSignature::Untrusted,
        })?;

    let claim = token.claims;
    if claim.name != entry.name
        || claim.version != entry.version
        || claim.url != entry.url
        || claim.content_hash != entry.content_hash
    {
        return Err(BadSignature::ClaimMismatch);
    }

    Ok(())
}

/// SHA-256 of `bytes`, formatted as `"sha256:<lowercase-hex>"`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity("sha256:".len() + digest.len() * 2);
    out.push_str("sha256:");
    for b in digest {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}

fn check_content_hash(expected: &str, actual_bytes: &[u8]) -> Result<(), FetchError> {
    let actual = sha256_hex(actual_bytes);
    if actual == expected {
        Ok(())
    } else {
        Err(FetchError::ContentHash {
            expected: expected.to_string(),
            actual,
        })
    }
}

fn extract_tarball(bytes: &[u8], dest: &Path) -> Result<(), FetchError> {
    bundle_limits::extract_tar_capped(bytes, dest, bundle_limits::MAX_BUNDLE_UNPACKED)
        .map_err(|e| FetchError::Extract(e.to_string()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// resolve_platform_bundle — pure, deterministic
// ---------------------------------------------------------------------------

/// Choose the newest entry whose `name` matches and whose `daemon_compat`
/// satisfies `daemon_version`.
///
/// "Newest" is by semver on `entry.version`. Ties are broken by URL lex order
/// so the result is fully deterministic across runs.
///
/// Returns `None` if no entry in the manifest matches. The signature is NOT
/// checked here — resolution is a pure function over the manifest, and the
/// fetch path verifies signatures on the chosen entry.
pub fn resolve_platform_bundle(
    manifest: &ChannelManifest,
    name: &str,
    daemon_version: &str,
) -> Option<PlatformBundleEntry> {
    let daemon = Version::parse(daemon_version).ok()?;

    let mut best: Option<(Version, PlatformBundleEntry)> = None;
    for entry in &manifest.bundles {
        if entry.name != name {
            continue;
        }
        // Skip entries whose own daemon_compat we can't parse, or whose
        // daemon_compat doesn't satisfy this daemon. Also skip entries whose
        // version isn't valid semver — the "newest" comparison is only
        // well-defined on valid versions.
        let Ok(entry_req) = semver::VersionReq::parse(&entry.daemon_compat) else {
            continue;
        };
        if !entry_req.matches(&daemon) {
            continue;
        }
        let Ok(entry_ver) = Version::parse(&entry.version) else {
            continue;
        };
        match &best {
            None => best = Some((entry_ver, entry.clone())),
            Some((cur_ver, cur_entry)) => {
                if entry_ver > *cur_ver
                    || (entry_ver == *cur_ver && entry.url < cur_entry.url)
                {
                    best = Some((entry_ver, entry.clone()));
                }
            }
        }
    }

    best.map(|(_, e)| e)
}

// ---------------------------------------------------------------------------
// fetch_platform_bundle
// ---------------------------------------------------------------------------

/// Fetch a platform bundle described by `entry`, verify it, and store the
/// extracted tree in `cache`. Returns the local [`CachedPackage`].
///
/// Verification order is fixed: signature FIRST, then content hash. If the
/// signature is invalid the bytes are discarded immediately — the daemon
/// never hashes content from a manifest entry it cannot trust.
///
/// The cache is keyed by `"platform/<channel>/<name>"` (dep path) and the
/// bundle's `version`. The cache's `commit` field carries the manifest's
/// `content_hash` for traceability.
pub fn fetch_platform_bundle(
    entry: &PlatformBundleEntry,
    channel: &str,
    cache: &PackageCache,
    fetcher: &dyn BundleFetcher,
    trusted_key: &TrustedKey,
) -> Result<CachedPackage, FetchError> {
    // 1. Signature — verify against the product key BEFORE touching bytes.
    verify_entry_signature(entry, trusted_key).map_err(FetchError::Signature)?;

    // 2. Bytes — fetch from the URL the signature just vouched for.
    let bytes = fetcher
        .fetch(&entry.url)
        .map_err(|e| FetchError::Fetch(e.to_string()))?;

    // 3. Content hash — bytes must match what the signature bound.
    check_content_hash(&entry.content_hash, &bytes)?;

    // 4. Extract into an isolated temp dir; on failure the temp dir is
    //    dropped by the `TempDir` destructor, so unverified content never
    //    reaches the cache.
    let tmp = tempfile::tempdir().map_err(|e| FetchError::Extract(e.to_string()))?;
    extract_tarball(&bytes, tmp.path())?;
    // On any error path from here on, the temp dir still owns its files and
    // they get cleaned up by `Drop`.

    // 5. Hand the extracted tree to the existing content-addressed cache.
    let dep_path = platform_dep_path(channel, &entry.name);
    let cached = cache
        .store(&dep_path, &entry.version, &entry.content_hash, tmp.path())
        .map_err(|e| FetchError::Cache(e.to_string()))?;

    Ok(cached)
}

/// The dep path the cache uses for a platform bundle. Public so callers can
/// look up a previously-cached bundle without re-fetching.
pub fn platform_dep_path(channel: &str, name: &str) -> String {
    format!("platform/{channel}/{name}")
}

// ---------------------------------------------------------------------------
// Convenience: build the union of trust anchors.
//
// `trusted_keys_for_manifest` is the default for production: just the baked-in
/// PRODUCT key. Tests call `verify_entry_signature` directly with an
/// injected key. This helper exists so the next slice (which fetches JWKS)
/// has a single place to swap the union construction.
// ---------------------------------------------------------------------------

/// Default trust anchor set: the baked-in [`PRODUCT_PUBKEY`] only.
///
/// When the JWKS-backed product-key rotation lands (out of scope here),
/// merge `trusted_keys_from_jwks(&body)` with [`product_key()`].
#[allow(dead_code)]
pub fn default_product_trust_anchors() -> Vec<TrustedKey> {
    vec![product_key()]
}

/// Build the trust-anchor set from a JWKS body. Convenience wrapper that
/// keeps the module's `pub` surface symmetric and hides the upstream helper.
#[allow(dead_code)]
pub fn product_trust_anchors_from_jwks(jwks: &str) -> Vec<TrustedKey> {
    trusted_keys_from_jwks(jwks)
}

// ---------------------------------------------------------------------------
// Internal helper for callers that want the local path without the metadata.
// ---------------------------------------------------------------------------

/// Like [`CachedPackage::dir`] but with the `PathBuf` clone already done.
#[allow(dead_code)]
pub fn cached_bundle_dir(cached: &CachedPackage) -> PathBuf {
    cached.dir.clone()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "platform_bundles_tests.rs"]
mod tests;
