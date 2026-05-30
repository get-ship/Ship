/**
 * Bundle attestation signer — the gateway's half of signed-bundle
 * distribution. Mints a detached EdDSA JWT whose claim binds a package
 * coordinate to its content hash; the daemon verifies it (Rust
 * `registry::bundle_sig`) against the tree hash of the bundle it fetched.
 *
 * Reuses the daemon-attest Ed25519 key (`SHIP_JWT_PRIVATE_KEY`, served via
 * `buildJwks` at `/v1/jwks`) — so the same key the daemon already trusts signs
 * bundle attestations, no new key infra to start. Production splits this to a
 * dedicated publisher key under a distinct `kid` (see the registry-distribution
 * design doc, key decision #1).
 *
 * The claim shape mirrors the Rust `BundleClaim`: `{ path, version, sha256,
 * kid, iat }`. `sha256` is the canonical TREE hash (`compute_tree_hash`) — the
 * same value stored as the registry `contentHash` and used by the daemon cache
 * as the object key. One content hash across registry ↔ attestation ↔ cache.
 */
import { importPKCS8, SignJWT } from 'jose'

const ALG = 'EdDSA'

/** Mint a bundle attestation JWT. `sha256` is the `"sha256:<hex>"` tree hash. */
export async function signBundleAttestation(params: {
  env: Env
  path: string
  version: string
  sha256: string
}): Promise<string> {
  const privateKeyPem = params.env.SHIP_JWT_PRIVATE_KEY
  const keyId = params.env.SHIP_JWT_PRIVATE_KEY_ID || 'default'
  if (!privateKeyPem) throw new Error('SHIP_JWT_PRIVATE_KEY secret is not set')

  const key = await importPKCS8(privateKeyPem, ALG)
  const now = Math.floor(Date.now() / 1000)

  // Bundle attestations do not expire — a published version is immutable. The
  // daemon's verifier disables exp validation accordingly.
  return new SignJWT({
    path: params.path,
    version: params.version,
    sha256: params.sha256,
    kid: keyId,
  })
    .setProtectedHeader({ alg: ALG, kid: keyId, typ: 'JWT' })
    .setIssuedAt(now)
    .sign(key)
}
