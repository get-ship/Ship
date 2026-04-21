import { importPKCS8, importJWK, SignJWT, type JWK } from 'jose'

const ALG = 'EdDSA'

export interface DaemonAttestationClaims {
  iss: string
  sub: string
  aud: 'ship-daemon'
  iat: number
  exp: number
  plan?: string
  caps?: string[]
}

export async function signDaemonJwt(params: {
  env: Env
  issuer: string
  userId: string
  plan?: string
  caps?: string[]
  ttlSeconds?: number
}): Promise<string> {
  const { env, ...rest } = params
  const privateKeyPem = env.SHIP_JWT_PRIVATE_KEY
  const keyId = env.SHIP_JWT_PRIVATE_KEY_ID || 'default'
  if (!privateKeyPem) throw new Error('SHIP_JWT_PRIVATE_KEY secret is not set')

  const key = await importPKCS8(privateKeyPem, ALG)
  const ttl = rest.ttlSeconds ?? 3600
  const now = Math.floor(Date.now() / 1000)

  return new SignJWT({ plan: rest.plan, caps: rest.caps })
    .setProtectedHeader({ alg: ALG, kid: keyId, typ: 'JWT' })
    .setIssuer(rest.issuer)
    .setSubject(rest.userId)
    .setAudience('ship-daemon')
    .setIssuedAt(now)
    .setExpirationTime(now + ttl)
    .sign(key)
}

export async function buildJwks(env: Env): Promise<{ keys: JWK[] }> {
  const multi = env.SHIP_JWT_PUBLIC_JWKS
  if (multi) {
    try {
      const parsed = JSON.parse(multi) as { keys: JWK[] }
      if (Array.isArray(parsed.keys)) return parsed
    } catch {
      /* fall through */
    }
  }

  const publicKeyPem = env.SHIP_JWT_PUBLIC_KEY
  const keyId = env.SHIP_JWT_PRIVATE_KEY_ID || 'default'
  if (!publicKeyPem) return { keys: [] }

  const jwk = await pemPublicToJwk(publicKeyPem, keyId)
  return { keys: [jwk] }
}

async function pemPublicToJwk(pem: string, kid: string): Promise<JWK> {
  const mod = await import('jose')
  const key = await mod.importSPKI(pem, ALG, { extractable: true })
  const jwk = await mod.exportJWK(key)
  return { ...jwk, alg: ALG, use: 'sig', kid }
}

export { importJWK }
