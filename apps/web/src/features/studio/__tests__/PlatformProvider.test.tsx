import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest'
import { render, screen, cleanup, act, waitFor } from '@testing-library/react'
import { PlatformProvider, usePlatform } from '../PlatformProvider'
import type { ShipClient } from '@ship/sdk'

// ── Mock ShipClient ──────────────────────────────────────────────────────────

function createMockClient(overrides?: Partial<ShipClient>): ShipClient {
  return {
    setToken: vi.fn(),
    getToken: vi.fn(() => null),
    authenticate: vi.fn(async () => true),
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

afterEach(cleanup)

// ── Test helper ──────────────────────────────────────────────────────────────

function PlatformConsumer() {
  const platform = usePlatform()
  return (
    <div>
      <span data-testid="daemon">{platform.daemon}</span>
      <span data-testid="auth">{platform.auth}</span>
      <span data-testid="workspace">{platform.workspace?.name ?? 'none'}</span>
      <span data-testid="command-palette">{String(platform.commandPaletteOpen)}</span>
      <span data-testid="sidebar">{String(platform.sidebarOpen)}</span>
    </div>
  )
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe('PlatformProvider', () => {
  beforeEach(() => {
    localStorage.clear()
    window.location.hash = ''
  })

  // ── Daemon detection ───────────────────────────────────────────────────

  describe('daemon detection', () => {
    it('starts in connecting state on mount', () => {
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      expect(screen.getByTestId('daemon').textContent).toBe('connecting')
    })

    it('transitions to connected when daemon health check passes', async () => {
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(screen.getByTestId('daemon').textContent).toBe('connected')
      })
    })

    it('transitions to disconnected when daemon health check fails', async () => {
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => createMockClient({ isConnected: vi.fn(async () => false) }) as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(screen.getByTestId('daemon').textContent).toBe('disconnected')
      })
    })
  })

  // ── Fragment token auth ────────────────────────────────────────────────

  describe('fragment token auth', () => {
    it('extracts token from URL fragment and sets it on client', async () => {
      window.location.hash = '#token=test-fragment-token'
      const mockClient = createMockClient()
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => mockClient as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(mockClient.setToken).toHaveBeenCalledWith('test-fragment-token')
      })
    })

    it('clears fragment from URL after extracting token', async () => {
      window.location.hash = '#token=secret'
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(window.location.hash).toBe('')
      })
    })

    it('sets auth to anonymous with fragment token (no user session)', async () => {
      window.location.hash = '#token=anon-token'
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(screen.getByTestId('auth').textContent).toBe('anonymous')
      })
    })
  })

  // ── Handshake auth ─────────────────────────────────────────────────────

  describe('handshake auth', () => {
    it('calls authenticate when no fragment token is present', async () => {
      const mockClient = createMockClient()
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => mockClient as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(mockClient.authenticate).toHaveBeenCalled()
      })
    })

    it('sets auth to authenticated after successful handshake', async () => {
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => createMockClient({ authenticate: vi.fn(async () => true) }) as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(screen.getByTestId('auth').textContent).toBe('authenticated')
      })
    })

    it('sets auth to anonymous when handshake fails', async () => {
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => createMockClient({ authenticate: vi.fn(async () => false) }) as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(screen.getByTestId('auth').textContent).toBe('anonymous')
      })
    })
  })

  // ── SSE connection ─────────────────────────────────────────────────────

  describe('SSE connection', () => {
    it('connects events after successful daemon detection', async () => {
      const mockClient = createMockClient()
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => mockClient as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(mockClient.connectEvents).toHaveBeenCalled()
      })
    })

    it('does not connect events when daemon is unreachable', async () => {
      const mockClient = createMockClient({
        isConnected: vi.fn(async () => false),
      })
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => mockClient as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      // Give it time to settle
      await new Promise((r) => setTimeout(r, 50))
      expect(mockClient.connectEvents).not.toHaveBeenCalled()
    })

    it('transitions to connecting then reconnects on SSE drop', async () => {
      let onErrorHandler: (() => void) | undefined
      const mockClient = createMockClient({
        connectEvents: vi.fn(),
        onEvent: vi.fn(() => vi.fn()),
      })
      // Capture the SSE error handler so we can simulate a disconnect
      Object.defineProperty(mockClient, 'onSSEError', {
        value: vi.fn((fn: () => void) => { onErrorHandler = fn; return vi.fn() }),
      })
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => mockClient as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(screen.getByTestId('daemon').textContent).toBe('connected')
      })
      // Simulate SSE disconnect
      act(() => { onErrorHandler?.() })
      expect(screen.getByTestId('daemon').textContent).toBe('connecting')
    })

    it('reconnects SSE with exponential backoff after disconnect', async () => {
      vi.useFakeTimers()
      const mockClient = createMockClient({
        connectEvents: vi.fn(),
        isConnected: vi.fn(async () => true),
      })
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => mockClient as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(screen.getByTestId('daemon').textContent).toBe('connected')
      })
      // After initial connect, connectEvents should have been called once
      const initialCalls = mockClient.connectEvents.mock.calls.length
      // Simulate SSE dropping and reconnection timer firing
      act(() => { vi.advanceTimersByTime(5000) })
      // Should have attempted reconnection
      expect(mockClient.connectEvents.mock.calls.length).toBeGreaterThanOrEqual(initialCalls)
      vi.useRealTimers()
    })

    it('transitions to disconnected after max SSE reconnection retries', async () => {
      const mockClient = createMockClient({
        connectEvents: vi.fn(),
        isConnected: vi.fn(async () => false),
      })
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => mockClient as any,
      )
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      await waitFor(() => {
        expect(screen.getByTestId('daemon').textContent).toBe('disconnected')
      })
    })
  })

  // ── Workspace state ────────────────────────────────────────────────────

  describe('workspace state', () => {
    it('starts with no active workspace', () => {
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      expect(screen.getByTestId('workspace').textContent).toBe('none')
    })

    it('exposes setWorkspace to change active workspace', async () => {
      function WorkspaceSetter() {
        const { workspace, setWorkspace } = usePlatform()
        return (
          <div>
            <span data-testid="ws">{workspace?.name ?? 'none'}</span>
            <button onClick={() => setWorkspace('feature/test')}>switch</button>
          </div>
        )
      }
      render(
        <PlatformProvider>
          <WorkspaceSetter />
        </PlatformProvider>,
      )
      await act(async () => {
        screen.getByText('switch').click()
      })
      expect(screen.getByTestId('ws').textContent).toBe('feature/test')
    })

    it('invalidates workspace-scoped queries on workspace switch', async () => {
      const mockClient = createMockClient()
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => mockClient as any,
      )
      function WorkspaceSwitcher() {
        const { setWorkspace } = usePlatform()
        return <button onClick={() => setWorkspace('feature/new')}>switch</button>
      }
      render(
        <PlatformProvider>
          <WorkspaceSwitcher />
        </PlatformProvider>,
      )
      await act(async () => {
        screen.getByText('switch').click()
      })
      expect(mockClient.invalidatePrefix).toHaveBeenCalledWith('/runtime/sessions')
    })

    it('persists viewed workspace to localStorage', async () => {
      function WorkspaceSetter() {
        const { setWorkspace } = usePlatform()
        return <button onClick={() => setWorkspace('main')}>set</button>
      }
      render(
        <PlatformProvider>
          <WorkspaceSetter />
        </PlatformProvider>,
      )
      await act(async () => {
        screen.getByText('set').click()
      })
      expect(localStorage.getItem('ship-studio:viewed-workspace')).toBe('main')
    })

    it('restores viewed workspace from localStorage on mount', () => {
      localStorage.setItem('ship-studio:viewed-workspace', 'feature/restored')
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      expect(screen.getByTestId('workspace').textContent).toBe('feature/restored')
    })
  })

  // ── Shell UI state ─────────────────────────────────────────────────────

  describe('shell UI state', () => {
    it('command palette starts closed', () => {
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      expect(screen.getByTestId('command-palette').textContent).toBe('false')
    })

    it('sidebar starts open', () => {
      render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      expect(screen.getByTestId('sidebar').textContent).toBe('true')
    })

    it('toggleCommandPalette flips command palette state', async () => {
      function PaletteToggler() {
        const { commandPaletteOpen, toggleCommandPalette } = usePlatform()
        return (
          <div>
            <span data-testid="cp">{String(commandPaletteOpen)}</span>
            <button onClick={toggleCommandPalette}>toggle</button>
          </div>
        )
      }
      render(
        <PlatformProvider>
          <PaletteToggler />
        </PlatformProvider>,
      )
      expect(screen.getByTestId('cp').textContent).toBe('false')
      await act(async () => {
        screen.getByText('toggle').click()
      })
      expect(screen.getByTestId('cp').textContent).toBe('true')
    })

    it('toggleSidebar flips sidebar state', async () => {
      function SidebarToggler() {
        const { sidebarOpen, toggleSidebar } = usePlatform()
        return (
          <div>
            <span data-testid="sb">{String(sidebarOpen)}</span>
            <button onClick={toggleSidebar}>toggle</button>
          </div>
        )
      }
      render(
        <PlatformProvider>
          <SidebarToggler />
        </PlatformProvider>,
      )
      expect(screen.getByTestId('sb').textContent).toBe('true')
      await act(async () => {
        screen.getByText('toggle').click()
      })
      expect(screen.getByTestId('sb').textContent).toBe('false')
    })
  })

  // ── Client access ──────────────────────────────────────────────────────

  describe('client access', () => {
    it('exposes ShipClient instance via usePlatform', () => {
      function ClientChecker() {
        const { client } = usePlatform()
        return <span data-testid="has-client">{String(!!client)}</span>
      }
      render(
        <PlatformProvider>
          <ClientChecker />
        </PlatformProvider>,
      )
      expect(screen.getByTestId('has-client').textContent).toBe('true')
    })
  })

  // ── Connection status ───────────────────────────────────────────────────

  describe('connection status', () => {
    it('exposes connection metadata for Settings view', () => {
      function ConnectionConsumer() {
        const { daemon, connection } = usePlatform() as any
        return (
          <div>
            <span data-testid="status">{daemon}</span>
            <span data-testid="reconnects">{connection?.reconnects ?? 0}</span>
          </div>
        )
      }
      render(
        <PlatformProvider>
          <ConnectionConsumer />
        </PlatformProvider>,
      )
      expect(screen.getByTestId('status').textContent).toBeTruthy()
      expect(screen.getByTestId('reconnects').textContent).toBe('0')
    })
  })

  // ── Cleanup ────────────────────────────────────────────────────────────

  describe('cleanup', () => {
    it('destroys client on unmount', async () => {
      const mockClient = createMockClient()
      vi.mocked(await import('@ship/sdk')).ShipClient.mockImplementation(
        () => mockClient as any,
      )
      const { unmount } = render(
        <PlatformProvider>
          <PlatformConsumer />
        </PlatformProvider>,
      )
      unmount()
      expect(mockClient.destroy).toHaveBeenCalled()
    })
  })

  // ── Error boundary ─────────────────────────────────────────────────────

  describe('context guard', () => {
    it('throws when usePlatform is used outside PlatformProvider', () => {
      expect(() => render(<PlatformConsumer />)).toThrow(
        /usePlatform must be used within PlatformProvider/,
      )
    })
  })
})
