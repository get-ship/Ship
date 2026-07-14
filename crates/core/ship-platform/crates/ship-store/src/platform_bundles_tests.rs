//! Tests for [`super::platform_bundles`].
//!
//! Coverage:
//! - Manifest serde round-trip (golden fixture).
//! - `resolve_platform_bundle`: matches by name + daemon_compat, picks newest
//!   semver, deterministic tie-break.
//! - `verify_entry_signature`: valid signature passes; tampered claim fails;
//!   wrong key fails; PLACEHOLDER stub rejects.
//! - `fetch_platform_bundle`: happy path stores in cache; bad signature
//!   rejects without touching cache; content-hash mismatch rejects without
//!   extracting; cache miss path round-trips.
//!
//! All tests use offline-generated Ed25519 keypairs and a closure-backed
//! `BundleFetcher` mock. No network, no real reqwest calls.

use std::collections::HashMap;
use std::sync::Arc;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde_json::json;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use super::{
    BadSignature, BundleFetcher, ChannelManifest, FetchError, PlatformBundleEntry,
    fetch_platform_bundle, platform_dep_path, resolve_platform_bundle, sha256_hex,
    verify_entry_signature,
};
use crate::bundle_sig::TrustedKey;
use crate::cache::PackageCache;

// ---------------------------------------------------------------------------
// Ed25519 test keypair (offline-generated; product-key analog).
//
// The slice-1 default `PRODUCT_PUBKEY` is a PLACEHOLDER that MUST refuse to
// verify (runtime guard). Tests use a different kid + key pair so we exercise
// the verify path against a real, non-PLACEHOLDER key.
// ---------------------------------------------------------------------------

const PROD_KID: &str = "test-product-key-1";
const PROD_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
    MC4CAQAwBQYDK2VwBCIEIOW6/4Bk92wZFSF/zU+mEu2pg5RUnvWjXZETOARHXDfg\n\
    -----END PRIVATE KEY-----\n";
const PROD_PUB_X: &str = "CvAocfcdR4xFInJmrmMkzg38YEkoRTodNzIHXFXsUDo";
const OTHER_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
    MC4CAQAwBQYDK2VwBCIEIAwS+2XJRdeWScscbyz0waAZmRmUm27ZwGgmdRcU0r9n\n\
    -----END PRIVATE KEY-----\n";

fn product_trusted_key() -> TrustedKey {
    TrustedKey {
        kid: PROD_KID.to_string(),
        x_base64url: PROD_PUB_X.to_string(),
    }
}

// Mock fetcher
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
struct MockFetcher {
    /// URL → response bytes. Returns the bytes verbatim on GET.
    responses: Arc<std::sync::Mutex<HashMap<String, Vec<u8>>>>,
    /// Counts each GET so tests can assert call patterns.
    hits: Arc<std::sync::Mutex<Vec<String>>>,
}

impl MockFetcher {
    fn new() -> Self {
        Self::default()
    }

    fn put(&self, url: &str, body: &[u8]) {
        self.responses
            .lock()
            .unwrap()
            .insert(url.to_string(), body.to_vec());
    }
}

impl BundleFetcher for MockFetcher {
    fn fetch(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        self.hits.lock().unwrap().push(url.to_string());
        self.responses
            .lock()
            .unwrap()
            .get(url)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("mock fetcher: no body for {url}"))
    }
}

// ---------------------------------------------------------------------------
// Tarball helpers — build a gzip-tarball of one file in memory.
// ---------------------------------------------------------------------------

fn make_tarball(path_in_archive: &str, body: &[u8]) -> Vec<u8> {
    let mut tar_buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_buf);
        let mut header = tar::Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, path_in_archive, body)
            .unwrap();
        builder.finish().unwrap();
    }
    let mut gz = Vec::new();
    {
        use std::io::Write;
        let mut enc =
            flate2::write::GzEncoder::new(&mut gz, flate2::Compression::default());
        enc.write_all(&tar_buf).unwrap();
        enc.finish().unwrap();
    }
    gz
}

// ---------------------------------------------------------------------------
// Manifest signing helper.
// ---------------------------------------------------------------------------

fn sign_entry(entry: &PlatformBundleEntry, priv_pem: &str, kid: &str) -> String {
    // The JWT claim shape — keep it identical to EntryClaim's serde view so
    // verify_entry_signature can decode it back. We build via json! to make
    // sure the field names line up exactly with the #[serde(rename)] choices
    // on PlatformBundleEntry (the public-facing struct).
    let claim = json!({
        "name": entry.name,
        "version": entry.version,
        "url": entry.url,
        "content_hash": entry.content_hash,
        "kid": kid,
        "iat": 1_700_000_000_i64,
    });
    let key = EncodingKey::from_ed_pem(priv_pem.as_bytes()).expect("valid ed25519 pem");
    let mut header = Header::new(Algorithm::EdDSA);
    header.kid = Some(kid.to_string());
    jsonwebtoken::encode(&header, &claim, &key).expect("sign jwt")
}

fn signed_entry(
    name: &str,
    version: &str,
    daemon_compat: &str,
    url: &str,
    tarball: &[u8],
) -> PlatformBundleEntry {
    let entry = PlatformBundleEntry {
        name: name.to_string(),
        version: version.to_string(),
        daemon_compat: daemon_compat.to_string(),
        url: url.to_string(),
        content_hash: sha256_hex(tarball),
        signature: String::new(),
    };
    let signature = sign_entry(&entry, PROD_PRIV_PEM, PROD_KID);
    PlatformBundleEntry { signature, ..entry }
}

// ---------------------------------------------------------------------------
// Tests: serde + golden fixture
// ---------------------------------------------------------------------------

#[test]
fn manifest_serde_round_trip_golden_fixture() {
    let tarball = make_tarball("views/x/page.sui", b"<p>hello</p>");
    let entry = signed_entry(
        "ship/themes/midnight",
        "1.2.0",
        ">=0.1.0, <0.3.0",
        "https://cdn.example.com/themes/midnight-1.2.0.tgz",
        &tarball,
    );
    let manifest = ChannelManifest {
        channel: "stable".to_string(),
        bundles: vec![entry],
    };

    let serialized = serde_json::to_string_pretty(&manifest).expect("serialize");
    let golden = serde_json::to_string_pretty(&json!({
        "channel": "stable",
        "bundles": [{
            "name": "ship/themes/midnight",
            "version": "1.2.0",
            "daemon_compat": ">=0.1.0, <0.3.0",
            "url": "https://cdn.example.com/themes/midnight-1.2.0.tgz",
            "content_hash": manifest.bundles[0].content_hash,
            "signature": manifest.bundles[0].signature,
        }]
    }))
    .expect("serialize golden");

    assert_eq!(serialized, golden, "serialized manifest must match golden shape");

    let parsed: ChannelManifest = serde_json::from_str(&serialized).expect("parse");
    assert_eq!(parsed, manifest, "round-trip preserves manifest");
}

#[test]
fn manifest_serde_accepts_alternate_field_names() {
    // Verify the renames work in BOTH directions: `daemon_compat` and
    // `content_hash` round-trip through serde_json.
    let raw = json!({
        "channel": "beta",
        "bundles": [{
            "name": "x",
            "version": "0.1.0",
            "daemon_compat": "^0.1.0",
            "url": "https://x/",
            "content_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "signature": "fake.jwt.value",
        }]
    });
    let m: ChannelManifest = serde_json::from_value(raw.clone()).expect("parse");
    let out = serde_json::to_value(&m).expect("serialize");
    assert_eq!(out["bundles"][0]["daemon_compat"], "^0.1.0");
    assert_eq!(out["bundles"][0]["content_hash"], raw["bundles"][0]["content_hash"]);
}

// ---------------------------------------------------------------------------
// Tests: resolve_platform_bundle
// ---------------------------------------------------------------------------

fn entry_no_sig(name: &str, version: &str, compat: &str, url: &str) -> PlatformBundleEntry {
    PlatformBundleEntry {
        name: name.to_string(),
        version: version.to_string(),
        daemon_compat: compat.to_string(),
        url: url.to_string(),
        content_hash: "sha256:0".repeat(8).replace('0', "a"),
        signature: String::new(),
    }
}

#[test]
fn resolve_picks_newest_matching_entry() {
    let m = ChannelManifest {
        channel: "stable".to_string(),
        bundles: vec![
            entry_no_sig("ship/themes/midnight", "1.0.0", "^0.1.0", "https://a/1.0.0.tgz"),
            entry_no_sig("ship/themes/midnight", "1.2.0", "^0.1.0", "https://a/1.2.0.tgz"),
            entry_no_sig("ship/themes/midnight", "1.1.5", "^0.1.0", "https://a/1.1.5.tgz"),
        ],
    };
    let resolved =
        resolve_platform_bundle(&m, "ship/themes/midnight", "0.1.4").expect("resolved");
    assert_eq!(resolved.version, "1.2.0");
    assert_eq!(resolved.url, "https://a/1.2.0.tgz");
}

#[test]
fn resolve_filters_by_daemon_compat() {
    let m = ChannelManifest {
        channel: "stable".to_string(),
        bundles: vec![
            // Only matches daemon 0.2.x
            entry_no_sig(
                "ship/themes/midnight",
                "2.0.0",
                ">=0.2.0, <0.3.0",
                "https://a/2.0.0.tgz",
            ),
            // Always matches
            entry_no_sig("ship/themes/midnight", "1.0.0", "^0.1.0", "https://a/1.0.0.tgz"),
        ],
    };
    // Daemon 0.2.5 → only the 2.0.0 entry qualifies; it's the newest, so it wins.
    let r = resolve_platform_bundle(&m, "ship/themes/midnight", "0.2.5").expect("resolved");
    assert_eq!(r.version, "2.0.0");

    // Daemon 0.1.4 → only the 1.0.0 entry qualifies.
    let r = resolve_platform_bundle(&m, "ship/themes/midnight", "0.1.4").expect("resolved");
    assert_eq!(r.version, "1.0.0");

    // Daemon 0.3.0 → nothing matches.
    assert!(resolve_platform_bundle(&m, "ship/themes/midnight", "0.3.0").is_none());
}

#[test]
fn resolve_skips_unparseable_compat_or_version() {
    let m = ChannelManifest {
        channel: "stable".to_string(),
        bundles: vec![
            // Garbage daemon_compat — must not be a candidate even though the
            // version parses.
            entry_no_sig("ship/x", "9.9.9", "not-a-valid-req", "https://a/"),
            // Garbage version — same: must not be a candidate.
            entry_no_sig("ship/x", "not-semver", "^0.1.0", "https://b/"),
            // A valid candidate.
            entry_no_sig("ship/x", "1.0.0", "^0.1.0", "https://c/"),
        ],
    };
    let r = resolve_platform_bundle(&m, "ship/x", "0.1.4").expect("resolved");
    assert_eq!(r.url, "https://c/");
}

#[test]
fn resolve_tie_break_is_deterministic_by_url() {
    let m = ChannelManifest {
        channel: "stable".to_string(),
        bundles: vec![
            entry_no_sig("ship/x", "1.0.0", "^0.1.0", "https://zzz/"),
            entry_no_sig("ship/x", "1.0.0", "^0.1.0", "https://aaa/"),
            entry_no_sig("ship/x", "1.0.0", "^0.1.0", "https://mmm/"),
        ],
    };
    let r = resolve_platform_bundle(&m, "ship/x", "0.1.0").expect("resolved");
    assert_eq!(r.url, "https://aaa/", "ties broken by URL lex order");
}

#[test]
fn resolve_returns_none_when_name_unknown() {
    let m = ChannelManifest {
        channel: "stable".to_string(),
        bundles: vec![entry_no_sig("ship/x", "1.0.0", "^0.1.0", "https://a/")],
    };
    assert!(resolve_platform_bundle(&m, "ship/y", "0.1.0").is_none());
}

#[test]
fn resolve_returns_none_for_unparseable_daemon_version() {
    let m = ChannelManifest {
        channel: "stable".to_string(),
        bundles: vec![entry_no_sig("ship/x", "1.0.0", "^0.1.0", "https://a/")],
    };
    assert!(resolve_platform_bundle(&m, "ship/x", "not-a-version").is_none());
}

// ---------------------------------------------------------------------------
// Tests: verify_entry_signature
// ---------------------------------------------------------------------------

#[test]
fn verify_accepts_signature_by_trusted_product_key() {
    let tarball = make_tarball("views/x.sui", b"hi");
    let entry = signed_entry("ship/x", "1.0.0", "^0.1.0", "https://cdn/x.tgz", &tarball);
    assert!(verify_entry_signature(&entry, &product_trusted_key()).is_ok());
}

#[test]
fn verify_rejects_signature_by_untrusted_key() {
    let tarball = make_tarball("v.sui", b"hi");
    let mut entry = signed_entry("ship/x", "1.0.0", "^0.1.0", "https://cdn/x.tgz", &tarball);
    // Re-sign with the OTHER (untrusted) private key.
    entry.signature = sign_entry(&entry, OTHER_PRIV_PEM, "wrong-kid");
    let err = verify_entry_signature(&entry, &product_trusted_key()).unwrap_err();
    // The wrong-key signer produces a signature that fails Ed25519
    // verification against the trusted product key. Either `Invalid`
    // (signature math failed) or `Untrusted` (header kid didn't match) is
    // a valid rejection — the verifier MUST NOT accept the entry.
    assert!(
        err == BadSignature::Invalid || err == BadSignature::Untrusted,
        "wrong-key signature must be rejected, got {err:?}"
    );
}

#[test]
fn verify_rejects_tampered_claims() {
    let tarball = make_tarball("v.sui", b"hi");
    let mut entry = signed_entry("ship/x", "1.0.0", "^0.1.0", "https://cdn/x.tgz", &tarball);
    // Tamper with the URL after signing.
    entry.url = "https://attacker.example/evil.tgz".to_string();
    let err = verify_entry_signature(&entry, &product_trusted_key()).unwrap_err();
    // Either signature or claim-mismatch is acceptable here — the important
    // thing is the verifier refuses to trust the entry.
    assert!(
        err == BadSignature::Invalid || err == BadSignature::ClaimMismatch,
        "tampered entry must be rejected, got {err:?}"
    );
}

#[test]
fn verify_rejects_placeholder_product_key() {
    // The slice-1 default PRODUCT_PUBKEY is a stub. If someone wires it in,
    // the runtime guard MUST refuse.
    let stub = TrustedKey {
        kid: super::PRODUCT_KID.to_string(),
        x_base64url: super::PRODUCT_PUB_X.to_string(),
    };
    let tarball = make_tarball("v.sui", b"hi");
    let entry = signed_entry("ship/x", "1.0.0", "^0.1.0", "https://cdn/x.tgz", &tarball);
    let err = verify_entry_signature(&entry, &stub).unwrap_err();
    assert_eq!(err, BadSignature::Untrusted);
}

// ---------------------------------------------------------------------------
// Tests: fetch_platform_bundle — happy path + failure modes
// ---------------------------------------------------------------------------

#[test]
fn fetch_happy_path_stores_extracted_tree_in_cache() {
    let tarball = make_tarball("views/x/page.sui", b"<p>hello from a platform bundle</p>");
    let entry = signed_entry(
        "ship/themes/midnight",
        "1.2.0",
        "^0.1.0",
        "https://cdn.example.com/midnight-1.2.0.tgz",
        &tarball,
    );

    let fetcher = MockFetcher::new();
    fetcher.put(&entry.url, &tarball);

    let cache_root = tempdir().expect("cache tempdir");
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let cached = fetch_platform_bundle(
        &entry,
        "stable",
        &cache,
        &fetcher,
        &product_trusted_key(),
    )
    .expect("fetch ok");

    // Cache key is platform/<channel>/<name>.
    assert_eq!(
        cached.path,
        platform_dep_path("stable", "ship/themes/midnight"),
    );
    assert_eq!(cached.version, "1.2.0");
    assert_eq!(cached.commit, entry.content_hash);
    assert!(cached.dir.is_dir(), "extracted tree lives under cache objects/");
    assert!(
        cached.dir.join("views/x/page.sui").is_file(),
        "tree contents preserved"
    );

    // Subsequent cache hit returns the same entry.
    let again = cache
        .get(
            &platform_dep_path("stable", "ship/themes/midnight"),
            "1.2.0",
        )
        .expect("second cache hit");
    assert_eq!(again.hash, cached.hash);

    // The fetcher was hit exactly once.
    assert_eq!(fetcher.hits.lock().unwrap().len(), 1);
}

#[test]
fn fetch_rejects_bad_signature_before_fetching() {
    let tarball = make_tarball("v.sui", b"hi");
    let mut entry = signed_entry("ship/x", "1.0.0", "^0.1.0", "https://cdn/x.tgz", &tarball);
    entry.signature = sign_entry(&entry, OTHER_PRIV_PEM, "wrong-kid");

    let fetcher = MockFetcher::new();
    let cache_root = tempdir().expect("cache tempdir");
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let err = fetch_platform_bundle(
        &entry,
        "stable",
        &cache,
        &fetcher,
        &product_trusted_key(),
    )
    .unwrap_err();
    // A wrong-key signer produces a signature that fails Ed25519 verification
    // against the trusted product key. Either `Invalid` (math failed) or
    // `Untrusted` (kid didn't match) is a correct rejection — the invariant
    // is that the fetch path aborts BEFORE touching the network.
    assert!(
        matches!(
            &err,
            FetchError::Signature(BadSignature::Untrusted)
                | FetchError::Signature(BadSignature::Invalid)
        ),
        "bad-signature fetch must abort before HTTP, got {err:?}"
    );
    assert_eq!(fetcher.hits.lock().unwrap().len(), 0, "no fetch on bad sig");
    assert!(
        cache.get(&platform_dep_path("stable", "ship/x"), "1.0.0").is_none(),
        "cache untouched on bad sig"
    );
}

#[test]
fn fetch_rejects_content_hash_mismatch() {
    let tarball = make_tarball("v.sui", b"the genuine bundle");
    let entry = signed_entry("ship/x", "1.0.0", "^0.1.0", "https://cdn/x.tgz", &tarball);

    // Fetcher returns tampered bytes (signature is valid over the ORIGINAL
    // tarball, so the bytes the fetcher serves must match the signed hash).
    let tampered = make_tarball("v.sui", b"attacker-substituted bytes");
    let fetcher = MockFetcher::new();
    fetcher.put(&entry.url, &tampered);

    let cache_root = tempdir().expect("cache tempdir");
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let err = fetch_platform_bundle(
        &entry,
        "stable",
        &cache,
        &fetcher,
        &product_trusted_key(),
    )
    .unwrap_err();
    match err {
        FetchError::ContentHash { expected, actual } => {
            assert_eq!(expected, entry.content_hash);
            assert_eq!(actual, sha256_hex(&tampered));
        }
        other => panic!("expected ContentHash error, got {other:?}"),
    }
    assert!(
        cache.get(&platform_dep_path("stable", "ship/x"), "1.0.0").is_none(),
        "cache untouched on content hash mismatch"
    );
}

#[test]
fn fetch_propagates_fetcher_error() {
    let tarball = make_tarball("v.sui", b"hi");
    let entry = signed_entry("ship/x", "1.0.0", "^0.1.0", "https://cdn/x.tgz", &tarball);

    // Fetcher has no body for this URL.
    let fetcher = MockFetcher::new();
    let cache_root = tempdir().expect("cache tempdir");
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    let err = fetch_platform_bundle(
        &entry,
        "stable",
        &cache,
        &fetcher,
        &product_trusted_key(),
    )
    .unwrap_err();
    assert!(matches!(err, FetchError::Fetch(_)));
}

// ---------------------------------------------------------------------------
// Tests: cache integration — different versions store independently
// ---------------------------------------------------------------------------

#[test]
fn cache_keeps_distinct_bundles_per_version() {
    let cache_root = tempdir().expect("cache tempdir");
    let cache = PackageCache::with_root(cache_root.path().to_path_buf());

    // Each tarball carries the manifest + a domain-dir file with
    // version-specific content. The new allow-list hash reads both, so
    // the two versions hash to distinct values (the same property the
    // old deny-list hash provided by accident).
    let v1 = make_tarball_with_manifest("views/x/page.sui", b"v1 contents");
    let v2 = make_tarball_with_manifest(
        "views/x/page.sui",
        b"v2 contents -- different body",
    );
    let entry_v1 = signed_entry("ship/x", "1.0.0", "^0.1.0", "https://cdn/x-1.0.0.tgz", &v1);
    let entry_v2 = signed_entry("ship/x", "2.0.0", "^0.1.0", "https://cdn/x-2.0.0.tgz", &v2);

    let fetcher = MockFetcher::new();
    fetcher.put(&entry_v1.url, &v1);
    fetcher.put(&entry_v2.url, &v2);

    let c1 = fetch_platform_bundle(
        &entry_v1,
        "stable",
        &cache,
        &fetcher,
        &product_trusted_key(),
    )
    .expect("v1 fetched");
    let c2 = fetch_platform_bundle(
        &entry_v2,
        "stable",
        &cache,
        &fetcher,
        &product_trusted_key(),
    )
    .expect("v2 fetched");

    assert_ne!(c1.hash, c2.hash);
    let v1_contents = std::fs::read(c1.dir.join("views/x/page.sui"))
        .expect("v1 body readable");
    let v2_contents = std::fs::read(c2.dir.join("views/x/page.sui"))
        .expect("v2 body readable");
    assert_eq!(v1_contents, b"v1 contents");
    assert_eq!(v2_contents, b"v2 contents -- different body");
}

/// Build a tarball that includes the stack manifest plus one file at
/// `path_in_archive` with `body` content — the shape a real pack
/// produces, so the new `compute_tree_hash` reads it the same way
/// (`stack.config.ts` + entries under domain dirs).
fn make_tarball_with_manifest(path_in_archive: &str, body: &[u8]) -> Vec<u8> {
    let manifest_bytes = b"export default {};\n";
    let mut tar_buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_buf);
        let mut manifest_header = tar::Header::new_gnu();
        manifest_header.set_size(manifest_bytes.len() as u64);
        manifest_header.set_mode(0o644);
        manifest_header.set_cksum();
        builder
            .append_data(&mut manifest_header, "stack.config.ts", &manifest_bytes[..])
            .unwrap();
        let mut header = tar::Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, path_in_archive, body)
            .unwrap();
        builder.finish().unwrap();
    }
    let mut gz = Vec::new();
    {
        use std::io::Write;
        let mut enc =
            flate2::write::GzEncoder::new(&mut gz, flate2::Compression::default());
        enc.write_all(&tar_buf).unwrap();
        enc.finish().unwrap();
    }
    gz
}

// ---------------------------------------------------------------------------
// Tests: sha256_hex shape
// ---------------------------------------------------------------------------

#[test]
fn sha256_hex_format_is_lowercase_hex_with_prefix() {
    let h = sha256_hex(b"abc");
    // Hand-computed: sha256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
    assert_eq!(
        h,
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

// ---------------------------------------------------------------------------
// Tests: PLACEHOLDER guard covers the slice-1 default
// ---------------------------------------------------------------------------

#[test]
fn product_key_default_is_placeholder_and_refused() {
    use super::product_key;
    let k = product_key();
    assert_eq!(k.kid, super::PRODUCT_KID);
    assert!(super::PRODUCT_KID.contains("PLACEHOLDER"));
    assert!(super::PRODUCT_PUB_X.contains("PLACEHOLDER"));

    let tarball = make_tarball("v.sui", b"hi");
    let entry = signed_entry("ship/x", "1.0.0", "^0.1.0", "https://cdn/x.tgz", &tarball);
    let err = verify_entry_signature(&entry, &k).unwrap_err();
    assert_eq!(err, BadSignature::Untrusted);
}

// Keep base64 import in scope for future x conversion (no longer needed for
// this test file but a useful reference: a JWK `x` value can be re-derived
// from a PEM via `openssl pkey -pubout -outform DER | tail -c 32`).
#[allow(dead_code)]
fn _base64_engine() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
}

// Keep the unused sha2::Digest + Sha256 import from triggering a warning if
// the test module's body is ever slimmed down — they document the dep.
#[allow(dead_code)]
fn _digest_smoke() -> String {
    let mut h = Sha256::new();
    h.update(b"x");
    format!("{:x}", h.finalize())
}
