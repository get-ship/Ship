/**
 * Connect-RPC implementation of `ship.gateway.v1.AuthService`. Each
 * method mirrors the logic of the legacy REST handler it replaces; the
 * difference is the wire shape (Connect uses proto-derived URLs +
 * proto-encoded bodies, REST used hand-rolled JSON).
 *
 * REST routes that stay (JWKS, OAuth callbacks, Better Auth catch-all,
 * /v1/auth/cli redirects) are not RPC-shaped — they're either binary-ish
 * responses, browser-facing redirects, or owned by a third-party library.
 */
import { Code, ConnectError, type HandlerContext } from '@connectrpc/connect'
import { create } from '@bufbuild/protobuf'
import type { ServiceImpl } from '@connectrpc/connect'
import {
  AuthService,
  type TokenExchangeRequest,
  type TokenExchangeResponse,
  TokenExchangeResponseSchema,
  type RefreshResponse,
  RefreshResponseSchema,
  type DaemonAttestResponse,
  DaemonAttestResponseSchema,
  type GetMeResponse,
  GetMeResponseSchema,
  UserSchema,
} from '../proto/gen/ship/gateway/v1/auth_pb.js'
import { envContextKey } from '../lib/connect-context.js'
import { getAuthDb } from '../lib/d1.js'
import { getAuth } from '../lib/auth.js'
import { requireSession } from '../lib/session-auth.js'
import { signDaemonJwt } from '../lib/daemon-jwt.js'
import { checkRateLimit } from '../lib/rate-limit.js'

function requireEnv(ctx: HandlerContext): Env {
  const env = ctx.values.get(envContextKey)
  if (!env) throw new ConnectError('env binding not injected', Code.Internal)
  return env
}

/**
 * Build a Request-shaped wrapper around the Connect headers so existing
 * header-driven helpers (`requireSession`, `checkRateLimit`) can be reused
 * unchanged. The URL/method don't matter — these helpers only read headers.
 */
function headerRequest(ctx: HandlerContext): Request {
  return new Request('https://internal/connect', {
    method: 'POST',
    headers: ctx.requestHeader,
  })
}

async function sha256Base64url(input: string): Promise<string> {
  const data = new TextEncoder().encode(input)
  const hash = await crypto.subtle.digest('SHA-256', data)
  const bytes = new Uint8Array(hash)
  let binary = ''
  for (const b of bytes) binary += String.fromCharCode(b)
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '')
}

async function tokenExchange(
  req: TokenExchangeRequest,
  ctx: HandlerContext,
): Promise<TokenExchangeResponse> {
  const env = requireEnv(ctx)

  const rl = await checkRateLimit(headerRequest(ctx), 'RATE_LIMITER_CLAIM', 60, env)
  if (!rl.allowed) throw new ConnectError('rate limited', Code.ResourceExhausted)

  if (!req.code || !req.verifier) {
    throw new ConnectError('code and verifier required', Code.InvalidArgument)
  }

  const db = getAuthDb(env)
  if (!db) throw new ConnectError('database unavailable', Code.Unavailable)

  const claim = await db
    .prepare('UPDATE cli_auth_codes SET used = 1 WHERE code = ? AND used = 0')
    .bind(req.code)
    .run()
  if (!claim.meta.changes) {
    throw new ConnectError('invalid or already-used auth code', Code.Unauthenticated)
  }

  const row = await db
    .prepare('SELECT user_id, code_challenge, created_at FROM cli_auth_codes WHERE code = ?')
    .bind(req.code)
    .first<{ user_id: string; code_challenge: string; created_at: number }>()
  if (!row) throw new ConnectError('invalid auth code', Code.Unauthenticated)
  if (Date.now() - row.created_at > 5 * 60 * 1000) {
    throw new ConnectError('auth code expired', Code.Unauthenticated)
  }

  const computed = await sha256Base64url(req.verifier)
  if (computed !== row.code_challenge) {
    throw new ConnectError('PKCE verification failed', Code.Unauthenticated)
  }

  const a = await getAuth(env)
  const result = await a.api.createApiKey({
    body: {
      userId: row.user_id,
      name: `ship-cli ${new Date().toISOString().slice(0, 10)}`,
      prefix: 'ship_',
      // 1-year default TTL. Daemons re-mint via `ship login` when expired.
      expiresIn: 365 * 24 * 60 * 60,
      metadata: { issued_via: 'pkce', issued_at: Date.now() },
    },
  })

  return create(TokenExchangeResponseSchema, {
    key: result.key,
    userId: row.user_id,
    expiresAt: result.expiresAt ? BigInt(Math.floor(new Date(result.expiresAt).getTime() / 1000)) : 0n,
  })
}

async function refresh(_req: unknown, ctx: HandlerContext): Promise<RefreshResponse> {
  const env = requireEnv(ctx)
  const rl = await checkRateLimit(headerRequest(ctx), 'RATE_LIMITER_CLAIM', 60, env)
  if (!rl.allowed) throw new ConnectError('rate limited', Code.ResourceExhausted)

  const session = await requireSession(headerRequest(ctx), env)
  if (session instanceof Response) {
    throw new ConnectError('authentication required', Code.Unauthenticated)
  }

  return create(RefreshResponseSchema, {
    ok: true,
    userId: session.sub,
    note: 'API keys do not refresh in place; run `ship login` to get a fresh key.',
  })
}

async function daemonAttest(_req: unknown, ctx: HandlerContext): Promise<DaemonAttestResponse> {
  const env = requireEnv(ctx)
  const session = await requireSession(headerRequest(ctx), env)
  if (session instanceof Response) {
    throw new ConnectError('authentication required', Code.Unauthenticated)
  }

  const db = getAuthDb(env)
  let plan: string | undefined
  if (db) {
    try {
      const row = await db.prepare('SELECT plan FROM user WHERE id = ? LIMIT 1').bind(session.sub).first<{ plan?: string }>()
      plan = row?.plan ?? undefined
    } catch { /* optional column */ }
  }

  const caps = plan === 'pro' ? ['pro'] : []
  const issuer = env.SHIP_ATTESTATION_ISSUER ?? 'https://getship.dev'
  const ttlSeconds = 3600

  let jwt: string
  try {
    jwt = await signDaemonJwt({ env, issuer, userId: session.sub, plan, caps, ttlSeconds })
  } catch (e) {
    throw new ConnectError(`signing unavailable: ${String(e)}`, Code.Unavailable)
  }

  return create(DaemonAttestResponseSchema, {
    jwt,
    issuer,
    expiresAt: BigInt(Math.floor(Date.now() / 1000) + ttlSeconds),
  })
}

async function getMe(_req: unknown, ctx: HandlerContext): Promise<GetMeResponse> {
  const env = requireEnv(ctx)
  const session = await requireSession(headerRequest(ctx), env)
  if (session instanceof Response) {
    throw new ConnectError('authentication required', Code.Unauthenticated)
  }
  const db = getAuthDb(env)
  if (!db) throw new ConnectError('database unavailable', Code.Unavailable)
  const user = await db
    .prepare('SELECT id, name, email, image FROM user WHERE id = ?')
    .bind(session.sub)
    .first<{ id: string; name: string; email: string; image: string | null }>()
  if (!user) throw new ConnectError('user not found', Code.NotFound)
  return create(GetMeResponseSchema, {
    user: create(UserSchema, {
      id: user.id,
      name: user.name ?? '',
      email: user.email ?? '',
      image: user.image ?? '',
    }),
  })
}

export const authServiceImpl: ServiceImpl<typeof AuthService> = {
  tokenExchange,
  refresh,
  daemonAttest,
  getMe,
}
