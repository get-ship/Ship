/**
 * Auth route hardening tests — PKCE format validation and rate-limit wiring
 * on the gateway auth endpoints.
 */
import { describe, it, expect, vi } from 'vitest'

vi.mock('../src/lib/rate-limit.js', () => ({
  checkRateLimit: vi.fn().mockResolvedValue({ allowed: true, retryAfter: 0 }),
  rateLimitResponse: vi.fn(),
}))

vi.mock('../src/lib/d1.js', () => ({
  getAuthDb: vi.fn(() => null),
  getRegistryDb: vi.fn(() => null),
  nanoid: vi.fn(() => 'test-id'),
}))

vi.mock('../src/lib/auth.js', () => ({
  getAuth: vi.fn(() => Promise.resolve({ handler: vi.fn() })),
}))

vi.mock('../src/lib/daemon-jwt.js', () => ({
  signDaemonJwt: vi.fn(),
  buildJwks: vi.fn().mockResolvedValue({ keys: [] }),
}))

const env = {
  GITHUB_CLIENT_ID: 'gh-client',
  GITHUB_CLIENT_SECRET: 'gh-secret',
} as unknown as Env

const VALID_CHALLENGE = 'a'.repeat(43)
const VALID_REDIRECT = 'http://localhost:9000/cb'

async function getAuthApp() {
  const { authRoutes } = await import('../src/routes/auth.js')
  const { Hono } = await import('hono')
  const app = new Hono<{ Bindings: Env }>()
  app.route('/', authRoutes)
  return app
}

describe('gateway auth — PKCE code_challenge format', () => {
  it('rejects challenge shorter than 43 chars', async () => {
    const app = await getAuthApp()
    const url = `http://localhost/v1/auth/cli?code_challenge=tooShort&redirect_uri=${encodeURIComponent(VALID_REDIRECT)}`
    const res = await app.fetch(new Request(url), env)
    expect(res.status).toBe(400)
    const body = (await res.json()) as { error: string }
    expect(body.error).toMatch(/code_challenge/)
  })

  it('rejects challenge with non-url-safe characters', async () => {
    const app = await getAuthApp()
    const bad = '!'.repeat(43)
    const url = `http://localhost/v1/auth/cli?code_challenge=${encodeURIComponent(bad)}&redirect_uri=${encodeURIComponent(VALID_REDIRECT)}`
    const res = await app.fetch(new Request(url), env)
    expect(res.status).toBe(400)
  })

  it('rejects challenge longer than 128 chars', async () => {
    const app = await getAuthApp()
    const tooLong = 'a'.repeat(129)
    const url = `http://localhost/v1/auth/cli?code_challenge=${tooLong}&redirect_uri=${encodeURIComponent(VALID_REDIRECT)}`
    const res = await app.fetch(new Request(url), env)
    expect(res.status).toBe(400)
  })

  it('passes regex but still fails DB check (proves format gate runs first)', async () => {
    const app = await getAuthApp()
    // 503 = "Database unavailable" — past the regex gate.
    const url = `http://localhost/v1/auth/cli?code_challenge=${VALID_CHALLENGE}&redirect_uri=${encodeURIComponent(VALID_REDIRECT)}`
    const res = await app.fetch(new Request(url), env)
    expect(res.status).toBe(503)
  })
})
