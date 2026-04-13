// Daemon attestation JWT — signed by getship.dev, verified by local shipd.
//
// We use EdDSA (Ed25519) via jose. Private keys are stored as Cloudflare
// Worker secrets (base64-encoded PKCS8), exposed to the runtime via
// `SHIP_JWT_PRIVATE_KEY` and `SHIP_JWT_PRIVATE_KEY_ID`.
//
// Public keys are served from /.well-known/jwks.json. Key rotation works by
// adding a new key and flipping SHIP_JWT_PRIVATE_KEY to the new one; old
// public keys stay in the JWKS until all issued JWTs have expired.

import { importPKCS8, importJWK, SignJWT, type JWK } from 'jose'

const ALG = 'EdDSA'

/** Claim set embedded in daemon attestation JWTs. */
export interface DaemonAttestationClaims {
  /** Issuer — getship.dev origin. */
  iss: string
  /** Subject — user ID from Better Auth. */
  sub: string
  /** Audience — always "ship-daemon". */
  aud: 'ship-daemon'
  /** Issued-at (unix seconds). */
  iat: number
  /** Expiry (unix seconds). */
  exp: number
  /** User's plan tier (free, pro, etc.) */
  plan?: string
  /** Capability flags the daemon should honor (e.g. ["workspace:unlimited"]). */
  caps?: string[]
}

function getEnv(key: string): string {
  return (
    ((globalThis as Record<string, unknown>)[key] as string | undefined) ??
    process.env[key] ??
    ''
  )
}

/**
 * Sign a daemon-attestation JWT for the given user.
 *
 * Throws if SHIP_JWT_PRIVATE_KEY is missing or malformed.
 */
export async function signDaemonJwt(params: {
  issuer: string
  userId: string
  plan?: string
  caps?: string[]
  ttlSeconds?: number
}): Promise<string> {
  const privateKeyPem = getEnv('SHIP_JWT_PRIVATE_KEY')
  const keyId = getEnv('SHIP_JWT_PRIVATE_KEY_ID') || 'default'
  if (!privateKeyPem) {
    throw new Error('SHIP_JWT_PRIVATE_KEY secret is not set')
  }

  const key = await importPKCS8(privateKeyPem, ALG)
  const ttl = params.ttlSeconds ?? 3600

  const now = Math.floor(Date.now() / 1000)
  const jwt = new SignJWT({
    plan: params.plan,
    caps: params.caps,
  })
    .setProtectedHeader({ alg: ALG, kid: keyId, typ: 'JWT' })
    .setIssuer(params.issuer)
    .setSubject(params.userId)
    .setAudience('ship-daemon')
    .setIssuedAt(now)
    .setExpirationTime(now + ttl)

  return await jwt.sign(key)
}

/**
 * Build the public JWKS served at /.well-known/jwks.json.
 *
 * Returns ALL active public keys (current and any rotating-out keys still
 * needed to verify outstanding JWTs). Each key is in JWK form.
 *
 * Environment layout:
 *   SHIP_JWT_PUBLIC_JWKS — JSON string, array of JWKs
 * Or the simpler single-key mode:
 *   SHIP_JWT_PUBLIC_KEY   — PEM-encoded SPKI public key
 *   SHIP_JWT_PRIVATE_KEY_ID — key id (reused to stamp the JWK)
 */
export async function buildJwks(): Promise<{ keys: JWK[] }> {
  const multi = getEnv('SHIP_JWT_PUBLIC_JWKS')
  if (multi) {
    try {
      const parsed = JSON.parse(multi) as { keys: JWK[] }
      if (Array.isArray(parsed.keys)) return parsed
    } catch {
      /* fall through to single-key path */
    }
  }

  const publicKeyPem = getEnv('SHIP_JWT_PUBLIC_KEY')
  const keyId = getEnv('SHIP_JWT_PRIVATE_KEY_ID') || 'default'
  if (!publicKeyPem) {
    return { keys: [] }
  }

  const jwk = await pemPublicToJwk(publicKeyPem, keyId)
  return { keys: [jwk] }
}

/**
 * Convert a PEM-encoded Ed25519 public key (SPKI) to a JWK.
 * Uses jose's import-then-export round-trip.
 */
async function pemPublicToJwk(pem: string, kid: string): Promise<JWK> {
  // jose's importSPKI needs the alg. For Ed25519 we use 'EdDSA'.
  const mod = await import('jose')
  const key = await mod.importSPKI(pem, ALG, { extractable: true })
  const jwk = await mod.exportJWK(key)
  return {
    ...jwk,
    alg: ALG,
    use: 'sig',
    kid,
  }
}

// Re-export for test convenience.
export { importJWK }
