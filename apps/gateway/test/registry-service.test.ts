import { describe, it, expect, vi, beforeEach } from 'vitest'
import { decodeJwt } from 'jose'

vi.mock('../src/lib/d1.js', () => ({ getRegistryDb: vi.fn(() => ({})), getAuthDb: vi.fn() }))
vi.mock('../src/data/registry.repository.js', () => ({ createRegistryRepositories: vi.fn() }))

import { createContextValues } from '@connectrpc/connect'
import { Code, ConnectError, type HandlerContext } from '@connectrpc/connect'
import { create } from '@bufbuild/protobuf'
import {
  FetchBundleRequestSchema,
  ResolveVersionRequestSchema,
  PublishBundleRequestSchema,
} from '../src/proto/gen/ship/gateway/v1/registry_pb.js'
import { registryServiceImpl } from '../src/services/registry-impl.js'
import { envContextKey } from '../src/lib/connect-context.js'
import * as rr from '../src/data/registry.repository.js'

// The slice-1 test Ed25519 key (offline-generated) — shared with the Rust
// verifier's fixtures so a real round-trip is possible.
const TEST_PRIV_PEM =
  '-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIAFISiAspm6fXMD6Gg3dlW8qtiRFxoiHbiCjCuys/rg5\n-----END PRIVATE KEY-----\n'

const env = {
  SHIP_JWT_PRIVATE_KEY: TEST_PRIV_PEM,
  SHIP_JWT_PRIVATE_KEY_ID: 'test-publisher-1',
} as unknown as Env

function ctx(): HandlerContext {
  return { values: createContextValues().set(envContextKey, env) } as HandlerContext
}

function mockRepos(over: Record<string, unknown> = {}) {
  vi.mocked(rr.createRegistryRepositories).mockReturnValue({
    getPackage: vi.fn().mockResolvedValue({
      id: 'pkg-1',
      path: 'github.com/owner/notes',
      repoUrl: 'https://github.com/owner/notes',
    }),
    getPackageVersions: vi.fn().mockResolvedValue([
      { version: '1.0.0', commitSha: 'abc123', contentHash: 'sha256:deadbeef' },
      { version: '1.1.0', commitSha: 'def456', contentHash: 'sha256:cafef00d' },
    ]),
    ...over,
  } as unknown as ReturnType<typeof rr.createRegistryRepositories>)
}

beforeEach(() => vi.clearAllMocks())

describe('RegistryService.fetchBundle', () => {
  it('returns the source url + an attestation binding the contentHash', async () => {
    mockRepos()
    const req = create(FetchBundleRequestSchema, { path: 'github.com/owner/notes', version: '1.0.0' })
    const res = await registryServiceImpl.fetchBundle(req, ctx())

    expect(res.sha256).toBe('sha256:deadbeef')
    expect(res.downloadUrl).toBe('https://api.github.com/repos/owner/notes/tarball/abc123')
    // The attestation is a real EdDSA JWT binding {path, version, sha256}.
    expect(res.attestationJwt.split('.')).toHaveLength(3)
    const claim = decodeJwt(res.attestationJwt)
    expect(claim.path).toBe('github.com/owner/notes')
    expect(claim.version).toBe('1.0.0')
    expect(claim.sha256).toBe('sha256:deadbeef')
  })

  it('404s an unknown version', async () => {
    mockRepos()
    const req = create(FetchBundleRequestSchema, { path: 'github.com/owner/notes', version: '9.9.9' })
    await expect(registryServiceImpl.fetchBundle(req, ctx())).rejects.toMatchObject({
      code: Code.NotFound,
    })
  })
})

describe('RegistryService.resolveVersion', () => {
  it('resolves "*" to the highest published version', async () => {
    mockRepos()
    const req = create(ResolveVersionRequestSchema, { path: 'github.com/owner/notes', constraint: '*' })
    const res = await registryServiceImpl.resolveVersion(req, ctx())
    expect(res.version).toBe('1.1.0')
    expect(res.sha256).toBe('sha256:cafef00d')
  })

  it('resolves an exact constraint', async () => {
    mockRepos()
    const req = create(ResolveVersionRequestSchema, { path: 'github.com/owner/notes', constraint: '1.0.0' })
    const res = await registryServiceImpl.resolveVersion(req, ctx())
    expect(res.version).toBe('1.0.0')
  })
})

describe('RegistryService.publishBundle', () => {
  it('is Unimplemented (legacy proxy still serves publishing)', async () => {
    const req = create(PublishBundleRequestSchema, {
      path: 'p',
      version: '1.0.0',
      sha256: 'sha256:x',
      attestationJwt: 'j',
    })
    await expect(registryServiceImpl.publishBundle(req, ctx())).rejects.toMatchObject({
      code: Code.Unimplemented,
    })
  })
})
