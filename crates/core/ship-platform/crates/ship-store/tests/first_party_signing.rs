//! First-party publish/sign/verify round-trip.
//!
//! Exercises the new `publish_first_party` helper end-to-end against the
//! `verify_bundle_attestation` -> `unpack_stack` pipeline:
//!
//! 1. Build a ship-meta-shaped stack tree on disk.
//! 2. `publish_first_party` packs it, mints a JWT over the canonical tree
//!    hash, and returns the bundle + attestation.
//! 3. `extract_and_verify_bundle` re-hashes the extracted content and
//!    verifies the JWT against the trust set.
//! 4. `unpack_stack` re-hashes again and refuses tampered content.
//!
//! Plus two negatives: a JWT signed by an unrelated key is rejected with
//! `UntrustedKey`/`SignatureInvalid`, and tampering with the bytes after
//! publish produces a `HashMismatch` (because the recomputed tree hash no
//! longer matches the signed one).
//!
//! The PEM below is a TEST-ONLY Ed25519 keypair generated offline with
//! `openssl genpkey -algorithm ed25519`; the matching public `x` is derived
//! the same way as the slice-1 test keys (raw 32-byte pubkey, base64url-no-
//! pad). These MUST NOT be used for real first-party publication.

use std::fs;
use std::path::Path;

use ship_store::{
    BundleVerifyError, TrustedKey, extract_and_verify_bundle, pack_stack, publish_first_party,
    unpack_stack, verify_bundle_attestation,
};
use stack_model::{ArchiveHeader, manifest::ManifestExports};

const FIRST_PARTY_KID: &str = "test-first-party-1";
const FIRST_PARTY_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
    MC4CAQAwBQYDK2VwBCIEIAFISiAspm6fXMD6Gg3dlW8qtiRFxoiHbiCjCuys/rg5\n\
    -----END PRIVATE KEY-----\n";
// `openssl pkey -pubout -outform DER | tail -c 32 | basenc --base64url`
// over the same key (the trailing 32 DER bytes are the raw Ed25519 point).
const FIRST_PARTY_PUB_X: &str = "mZAmshQk4uLsAab8Rhqqt9kJGbP9F6hZZ39eyfUncAQ";

// A second, unrelated keypair — for the negative "wrong key" case. The public
// half is NOT in the trusted set, so an attestation that names it must fail
// to verify.
const OTHER_PRIV_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
    MC4CAQAwBQYDK2VwBCIEIOKTdLdQ37vqn0+6MYF8iMq3vrfGWl/d9nZWeYoMfIww\n\
    -----END PRIVATE KEY-----\n";

fn first_party_trusted() -> Vec<TrustedKey> {
    vec![TrustedKey {
        kid: FIRST_PARTY_KID.to_string(),
        x_base64url: FIRST_PARTY_PUB_X.to_string(),
    }]
}

/// Write a ship-meta-shaped stack fixture: a stack marker, a skill, an agent
/// script, and a `ship.json` carrying the publisher-stated version/commit.
/// `pack_stack` overwrites the placeholder `hash` with the canonical tree hash.
fn write_first_party_fixture(root: &Path) {
    fs::write(root.join("stack.config.ts"), "export default {};\n").unwrap();
    fs::create_dir_all(root.join("skills/review")).unwrap();
    fs::write(root.join("skills/review/SKILL.md"), "# Review skill\n").unwrap();
    fs::create_dir_all(root.join("agents")).unwrap();
    fs::write(
        root.join("agents/reviewer.ts"),
        "export default { id: 'reviewer' };\n",
    )
    .unwrap();

    let header = ArchiveHeader {
        path: "github.com/ship/ship-meta".into(),
        version: "0.4.0".into(),
        commit: "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
        hash: String::new(),
        exports: ManifestExports::default(),
        dependencies: Default::default(),
        recommended_agents: vec!["reviewer".into()],
        recommended_skills: vec!["review".into()],
    };
    fs::write(
        root.join("ship.json"),
        serde_json::to_vec_pretty(&header).expect("serialize header"),
    )
    .unwrap();
}

#[test]
fn first_party_round_trip_pack_sign_verify_unpack() {
    let src = tempfile::tempdir().unwrap();
    write_first_party_fixture(src.path());

    // Publish: pack + sign under the first-party key.
    let signed =
        publish_first_party(src.path(), FIRST_PARTY_PRIV_PEM, FIRST_PARTY_KID).expect("publish");

    // The header carries the canonical tree hash the attestation binds.
    assert!(signed.header.hash.starts_with("sha256:"));
    assert_eq!(signed.header.path, "github.com/ship/ship-meta");
    assert_eq!(signed.header.version, "0.4.0");

    // The install-side verify gate: extract into an isolated dir, recompute
    // the tree hash, verify the attestation against the trust set.
    let verify_dir = tempfile::tempdir().unwrap();
    let verified = extract_and_verify_bundle(
        &signed.bytes,
        &signed.attestation_jwt,
        &first_party_trusted(),
        verify_dir.path(),
    )
    .expect("verify accepts first-party signed bundle");

    assert_eq!(verified.path, "github.com/ship/ship-meta");
    assert_eq!(verified.version, "0.4.0");
    assert_eq!(verified.sha256, signed.header.hash);

    // The attestation JWT also verifies against the same tree hash via the
    // direct `verify_bundle_attestation` API (the lower-level primitive the
    // install bundle helper composes).
    let direct = verify_bundle_attestation(
        &signed.header.hash,
        &signed.attestation_jwt,
        &first_party_trusted(),
    )
    .expect("direct verify");
    assert_eq!(direct, verified);

    // The verified content extracts cleanly via `unpack_stack` — same header
    // shape round-trips, and the on-disk files match the source fixture.
    let dest = tempfile::tempdir().unwrap();
    let extracted = unpack_stack(&signed.bytes, dest.path()).expect("unpack verified bundle");
    assert_eq!(extracted, signed.header);
    assert_eq!(
        fs::read_to_string(dest.path().join("agents/reviewer.ts")).unwrap(),
        "export default { id: 'reviewer' };\n"
    );
    assert_eq!(
        fs::read_to_string(dest.path().join("skills/review/SKILL.md")).unwrap(),
        "# Review skill\n"
    );
}

#[test]
fn first_party_signed_bundle_rejects_signature_from_unrelated_key() {
    let src = tempfile::tempdir().unwrap();
    write_first_party_fixture(src.path());

    // Mint the bundle locally, then re-sign the SAME claim under an
    // unrelated key (carrying the trusted kid in the JWT header) so the
    // verifier selects a key from the trust set but the signature fails.
    let (bytes, header) = pack_stack(src.path()).expect("pack");
    let mut claim = ship_store::BundleClaim {
        path: header.path.clone(),
        version: header.version.clone(),
        sha256: header.hash.clone(),
        kid: Some(FIRST_PARTY_KID.to_string()),
        iat: Some(1_700_000_000),
    };
    // First the legit path: trust set accepts it.
    let good_jwt =
        ship_store::sign_bundle_attestation(&claim, FIRST_PARTY_PRIV_PEM, FIRST_PARTY_KID);
    let verify_dir = tempfile::tempdir().unwrap();
    extract_and_verify_bundle(&bytes, &good_jwt, &first_party_trusted(), verify_dir.path())
        .expect("real key verifies");

    // Now sign with the unrelated key — same claim, same kid → key is
    // selected from the trust set, signature fails to verify.
    claim.kid = Some(FIRST_PARTY_KID.to_string());
    let forged_jwt = ship_store::sign_bundle_attestation(&claim, OTHER_PRIV_PEM, FIRST_PARTY_KID);

    let verify_dir = tempfile::tempdir().unwrap();
    let err = extract_and_verify_bundle(
        &bytes,
        &forged_jwt,
        &first_party_trusted(),
        verify_dir.path(),
    )
    .expect_err("wrong-key signature must be rejected");
    assert!(
        matches!(
            err,
            ship_store::InstallBundleError::Verify(BundleVerifyError::SignatureInvalid)
        ),
        "expected SignatureInvalid, got {err:?}"
    );
    // The verify gate wiped the extract dir on failure.
    assert!(
        fs::read_dir(verify_dir.path()).unwrap().next().is_none(),
        "verify failure must not leave extracted files behind"
    );
}

#[test]
fn first_party_signed_bundle_rejects_tampered_content_with_hash_mismatch() {
    let src = tempfile::tempdir().unwrap();
    write_first_party_fixture(src.path());

    let signed =
        publish_first_party(src.path(), FIRST_PARTY_PRIV_PEM, FIRST_PARTY_KID).expect("publish");
    let original_hash = signed.header.hash.clone();

    // Tamper with the BUNDLE CONTENT: extract the tarball, modify a
    // payload file, then re-pack under the original header. The
    // recomputed tree hash will diverge from the one the JWT was signed
    // over, so the verify gate's claim-binding step must reject.
    let extract_dir = tempfile::tempdir().unwrap();
    let gz = flate2::read::GzDecoder::new(signed.bytes.as_slice());
    let mut archive = tar::Archive::new(gz);
    archive
        .unpack(extract_dir.path())
        .expect("extract for tamper");
    fs::write(
        extract_dir.path().join("agents/reviewer.ts"),
        "export default { id: 'attacker' };\n",
    )
    .expect("overwrite payload file");
    let tampered_bytes = {
        // Re-pack from the tampered tree; ship.json still carries the
        // ORIGINAL hash (header.hash), but pack_stack will overwrite it
        // with the recomputed tree hash. To keep the header's hash as
        // it was at publish time, we restore the original header first.
        let header_path = extract_dir.path().join("ship.json");
        let mut original_header: ArchiveHeader =
            serde_json::from_slice(&fs::read(&header_path).unwrap()).unwrap();
        // Wipe the hash so pack_stack doesn't replace it; then re-stamp
        // the original signed hash BEFORE pack writes the new one.
        original_header.hash = original_hash.clone();
        fs::write(
            &header_path,
            serde_json::to_vec_pretty(&original_header).unwrap(),
        )
        .unwrap();
        let (bytes, _) = pack_stack(extract_dir.path()).expect("re-pack tampered tree");
        // pack_stack overwrites header.hash with the new tree hash; the
        // attestation's claim still binds the ORIGINAL hash.
        bytes
    };
    // The lower-level verify primitive, called with the new (mismatched)
    // tree hash the install pipeline would compute from the tampered
    // extract, must reject: the signature is fine, but the claim's
    // sha256 no longer matches the presented (recomputed) hash.
    let re = tempfile::tempdir().unwrap();
    let gz = flate2::read::GzDecoder::new(tampered_bytes.as_slice());
    let mut archive = tar::Archive::new(gz);
    archive.unpack(re.path()).expect("re-extract tampered");
    let tampered_tree_hash = ship_store::compute_tree_hash(re.path()).expect("recompute hash");
    assert_ne!(
        tampered_tree_hash, original_hash,
        "tampered content must produce a different tree hash than the signed one"
    );
    let err = verify_bundle_attestation(
        &tampered_tree_hash,
        &signed.attestation_jwt,
        &first_party_trusted(),
    )
    .expect_err("verify must reject when claim hash != presented hash");
    assert_eq!(err, BundleVerifyError::HashMismatch);

    // And the higher-level extract-and-verify gate also refuses: the
    // attestation's claim was bound to the original tree hash, the
    // tampered extract yields a different one, the gate trips on the
    // hash-mismatch step before the bundle reaches the cache.
    let verify_dir = tempfile::tempdir().unwrap();
    let err = extract_and_verify_bundle(
        &tampered_bytes,
        &signed.attestation_jwt,
        &first_party_trusted(),
        verify_dir.path(),
    )
    .expect_err("extract+verify on tampered bytes must fail");
    assert!(
        matches!(
            err,
            ship_store::InstallBundleError::Verify(BundleVerifyError::HashMismatch)
                | ship_store::InstallBundleError::Extract(_)
                | ship_store::InstallBundleError::Hash(_)
        ),
        "expected hash-mismatch / extract / hash failure on tampered bytes, got {err:?}"
    );
    assert!(
        fs::read_dir(verify_dir.path()).unwrap().next().is_none(),
        "verify failure on tampered bytes must not leave extracted files behind"
    );
}
