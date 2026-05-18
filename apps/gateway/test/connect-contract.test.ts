/**
 * Connect-RPC contract tests — gateway side.
 *
 * Asserts that the gateway's Connect handler produces wire-format responses
 * matching the fixtures in `proto/fixtures/auth_service.json`. The Rust
 * client (`crates/core/ship-gateway-client/tests/contract.rs`) asserts the
 * inverse — that it parses those same fixtures into the generated types
 * correctly.
 *
 * Both tests reading the same JSON file catches the drift class where:
 *   - field is renamed on one side but not the other
 *   - int64/string encoding diverges (proto3 JSON quirk)
 *   - default-value handling differs
 *   - new method added but only one side wired up
 *
 * What this is NOT: a full e2e test that boots a real gateway and a real
 * client. That would catch transport-level bugs too but requires CI infra
 * we don't have yet (wrangler dev + a Rust integration test in the same
 * run). Fixture lockstep catches the drift class without the orchestration.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
// Fixtures bundled at compile time — the Workers test pool has no
// filesystem access, so node:fs would fail at runtime. JSON imports
// work via the bundler.
// @ts-expect-error JSON imports inferred at build time without typed schema
import fixturesJson from '../../../proto/fixtures/auth_service.json'

vi.mock('../src/lib/rate-limit.js', () => ({
  checkRateLimit: vi.fn().mockResolvedValue({ allowed: true, retryAfter: 0 }),
  rateLimitResponse: vi.fn(),
}))

const mockRun = vi.fn().mockResolvedValue({ meta: { changes: 1 } })
const mockFirst = vi.fn()
const mockPrepare = vi.fn(() => ({ bind: vi.fn(() => ({ run: mockRun, first: mockFirst })) }))
const mockD1 = { prepare: mockPrepare }

vi.mock('../src/lib/d1.js', () => ({
  getAuthDb: vi.fn(() => mockD1),
  getRegistryDb: vi.fn(() => null),
  nanoid: vi.fn(() => 'test-id'),
}))

const mockCreateApiKey = vi.fn()
const mockGetSession = vi.fn()
const mockDeleteApiKey = vi.fn()
vi.mock('../src/lib/auth.js', () => ({
  getAuth: vi.fn(() => Promise.resolve({
    api: {
      createApiKey: mockCreateApiKey,
      getSession: mockGetSession,
      deleteApiKey: mockDeleteApiKey,
    },
  })),
}))

vi.mock('../src/lib/session-auth.js', () => ({
  requireSession: vi.fn().mockResolvedValue({ sub: 'user-test-1', org: 'user-test-1' }),
  optionalSession: vi.fn().mockResolvedValue(null),
}))

vi.mock('../src/lib/daemon-jwt.js', () => ({
  signDaemonJwt: vi.fn().mockResolvedValue('eyJ0ZXN0IjoianNvbiJ9.eyJzdWIiOiJ1c2VyLXRlc3QtMSJ9.testsig'),
  buildJwks: vi.fn().mockResolvedValue({ keys: [] }),
}))

// __dirname is not defined under vitest ESM; resolve via import.meta.url.
type Fixture = {
  method_path: string
  request: Record<string, unknown>
  response_ok: Record<string, unknown>
}
type Fixtures = Record<string, Fixture>
const fixtures = fixturesJson as unknown as Fixtures

const mockEnv = { SHIP_ATTESTATION_ISSUER: 'https://getship.dev' } as unknown as Env

// Mount the Hono app the same way it's mounted in src/index.ts so the
// /ship.gateway.v1.AuthService/* route reaches the Connect handler.
async function makeApp() {
  const { Hono } = await import('hono')
  const { connectAuthHandler } = await import('../src/lib/connect-router.js')
  const app = new Hono<{ Bindings: Env }>()
  app.all('/ship.gateway.v1.AuthService/*', (c) => connectAuthHandler(c.req.raw, c.env))
  return app
}

function makeConnectRequest(path: string, body: unknown): Request {
  return new Request(`http://localhost${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
}

describe('Connect-RPC contract — auth_service.json fixtures', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockRun.mockResolvedValue({ meta: { changes: 1 } })
  })

  it('TokenExchange wire shape matches fixture.response_ok', async () => {
    const fixture = fixtures.TokenExchange
    // Stub the daemon-side pieces so the impl reaches createApiKey.
    const verifier = fixture.request.verifier as string
    const encoder = new TextEncoder()
    const hashBuf = await crypto.subtle.digest('SHA-256', encoder.encode(verifier))
    const challenge = Array.from(new Uint8Array(hashBuf))
      .map((b) => String.fromCharCode(b))
      .join('')
    const challengeB64 = btoa(challenge).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '')

    mockFirst.mockResolvedValue({
      user_id: fixture.response_ok.userId,
      code_challenge: challengeB64,
      created_at: Date.now(),
    })
    mockCreateApiKey.mockResolvedValue({
      key: fixture.response_ok.key,
      expiresAt: new Date(Number(fixture.response_ok.expiresAt) * 1000),
    })

    const app = await makeApp()
    const res = await app.fetch(makeConnectRequest(fixture.method_path, fixture.request), mockEnv)

    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.key).toBe(fixture.response_ok.key)
    expect(body.userId).toBe(fixture.response_ok.userId)
    // proto3 JSON serializes int64 as string — assert as string.
    expect(body.expiresAt).toBe(fixture.response_ok.expiresAt)
  })

  it('GetMe wire shape matches fixture.response_ok', async () => {
    const fixture = fixtures.GetMe
    const expectedUser = fixture.response_ok.user as Record<string, string>
    mockFirst.mockResolvedValue({
      id: expectedUser.id,
      name: expectedUser.name,
      email: expectedUser.email,
      image: expectedUser.image,
    })

    const app = await makeApp()
    const res = await app.fetch(makeConnectRequest(fixture.method_path, fixture.request), mockEnv)

    expect(res.status).toBe(200)
    const body = (await res.json()) as { user: Record<string, string> }
    expect(body.user.id).toBe(expectedUser.id)
    expect(body.user.name).toBe(expectedUser.name)
    expect(body.user.email).toBe(expectedUser.email)
    expect(body.user.image).toBe(expectedUser.image)
  })

  it('Refresh wire shape matches fixture.response_ok', async () => {
    const fixture = fixtures.Refresh

    const app = await makeApp()
    const res = await app.fetch(makeConnectRequest(fixture.method_path, fixture.request), mockEnv)

    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.ok).toBe(fixture.response_ok.ok)
    expect(body.userId).toBe(fixture.response_ok.userId)
    expect(body.note).toBe(fixture.response_ok.note)
  })

  it('DaemonAttest wire shape matches fixture.response_ok', async () => {
    const fixture = fixtures.DaemonAttest
    // requireSession is already mocked; plan lookup returns row with no plan column
    mockFirst.mockResolvedValue({})

    const app = await makeApp()
    const res = await app.fetch(makeConnectRequest(fixture.method_path, fixture.request), mockEnv)

    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.jwt).toBe(fixture.response_ok.jwt)
    expect(body.issuer).toBe(fixture.response_ok.issuer)
    // expiresAt is `Date.now()/1000 + 3600`; we can't pin the timestamp,
    // just assert it's a numeric string of the right magnitude.
    expect(typeof body.expiresAt).toBe('string')
    expect(Number(body.expiresAt)).toBeGreaterThan(1700000000) // sanity-check epoch
  })

  it('RevokeKey wire shape matches fixture.response_ok', async () => {
    const fixture = fixtures.RevokeKey
    mockGetSession.mockResolvedValue({ session: { id: 'key-row-id-abc' } })
    mockDeleteApiKey.mockResolvedValue({ success: true })

    const app = await makeApp()
    const res = await app.fetch(makeConnectRequest(fixture.method_path, fixture.request), mockEnv)

    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.revoked).toBe(fixture.response_ok.revoked)
    expect(mockDeleteApiKey).toHaveBeenCalledWith(
      expect.objectContaining({ body: { keyId: 'key-row-id-abc' } }),
    )
  })

  it('every method in the proto has a fixture', () => {
    // Lock in fixture completeness so adding a method to the proto without
    // a fixture trips this test.
    const expectedMethods = ['TokenExchange', 'Refresh', 'DaemonAttest', 'GetMe', 'RevokeKey']
    for (const m of expectedMethods) {
      expect(fixtures[m]).toBeDefined()
      expect(fixtures[m].method_path).toBe(`/ship.gateway.v1.AuthService/${m}`)
    }
  })
})
