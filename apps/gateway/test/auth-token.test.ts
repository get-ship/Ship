import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('../src/lib/rate-limit.js', () => ({
  checkRateLimit: vi.fn().mockResolvedValue({ allowed: true, retryAfter: 0 }),
  rateLimitResponse: vi.fn((retryAfter: number) =>
    Response.json({ error: 'Rate limit exceeded', retryAfter }, { status: 429 }),
  ),
}))

const mockRun = vi.fn().mockResolvedValue({ meta: { changes: 1 } })
const mockFirst = vi.fn()
const mockBind = vi.fn(() => ({ run: mockRun, first: mockFirst }))
const mockPrepare = vi.fn(() => ({ bind: mockBind }))
const mockD1 = { prepare: mockPrepare }

vi.mock('../src/lib/d1.js', () => ({
  getAuthDb: vi.fn(() => mockD1),
  getRegistryDb: vi.fn(() => null),
  nanoid: vi.fn(() => 'test-id'),
}))
const mockCreateApiKey = vi.fn()
vi.mock('../src/lib/auth.js', () => ({
  getAuth: vi.fn(() => Promise.resolve({
    api: { createApiKey: mockCreateApiKey },
  })),
}))
vi.mock('../src/lib/session-auth.js', () => ({ requireSession: vi.fn(), optionalSession: vi.fn() }))
vi.mock('../src/lib/daemon-jwt.js', () => ({ signDaemonJwt: vi.fn(), buildJwks: vi.fn().mockResolvedValue({ keys: [] }) }))

import { authRoutes } from '../src/routes/auth.js'
import * as rateLimit from '../src/lib/rate-limit.js'

const mockEnv = {} as Env

function makeRequest(body: unknown) {
  return new Request('http://localhost/v1/auth/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
}

describe('POST /v1/auth/token', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(rateLimit.checkRateLimit).mockResolvedValue({ allowed: true, retryAfter: 0 })
    vi.mocked(rateLimit.rateLimitResponse).mockImplementation((retryAfter: number) =>
      Response.json({ error: 'Rate limit exceeded', retryAfter }, { status: 429 }),
    )
    mockRun.mockResolvedValue({ meta: { changes: 1 } })
    mockFirst.mockResolvedValue({
      user_id: 'user-1',
      code_challenge: 'challenge',
      created_at: Date.now(),
    })
  })

  it('returns 401 when code was already consumed', async () => {
    mockRun.mockResolvedValue({ meta: { changes: 0 } })
    const res = await authRoutes.request(makeRequest({ code: 'used-code', verifier: 'verifier' }), mockEnv)
    expect(res.status).toBe(401)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.error).toMatch(/invalid or already-used/i)
  })

  it('returns 429 when rate limited', async () => {
    vi.mocked(rateLimit.checkRateLimit).mockResolvedValue({ allowed: false, retryAfter: 60 })
    const res = await authRoutes.request(makeRequest({ code: 'code', verifier: 'verifier' }), mockEnv)
    expect(res.status).toBe(429)
  })

  it('returns 400 for missing verifier', async () => {
    const res = await authRoutes.request(makeRequest({ code: 'code' }), mockEnv)
    expect(res.status).toBe(400)
  })

  it('returns 400 for invalid JSON', async () => {
    const r = new Request('http://localhost/v1/auth/token', {
      method: 'POST', headers: { 'Content-Type': 'application/json' }, body: 'not-json',
    })
    const res = await authRoutes.request(r, mockEnv)
    expect(res.status).toBe(400)
  })

  it('mints a Better Auth API key on PKCE success', async () => {
    // Make PKCE verification succeed — code_challenge must match sha256
    // of the verifier. Use a known verifier with its precomputed challenge.
    const verifier = 'test-verifier-string-1234567890'
    const encoder = new TextEncoder()
    const hashBuf = await crypto.subtle.digest('SHA-256', encoder.encode(verifier))
    const challenge = Array.from(new Uint8Array(hashBuf))
      .map(b => String.fromCharCode(b))
      .join('')
    const challengeB64 = btoa(challenge).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '')
    mockFirst.mockResolvedValue({
      user_id: 'user-1',
      code_challenge: challengeB64,
      created_at: Date.now(),
    })
    const expiresAt = new Date(Date.now() + 365 * 24 * 60 * 60 * 1000)
    mockCreateApiKey.mockResolvedValue({ key: 'ship_test_key_abc123', expiresAt })

    const res = await authRoutes.request(
      makeRequest({ code: 'valid-code', verifier }),
      mockEnv,
    )
    expect(res.status).toBe(200)
    const body = (await res.json()) as Record<string, unknown>
    expect(body.key).toBe('ship_test_key_abc123')
    expect(body.user_id).toBe('user-1')
    expect(body.expires_at).toBeTypeOf('number')

    // Verify the API key was minted with the right shape
    expect(mockCreateApiKey).toHaveBeenCalledWith({
      body: expect.objectContaining({
        userId: 'user-1',
        prefix: 'ship_',
        expiresIn: 365 * 24 * 60 * 60,
      }),
    })
  })
})
