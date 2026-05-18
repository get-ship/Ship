import { describe, it, expect, vi, beforeEach } from 'vitest'

// Auth + membership gating in front of /v1/sync, /v1/projects, /v1/jobs.
//
// Membership lives in the per-project Platform DO. Tests mock the DO
// `projects()` facade so we never touch real storage — that's covered by
// the platform worker's own membership integration tests.

vi.mock('../src/lib/session-auth.js', () => ({
  requireSession: vi.fn(),
  optionalSession: vi.fn(),
}))

vi.mock('../src/lib/d1.js', () => ({
  getAuthDb: vi.fn(() => null),
  getRegistryDb: vi.fn(() => null),
  nanoid: vi.fn(() => 'test-id'),
}))

import app from '../src/index.js'
import * as sessionAuth from '../src/lib/session-auth.js'

function makeProjectsFacade(overrides: Record<string, unknown> = {}) {
  return {
    list: vi.fn().mockResolvedValue([]),
    listMembers: vi.fn().mockResolvedValue([]),
    isMember: vi.fn().mockResolvedValue(false),
    addMember: vi.fn().mockResolvedValue(undefined),
    removeMember: vi.fn().mockResolvedValue(undefined),
    claimOrCheck: vi.fn().mockResolvedValue({ authorized: true, claimed: true }),
    ...overrides,
  }
}

function makeSyncFacade(overrides: Record<string, unknown> = {}) {
  return {
    push: vi.fn().mockResolvedValue({ accepted: 1 }),
    pull: vi.fn().mockResolvedValue({ events: [], cursor: null }),
    ...overrides,
  }
}

function makeJobsFacade(overrides: Record<string, unknown> = {}) {
  return {
    list: vi.fn().mockResolvedValue([]),
    ...overrides,
  }
}

function mockEnv({
  projects = makeProjectsFacade(),
  sync = makeSyncFacade(),
  jobs = makeJobsFacade(),
}: {
  projects?: ReturnType<typeof makeProjectsFacade>
  sync?: ReturnType<typeof makeSyncFacade>
  jobs?: ReturnType<typeof makeJobsFacade>
} = {}) {
  const stub = {
    projects: vi.fn().mockResolvedValue(projects),
    sync: vi.fn().mockResolvedValue(sync),
    jobs: vi.fn().mockResolvedValue(jobs),
  }
  return {
    PLATFORM: {
      idFromName: vi.fn((name: string) => ({ name })),
      get: vi.fn(() => stub),
    },
    PROJECT: {
      idFromName: vi.fn((name: string) => ({ name })),
      get: vi.fn(() => stub),
    },
  } as unknown as Env
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('requireProjectAuth — authentication layer', () => {
  it('rejects unauthenticated push with 401', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 }),
    )
    const env = mockEnv()
    const req = new Request('http://localhost/v1/sync/platform/push', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'x-project-id': 'proj-1' },
      body: JSON.stringify({ events: [] }),
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(401)
  })

  it('rejects unauthenticated GET /v1/projects with 401', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 }),
    )
    const env = mockEnv()
    const req = new Request('http://localhost/v1/projects', {
      headers: { 'x-project-id': 'proj-1' },
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(401)
  })

  it('rejects unauthenticated GET /v1/jobs with 401', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue(
      Response.json({ error: 'Authentication required' }, { status: 401 }),
    )
    const env = mockEnv()
    const req = new Request('http://localhost/v1/jobs', {
      headers: { 'x-project-id': 'proj-1' },
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(401)
  })

  it('still requires x-project-id even when authenticated (400)', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'user-1', org: 'user-1' })
    const env = mockEnv()
    const req = new Request('http://localhost/v1/sync/platform/push', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ events: [] }),
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(400)
  })
})

describe('requireProjectAuth — membership layer', () => {
  it('claims an unowned project for the first authenticated caller (200)', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'creator-1', org: 'creator-1' })
    const projects = makeProjectsFacade({
      claimOrCheck: vi.fn().mockResolvedValue({ authorized: true, claimed: true }),
    })
    const env = mockEnv({ projects })
    const req = new Request('http://localhost/v1/sync/platform/push', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'x-project-id': 'new-proj' },
      body: JSON.stringify({ events: [] }),
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(200)
    expect(projects.claimOrCheck).toHaveBeenCalledWith('creator-1')
  })

  it('returns 403 when caller is not a member of the project', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'intruder', org: 'intruder' })
    const projects = makeProjectsFacade({
      claimOrCheck: vi.fn().mockResolvedValue({ authorized: false, claimed: false }),
    })
    const env = mockEnv({ projects })
    const req = new Request('http://localhost/v1/sync/platform/push', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'x-project-id': 'someone-elses-proj' },
      body: JSON.stringify({ events: [] }),
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(403)
    const body = await res.json() as { error: string }
    expect(body.error).toMatch(/not a member/i)
  })

  it('returns 200 when caller is already a member', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'member-1', org: 'member-1' })
    const projects = makeProjectsFacade({
      claimOrCheck: vi.fn().mockResolvedValue({ authorized: true, claimed: false }),
    })
    const env = mockEnv({ projects })
    const req = new Request('http://localhost/v1/projects', {
      headers: { 'x-project-id': 'existing-proj' },
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(200)
  })

  it('returns 500 when DO membership check throws', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'user-1', org: 'user-1' })
    const projects = makeProjectsFacade({
      claimOrCheck: vi.fn().mockRejectedValue(new Error('storage offline')),
    })
    const env = mockEnv({ projects })
    const req = new Request('http://localhost/v1/sync/platform/push', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'x-project-id': 'proj-1' },
      body: JSON.stringify({ events: [] }),
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(500)
  })
})

describe('/v1/projects/:id/members', () => {
  it('lists members for an authenticated member', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'owner-1', org: 'owner-1' })
    const projects = makeProjectsFacade({
      claimOrCheck: vi.fn().mockResolvedValue({ authorized: true, claimed: false }),
      listMembers: vi.fn().mockResolvedValue([
        { user_id: 'owner-1', role: 'owner', created_at: 't0' },
      ]),
    })
    const env = mockEnv({ projects })
    const req = new Request('http://localhost/v1/projects/proj-1/members', {
      headers: { 'x-project-id': 'proj-1' },
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(200)
    const body = await res.json() as { members: Array<{ user_id: string }> }
    expect(body.members).toHaveLength(1)
    expect(body.members[0]!.user_id).toBe('owner-1')
  })

  it('rejects non-owner adding a member with 403', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'plain-member', org: 'plain-member' })
    const projects = makeProjectsFacade({
      claimOrCheck: vi.fn().mockResolvedValue({ authorized: true, claimed: false }),
      listMembers: vi.fn().mockResolvedValue([
        { user_id: 'owner-1', role: 'owner', created_at: 't0' },
        { user_id: 'plain-member', role: 'member', created_at: 't1' },
      ]),
    })
    const env = mockEnv({ projects })
    const req = new Request('http://localhost/v1/projects/proj-1/members', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'x-project-id': 'proj-1' },
      body: JSON.stringify({ user_id: 'new-user' }),
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(403)
    expect(projects.addMember).not.toHaveBeenCalled()
  })

  it('lets an owner add a member', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'owner-1', org: 'owner-1' })
    const projects = makeProjectsFacade({
      claimOrCheck: vi.fn().mockResolvedValue({ authorized: true, claimed: false }),
      listMembers: vi.fn().mockResolvedValue([
        { user_id: 'owner-1', role: 'owner', created_at: 't0' },
      ]),
    })
    const env = mockEnv({ projects })
    const req = new Request('http://localhost/v1/projects/proj-1/members', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'x-project-id': 'proj-1' },
      body: JSON.stringify({ user_id: 'new-user', role: 'member' }),
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(200)
    expect(projects.addMember).toHaveBeenCalledWith('new-user', 'member')
  })

  it('refuses to remove the last owner', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'owner-1', org: 'owner-1' })
    const projects = makeProjectsFacade({
      claimOrCheck: vi.fn().mockResolvedValue({ authorized: true, claimed: false }),
      listMembers: vi.fn().mockResolvedValue([
        { user_id: 'owner-1', role: 'owner', created_at: 't0' },
      ]),
    })
    const env = mockEnv({ projects })
    const req = new Request('http://localhost/v1/projects/proj-1/members/owner-1', {
      method: 'DELETE',
      headers: { 'x-project-id': 'proj-1' },
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(409)
    expect(projects.removeMember).not.toHaveBeenCalled()
  })

  it('rejects member-route call when path id and x-project-id disagree', async () => {
    vi.mocked(sessionAuth.requireSession).mockResolvedValue({ sub: 'owner-1', org: 'owner-1' })
    const projects = makeProjectsFacade({
      claimOrCheck: vi.fn().mockResolvedValue({ authorized: true, claimed: false }),
    })
    const env = mockEnv({ projects })
    const req = new Request('http://localhost/v1/projects/proj-OTHER/members', {
      headers: { 'x-project-id': 'proj-1' },
    })
    const res = await app.fetch(req, env)
    expect(res.status).toBe(400)
  })
})
