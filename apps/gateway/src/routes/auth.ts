import { Hono } from 'hono'
import { getAuth } from '../lib/auth.js'
import { requireSession } from '../lib/session-auth.js'
import { checkRateLimit, rateLimitResponse } from '../lib/rate-limit.js'
// CLI/daemon-shaped auth (TokenExchange / Refresh / GetMe) now lives only
// at Connect-RPC paths (/ship.gateway.v1.AuthService/<Method>) — the
// daemon's account module is the sole consumer, and the CLI goes through
// the daemon. REST handlers for those methods were removed in the CLI
// swap commit.
//
// What remains here:
// - /v1/jwks               (browser + third-party tooling expect raw GET)
// - /v1/auth/cli           (browser redirect — not RPC-shaped)
// - /v1/auth/cli-callback  (OAuth callback — also not RPC-shaped)
// - /v1/auth/daemon-attest (browser/Studio flow — REST kept; Connect impl
//                            also exists, browsers may prefer one or the
//                            other depending on their client)
// - /v1/auth/*             (Better Auth catch-all)
import { signDaemonJwt, buildJwks } from '../lib/daemon-jwt.js'
import { getAuthDb, nanoid } from '../lib/d1.js'

const auth = new Hono<{ Bindings: Env }>()

// ── JWKS ─────────────────────────────────────────────────────────────────────

auth.get('/v1/jwks', async (c) => {
  const keys = await buildJwks(c.env)
  return c.json(keys)
})

// /v1/me REST handler removed — the daemon's account module proxies via
// Connect-RPC's GetMe instead. The browser uses Better Auth's typed
// client through the catch-all.

// ── CLI PKCE OAuth ────────────────────────────────────────────────────────────

// PKCE S256 challenge: base64url of a 32-byte SHA-256 → exactly 43 chars.
// RFC 7636 §4.2 allows 43–128 unreserved url-safe chars. Anything outside
// this shape is bogus (or an injection probe) — reject before touching the DB.
const PKCE_CHALLENGE = /^[A-Za-z0-9_-]{43,128}$/

auth.get('/v1/auth/cli', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_AUTH', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const url = new URL(c.req.url)
  const codeChallenge = url.searchParams.get('code_challenge')
  const redirectUri = url.searchParams.get('redirect_uri')

  if (!codeChallenge || !redirectUri) {
    return c.json({ error: 'Missing code_challenge or redirect_uri' }, 400)
  }
  if (!PKCE_CHALLENGE.test(codeChallenge)) {
    return c.json({ error: 'Invalid code_challenge format' }, 400)
  }

  let parsed: URL
  try {
    parsed = new URL(redirectUri)
  } catch {
    return c.json({ error: 'Invalid redirect_uri' }, 400)
  }
  if (parsed.hostname !== 'localhost' && parsed.hostname !== '127.0.0.1') {
    return c.json({ error: 'redirect_uri must be localhost' }, 400)
  }

  const clientId = c.env.GITHUB_CLIENT_ID
  if (!clientId) return c.json({ error: 'GitHub OAuth not configured' }, 500)

  const db = getAuthDb(c.env)
  if (!db) return c.json({ error: 'Database unavailable' }, 503)

  const state = nanoid()
  await db
    .prepare('INSERT INTO cli_auth_state (state, code_challenge, redirect_uri, created_at) VALUES (?, ?, ?, ?)')
    .bind(state, codeChallenge, redirectUri, Date.now())
    .run()

  const callbackUrl = `${url.origin}/v1/auth/cli-callback`
  const ghUrl = new URL('https://github.com/login/oauth/authorize')
  ghUrl.searchParams.set('client_id', clientId)
  ghUrl.searchParams.set('redirect_uri', callbackUrl)
  ghUrl.searchParams.set('scope', 'read:user user:email')
  ghUrl.searchParams.set('state', state)

  return new Response(null, { status: 302, headers: { Location: ghUrl.toString() } })
})

auth.get('/v1/auth/cli-callback', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_AUTH', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const url = new URL(c.req.url)
  const code = url.searchParams.get('code')
  const state = url.searchParams.get('state')
  const error = url.searchParams.get('error')

  const errorRedirect = (msg: string) =>
    new Response(null, { status: 302, headers: { Location: `/studio?cli_error=${encodeURIComponent(msg)}` } })

  if (error) return errorRedirect(error)
  if (!code || !state) return errorRedirect('missing_params')

  const db = getAuthDb(c.env)
  if (!db) return errorRedirect('db_unavailable')

  const pending = await db
    .prepare('SELECT code_challenge, redirect_uri, created_at FROM cli_auth_state WHERE state = ?')
    .bind(state)
    .first<{ code_challenge: string; redirect_uri: string; created_at: number }>()

  if (!pending) return errorRedirect('invalid_state')

  if (Date.now() - pending.created_at > 10 * 60 * 1000) {
    await db.prepare('DELETE FROM cli_auth_state WHERE state = ?').bind(state).run()
    return errorRedirect('state_expired')
  }

  await db.prepare('DELETE FROM cli_auth_state WHERE state = ?').bind(state).run()

  const clientId = c.env.GITHUB_CLIENT_ID
  const clientSecret = c.env.GITHUB_CLIENT_SECRET
  if (!clientId || !clientSecret) return errorRedirect('not_configured')

  const tokenRes = await fetch('https://github.com/login/oauth/access_token', {
    method: 'POST',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
    body: JSON.stringify({ client_id: clientId, client_secret: clientSecret, code }),
  })
  if (!tokenRes.ok) return errorRedirect('github_exchange_failed')
  const tokenData = (await tokenRes.json()) as Record<string, unknown>
  const accessToken = tokenData['access_token']
  if (typeof accessToken !== 'string') return errorRedirect('github_exchange_failed')

  const userRes = await fetch('https://api.github.com/user', {
    headers: { Authorization: `Bearer ${accessToken}`, Accept: 'application/vnd.github.v3+json', 'User-Agent': 'ship-studio/1.0' },
  })
  if (!userRes.ok) return errorRedirect('github_exchange_failed')
  const ghUser = (await userRes.json()) as { id: number; login: string; name: string | null; email: string | null; avatar_url: string }

  const now = Date.now()
  const githubEmail = ghUser.email ?? `${ghUser.login}@users.noreply.github.com`
  const userName = ghUser.name ?? ghUser.login

  const newUserId = nanoid()
  await db
    .prepare(`INSERT INTO user (id, name, email, emailVerified, image, createdAt, updatedAt)
              VALUES (?, ?, ?, ?, ?, ?, ?)
              ON CONFLICT(email) DO UPDATE SET name = excluded.name, image = excluded.image, updatedAt = excluded.updatedAt`)
    .bind(newUserId, userName, githubEmail, 1, ghUser.avatar_url, now, now)
    .run()

  const userRow = await db.prepare('SELECT id FROM user WHERE email = ?').bind(githubEmail).first<{ id: string }>()
  const userId = userRow!.id

  const authCode = nanoid()
  await db
    .prepare('INSERT INTO cli_auth_codes (code, user_id, code_challenge, created_at, used) VALUES (?, ?, ?, ?, 0)')
    .bind(authCode, userId, pending.code_challenge, now)
    .run()

  const cliRedirect = new URL(pending.redirect_uri)
  cliRedirect.searchParams.set('code', authCode)
  return new Response(null, { status: 302, headers: { Location: cliRedirect.toString() } })
})

// /v1/auth/token and /v1/auth/refresh REST handlers removed — Connect-RPC
// owns them at /ship.gateway.v1.AuthService/{TokenExchange,Refresh}.
// Daemon's account module is the only consumer.

auth.post('/v1/auth/daemon-attest', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_AUTH', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const session = await requireSession(c.req.raw, c.env)
  if (session instanceof Response) return session

  const db = getAuthDb(c.env)
  let plan: string | undefined
  if (db) {
    try {
      const row = await db.prepare('SELECT plan FROM user WHERE id = ? LIMIT 1').bind(session.sub).first<{ plan?: string }>()
      plan = row?.plan ?? undefined
    } catch { /* optional column */ }
  }

  const caps: string[] = plan === 'pro' ? ['pro'] : []
  const issuer = c.env.SHIP_ATTESTATION_ISSUER ?? 'https://getship.dev'
  const ttlSeconds = 3600

  let jwt: string
  try {
    jwt = await signDaemonJwt({ env: c.env, issuer, userId: session.sub, plan, caps, ttlSeconds })
  } catch (e) {
    return c.json({ error: 'signing unavailable', detail: String(e) }, 503)
  }

  return c.json({ jwt, issuer, expires_at: Math.floor(Date.now() / 1000) + ttlSeconds })
})

// ── Better Auth passthrough (catch-all, must be last) ─────────────────────────
// BA is configured with basePath '/v1/auth' (apps/gateway/src/lib/auth.ts), so
// its internal route table generates /v1/auth/* URLs — matching what's mounted
// here.

auth.on(['GET', 'POST'], '/v1/auth/*', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_AUTH', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const a = await getAuth(c.env)
  return a.handler(c.req.raw)
})

export { auth as authRoutes }
