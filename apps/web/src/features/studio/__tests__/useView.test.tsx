import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import { useView } from '../useView'
import { PlatformProvider } from '../PlatformProvider'
import type { ShipClient } from '@ship/sdk'

// ── Mocks ────────────────────────────────────────────────────────────────────

function createMockClient(overrides?: Partial<ShipClient>): ShipClient {
  return {
    setToken: vi.fn(),
    getToken: vi.fn(() => null),
    serverAttest: vi.fn(async () => 'ship_sess_attested'),
    get: vi.fn(async () => ({ workspaces: [] })),
    pairInit: vi.fn(async () => ({ code: 'A-B', claim_token: 'x', code_expires_in_secs: 300, dev_mode: false })),
    pairStatus: vi.fn(async () => ({ status: 'approved', session_token: 'ship_sess_test' })),
    isConnected: vi.fn(async () => true),
    connectEvents: vi.fn(),
    disconnectEvents: vi.fn(),
    destroy: vi.fn(),
    query: vi.fn(() => ({
      data: undefined,
      error: null,
      isLoading: true,
      isSuccess: false,
      isError: false,
      refetch: vi.fn(),
      subscribe: vi.fn(() => vi.fn()),
    })),
    invalidate: vi.fn(),
    invalidatePrefix: vi.fn(),
    onEvent: vi.fn(() => vi.fn()),
    ...overrides,
  } as unknown as ShipClient
}

vi.mock('@ship/sdk', () => ({
  ShipClient: vi.fn(() => createMockClient()),
}))

vi.mock('#/lib/daemon-config', () => ({
  DAEMON_BASE_URL: 'http://localhost:9315',
  setDaemonToken: vi.fn(),
  daemonFetch: vi.fn(),
}))

const originalFetch = globalThis.fetch

beforeEach(() => {
  globalThis.fetch = vi.fn(async (input: RequestInfo | URL) => {
    const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url
    if (url.includes('/api/auth/daemon-attest')) {
      return new Response(JSON.stringify({ jwt: 'test-jwt' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    }
    return new Response('', { status: 404 })
  }) as typeof fetch
})

afterEach(() => {
  globalThis.fetch = originalFetch
  cleanup()
})

// ── Test helper ──────────────────────────────────────────────────────────────

function ViewConsumer() {
  const view = useView()
  return (
    <div>
      <span data-testid="daemon">{view.daemon}</span>
      <span data-testid="auth">{view.auth}</span>
      <span data-testid="workspace">{view.workspace?.name ?? 'none'}</span>
      <span data-testid="has-client">{String(!!view.client)}</span>
      <span data-testid="require-daemon">{String(view.requireDaemon() !== null)}</span>
    </div>
  )
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe('useView', () => {
  it('settles to connected after server-attest succeeds', async () => {
    const { waitFor } = await import('@testing-library/react')
    render(
      <PlatformProvider>
        <ViewConsumer />
      </PlatformProvider>,
    )
    await waitFor(() => {
      expect(screen.getByTestId('daemon').textContent).toBe('connected')
    })
  })

  it('returns auth state from platform', () => {
    render(
      <PlatformProvider>
        <ViewConsumer />
      </PlatformProvider>,
    )
    expect(screen.getByTestId('auth').textContent).toBeTruthy()
  })

  it('returns current workspace from platform', () => {
    render(
      <PlatformProvider>
        <ViewConsumer />
      </PlatformProvider>,
    )
    expect(screen.getByTestId('workspace').textContent).toBe('none')
  })

  it('exposes ShipClient for direct queries', () => {
    render(
      <PlatformProvider>
        <ViewConsumer />
      </PlatformProvider>,
    )
    expect(screen.getByTestId('has-client').textContent).toBe('true')
  })

  it('requireDaemon returns null when daemon is not yet connected', () => {
    globalThis.fetch = vi.fn(async () =>
      new Response('', { status: 401 }),
    ) as typeof fetch
    render(
      <PlatformProvider>
        <ViewConsumer />
      </PlatformProvider>,
    )
    expect(screen.getByTestId('require-daemon').textContent).toBe('false')
  })

  it('requireDaemon returns client when daemon is connected', async () => {
    const { waitFor } = await import('@testing-library/react')
    render(
      <PlatformProvider>
        <ViewConsumer />
      </PlatformProvider>,
    )
    await waitFor(() => {
      expect(screen.getByTestId('daemon').textContent).toBe('connected')
    })
    expect(screen.getByTestId('require-daemon').textContent).toBe('true')
  })

  it('throws when used outside PlatformProvider', () => {
    expect(() => render(<ViewConsumer />)).toThrow()
  })
})
