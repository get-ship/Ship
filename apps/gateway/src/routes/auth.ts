import { Hono } from 'hono'
import { getAuth } from '../lib/auth.js'
import { requireSession } from '../lib/session-auth.js'
// CLI auth tokens are Better Auth API keys (migrated 2026-05 from custom
// HS256 JWTs). Mint via auth.api.createApiKey in the PKCE exchange handler.
// Validation runs through Better Auth's session middleware automatically
// (enableSessionForAPIKeys is on; see ../lib/auth.ts).
import { signDaemonJwt, buildJwks } from '../lib/daemon-jwt.js'
import { getAuthDb, nanoid } from '../lib/d1.js'
import { checkRateLimit, rateLimitResponse } from '../lib/rate-limit.js'

const auth = new Hono<{ Bindings: Env }>()

// ── JWKS ─────────────────────────────────────────────────────────────────────

auth.get('/api/jwks', async (c) => {
  const keys = await buildJwks(c.env)
  return c.json(keys)
})

// ── Identity ─────────────────────────────────────────────────────────────────

auth.get('/api/me', async (c) => {
  const session = await requireSession(c.req.raw, c.env)
  if (session instanceof Response) return session
  const db = getAuthDb(c.env)
  if (!db) return c.json({ error: 'Database unavailable' }, 503)
  const user = await db.prepare('SELECT id, name, email, image FROM user WHERE id = ?').bind(session.sub).first()
  if (!user) return c.json({ error: 'User not found' }, 404)
  return c.json({ user })
})

auth.get('/api/health', (c) => c.json({ status: 'ok' }))

// ── CLI PKCE OAuth ────────────────────────────────────────────────────────────

auth.get('/auth/cli', async (c) => {
  const url = new URL(c.req.url)
  const codeChallenge = url.searchParams.get('code_challenge')
  const redirectUri = url.searchParams.get('redirect_uri')

  if (!codeChallenge || !redirectUri) {
    return c.json({ error: 'Missing code_challenge or redirect_uri' }, 400)
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

  const callbackUrl = `${url.origin}/auth/cli-callback`
  const ghUrl = new URL('https://github.com/login/oauth/authorize')
  ghUrl.searchParams.set('client_id', clientId)
  ghUrl.searchParams.set('redirect_uri', callbackUrl)
  ghUrl.searchParams.set('scope', 'read:user user:email')
  ghUrl.searchParams.set('state', state)

  return new Response(null, { status: 302, headers: { Location: ghUrl.toString() } })
})

auth.get('/auth/cli-callback', async (c) => {
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

// ── Token exchange ────────────────────────────────────────────────────────────

async function sha256Base64url(input: string): Promise<string> {
  const data = new TextEncoder().encode(input)
  const hash = await crypto.subtle.digest('SHA-256', data)
  const bytes = new Uint8Array(hash)
  let binary = ''
  for (const b of bytes) binary += String.fromCharCode(b)
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '')
}

// Token exchange — PKCE: trades a one-time auth code (created by the CLI
// browser callback) for a Better Auth API key. The CLI stores the
// returned `key` and sends it as `Authorization: Bearer <key>` (or
// `x-api-key: <key>`) on subsequent gateway calls.
//
// Migrated 2026-05 from custom HS256 JWT → Better Auth API key. Migration
// note: existing daemons holding HS256 JWTs are invalidated; they re-login
// via `ship login` to get a key.
auth.post('/api/auth/token', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_CLAIM', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  let body: unknown
  try { body = await c.req.json() } catch { return c.json({ error: 'Invalid JSON body' }, 400) }

  const b = body as Record<string, unknown>
  if (!body || typeof b['code'] !== 'string' || typeof b['verifier'] !== 'string') {
    return c.json({ error: 'Missing code or verifier' }, 400)
  }
  const { code, verifier } = b as { code: string; verifier: string }

  const db = getAuthDb(c.env)
  if (!db) return c.json({ error: 'Database unavailable' }, 503)

  const claim = await db.prepare('UPDATE cli_auth_codes SET used = 1 WHERE code = ? AND used = 0').bind(code).run()
  if (!claim.meta.changes) return c.json({ error: 'Invalid or already-used auth code' }, 401)

  const row = await db
    .prepare('SELECT user_id, code_challenge, created_at FROM cli_auth_codes WHERE code = ?')
    .bind(code)
    .first<{ user_id: string; code_challenge: string; created_at: number }>()

  if (!row) return c.json({ error: 'Invalid auth code' }, 401)
  if (Date.now() - row.created_at > 5 * 60 * 1000) return c.json({ error: 'Auth code expired' }, 401)

  const computed = await sha256Base64url(verifier)
  if (computed !== row.code_challenge) return c.json({ error: 'PKCE verification failed' }, 401)

  const a = await getAuth(c.env)
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

  return c.json({
    key: result.key,
    user_id: row.user_id,
    expires_at: result.expiresAt ? new Date(result.expiresAt).getTime() / 1000 : null,
  })
})

// ── Token refresh ─────────────────────────────────────────────────────────────
// With Better Auth API keys (1-year TTL by default), the daemon mints a
// fresh key by re-running `ship login` rather than refreshing in-place.
// This endpoint is now a thin "is my key still valid + how long left"
// check; it doesn't extend or re-issue. Callers needing a longer-lived
// key call POST /api/auth/token via the PKCE flow again.

auth.post('/api/auth/refresh', async (c) => {
  const rl = await checkRateLimit(c.req.raw, 'RATE_LIMITER_CLAIM', 60, c.env)
  if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

  const session = await requireSession(c.req.raw, c.env)
  if (session instanceof Response) return session

  // requireSession resolves the API key via the plugin's middleware.
  // If we get here, the key is valid; report the remaining TTL.
  return c.json({
    ok: true,
    user_id: session.sub,
    refresh_via: 'pkce',
    note: 'API keys do not refresh in place; run `ship login` to get a fresh key.',
  })
})

// ── Daemon attestation ────────────────────────────────────────────────────────

auth.post('/api/auth/daemon-attest', async (c) => {
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
  // Issuer is the brand identity, not the request host. Pinning it stable
  // across api.getship.dev / staging.getship.dev / localhost means the
  // daemon's `iss` validation doesn't need per-env config. JWKS URL is
  // env-specific and configured separately on the daemon side.
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

auth.on(['GET', 'POST'], '/api/auth/*', async (c) => {
  const a = await getAuth(c.env)
  return a.handler(c.req.raw)
})

export { auth as authRoutes }
