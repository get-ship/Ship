// openSpace() — single seam for "open a space as its own process+window".
//
// The function branches on Tauri-vs-browser via the global that Tauri 2 plants
// when `withGlobalTauri: true` (window.__TAURI__ / __TAURI_INTERNALS__).
// We don't ship @tauri-apps/api in the shell bundle, so the test stubs the
// globals the same way a real Tauri webview exposes them.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { openSpace } from '../src/chrome/open-space'

function withTauri(invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>) {
  // Tauri 2 with `withGlobalTauri: true` puts `invoke` under __TAURI__.core;
  // the internals namespace is the lower-level fallback the runtime uses.
  ;(window as unknown as { __TAURI__?: unknown }).__TAURI__ = { core: { invoke } }
  delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__
}

function clearTauri() {
  delete (window as unknown as { __TAURI__?: unknown }).__TAURI__
  delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__
}

describe('openSpace', () => {
  const originalOpen = window.open

  beforeEach(() => {
    clearTauri()
  })
  afterEach(() => {
    clearTauri()
    window.open = originalOpen
  })

  it('invokes the Tauri open_space command when the Tauri global is present', async () => {
    const invoke = vi.fn().mockResolvedValue(undefined)
    withTauri(invoke)

    const transport = await openSpace('daily-digest')

    expect(transport).toBe('tauri')
    expect(invoke).toHaveBeenCalledTimes(1)
    expect(invoke).toHaveBeenCalledWith('open_space', { appId: 'daily-digest' })
  })

  it('falls back to __TAURI_INTERNALS__ when __TAURI__ is absent', async () => {
    const invoke = vi.fn().mockResolvedValue(undefined)
    ;(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = { invoke }

    const transport = await openSpace('demo')

    expect(transport).toBe('tauri')
    expect(invoke).toHaveBeenCalledWith('open_space', { appId: 'demo' })
  })

  it('opens /space/<id> in a new tab when no Tauri global is present', async () => {
    const opened = vi.fn().mockReturnValue(null)
    window.open = opened as unknown as typeof window.open

    const transport = await openSpace('my-project')

    expect(transport).toBe('browser')
    expect(opened).toHaveBeenCalledWith('/space/my-project', '_blank', 'noopener,noreferrer')
  })

  it('percent-encodes the id when falling back to the browser', async () => {
    const opened = vi.fn().mockReturnValue(null)
    window.open = opened as unknown as typeof window.open

    await openSpace('a/b c')

    expect(opened).toHaveBeenCalledWith('/space/a%2Fb%20c', '_blank', 'noopener,noreferrer')
  })

  it('is a no-op (browser transport) when id is empty', async () => {
    const invoke = vi.fn().mockResolvedValue(undefined)
    withTauri(invoke)
    const opened = vi.fn().mockReturnValue(null)
    window.open = opened as unknown as typeof window.open

    const transport = await openSpace('')

    expect(transport).toBe('browser')
    expect(invoke).not.toHaveBeenCalled()
    expect(opened).not.toHaveBeenCalled()
  })

  it('propagates a Tauri invoke rejection (the caller decides what to do)', async () => {
    const invoke = vi.fn().mockRejectedValue(new Error('spawn failed: ENOENT'))
    withTauri(invoke)

    await expect(openSpace('x')).rejects.toThrow('spawn failed: ENOENT')
  })
})
