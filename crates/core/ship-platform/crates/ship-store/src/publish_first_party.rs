//! First-party publish path — pack a stack tree, mint a local EdDSA attestation
//! over its canonical tree hash, and hand back everything a distributor needs to
//! write the bundle + attestation to a release channel.
//!
//! Distinct from the gateway server-side mint: this is the LOCAL signing flow
//! release-engineering uses to stamp a `ship-meta` or starter stack with a
//! first-party key. The output is a single [`SignedBundle`] the consumer can
//! hand to [`crate::install_bundle::extract_and_verify_bundle`] on the install
//! side (after writing the bytes to disk and the JWT next to it). The
//! verification trust model is unchanged: the same [`crate::bundle_sig`] verify
//! path, with the first-party kid resolved through the [`TrustedKey`] set
//! assembled by the daemon (baked-in [`crate::bundle_sig::FIRST_PARTY_KID`]
//! union the gateway JWKS publisher keys).
//!
//! The key material is the caller's PKCS8 Ed25519 PEM. Today the only
//! first-party key is a test/dev placeholder; production provisioning is a
//! release-engineering follow-up (see `FIRST_PARTY_KID` docs in
//! `bundle_sig.rs`). The function is intentionally narrow — pack + sign only —
//! so callers (CLI subcommand, CI job, release script) control distribution
//! and the kid they pass here.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use stack_model::ArchiveHeader;

use crate::archive::pack_stack;
use crate::bundle_sig::{BundleClaim, sign_bundle_attestation};

/// A stack archive plus the first-party attestation JWT binding its content
/// hash. Distribute `bytes` and `attestation_jwt` together; the install side
/// recomputes the tree hash from the extracted content and verifies the JWT
/// against it.
#[derive(Debug, Clone)]
pub struct SignedBundle {
    /// Gzip-compressed tarball produced by [`pack_stack`].
    pub bytes: Vec<u8>,
    /// Archive header packed into the tarball. `header.hash` is the canonical
    /// `sha256:<hex>` tree hash the attestation binds.
    pub header: ArchiveHeader,
    /// EdDSA JWT over the [`BundleClaim`] (path/version/sha256/kid/iat). The
    /// `kid` in the JWT header is the same `kid` passed in here.
    pub attestation_jwt: String,
}

/// Pack a stack tree and mint a first-party attestation over its content hash.
///
/// `root` is the stack source directory (must contain a publishable
/// `ship.json` — see [`crate::publish_walk::validate_publishable_stack`]). The
/// `signing_key_pem` is the PKCS8 Ed25519 private key for the first-party
/// publisher; `kid` is the JWK `kid` callers want the verifier to select
/// against on the install side. The attestation's claim is built from
/// `header.path`, `header.version`, and `header.hash` (the canonical tree
/// hash [`pack_stack`] computed) so the install-side verify step can recompute
/// the same hash from the extracted content and require an exact match.
pub fn publish_first_party(
    root: &Path,
    signing_key_pem: &str,
    kid: &str,
) -> anyhow::Result<SignedBundle> {
    let (bytes, header) =
        pack_stack(root).with_context(|| format!("packing stack at {}", root.display()))?;

    let iat = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let claim = BundleClaim {
        path: header.path.clone(),
        version: header.version.clone(),
        sha256: header.hash.clone(),
        kid: Some(kid.to_string()),
        iat: Some(iat),
    };

    let attestation_jwt = sign_bundle_attestation(&claim, signing_key_pem, kid);

    Ok(SignedBundle {
        bytes,
        header,
        attestation_jwt,
    })
}
