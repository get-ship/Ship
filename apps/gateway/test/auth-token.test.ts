import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('../src/lib/cloud-auth.js', () => ({
  signJwt: vi.fn().mockResolvedValue('signed-jwt-token'),
  getSecret: vi.fn().mockReturnValue('test-secret-key'),
  verifyJwt: vi.fn().mockResolvedValue(null),
}))
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
vi.mock('../src/lib/auth.js', () => ({ getAuth: vi.fn() }))
vi.mock('../src/lib/session-auth.js', () => ({ requireSession: vi.fn(), optionalSession: vi.fn() }))
vi.mock('../src/lib/daemon-jwt.js', () => ({ signDaemonJwt: vi.fn(), buildJwks: vi.fn().mockResolvedValue({ keys: [] }) }))

import { authRoutes } from '../src/routes/auth.js'
import * as rateLimit from '../src/lib/rate-limit.js'

const mockEnv = {} as Env

function makeRequest(body: unknown) {
  return new Request('http://localhost/api/auth/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
}

describe('POST /api/auth/token', () => {
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
    const r = new Request('http://localhost/api/auth/token', {
      method: 'POST', headers: { 'Content-Type': 'application/json' }, body: 'not-json',
    })
    const res = await authRoutes.request(r, mockEnv)
    expect(res.status).toBe(400)
  })
})
