// P0-3 — HMR recovery must not blank the app on a mid-edit failure.
//
// Before the fix, ANY failure during `remountView` (fetch, import, or hydrate)
// fell into a single catch that did `window.location.reload()` — discarding all
// signal state and hiding the real error. These tests drive `remountView`'s
// injectable IO to reproduce each failure mode and assert the fix: the real
// error is surfaced (a dev overlay + console), the LAST-GOOD mount is kept
// (`remount` is not called on a pre-hydrate failure), and the overlay clears on
// the next good build — which recovers IN PLACE (remount called), the direct
// proof that recovery needs no full reload. `remountView` no longer calls
// `window.location.reload()` at all (jsdom makes it unspyable anyway); these
// in-place assertions are what guard against a reload regression.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { remountView, type HmrDeps, type MountModule, type RemountIo } from '../src/hmr'

const OVERLAY_ID = 'sui-hmr-error'
const overlay = () => document.getElementById(OVERLAY_ID)

function deps(remount = vi.fn().mockResolvedValue(undefined)): HmrDeps & { remount: ReturnType<typeof vi.fn> } {
  return { getCurrentView: () => 'foo', remount }
}

const okModule: MountModule = { default: { setup: () => ({}) } }

describe('HMR remount failure handling (P0-3)', () => {
  beforeEach(() => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    vi.spyOn(console, 'info').mockImplementation(() => {})
  })
  afterEach(() => {
    overlay()?.remove()
    vi.restoreAllMocks()
  })

  it('fetch failure keeps the last-good mount, surfaces the error, and does not reload', async () => {
    const d = deps()
    const io: RemountIo = {
      fetchSource: () => Promise.reject(new Error('network down')),
      evalModule: () => Promise.reject(new Error('unreachable')),
    }
    await remountView(d, 'foo', '1', io)

    expect(d.remount).not.toHaveBeenCalled() // last-good view untouched
    expect(overlay()?.dataset.phase).toBe('fetch')
    expect(overlay()?.textContent).toContain('network down')
  })

  it('import/compile failure is classified as `import`, keeps last-good, no reload', async () => {
    const d = deps()
    const io: RemountIo = {
      fetchSource: () => Promise.resolve('bad source'),
      evalModule: () => Promise.reject(new SyntaxError('Unexpected token ;')),
    }
    await remountView(d, 'foo', '1', io)

    expect(d.remount).not.toHaveBeenCalled()
    expect(overlay()?.dataset.phase).toBe('import')
    expect(overlay()?.textContent).toContain('Unexpected token')
  })

  it('a successful remount clears a prior error overlay and does not reload', async () => {
    const d = deps()
    // First, a failed build paints the overlay.
    await remountView(
      d,
      'foo',
      '1',
      { fetchSource: () => Promise.reject(new Error('boom')), evalModule: () => Promise.reject(new Error()) },
    )
    expect(overlay()).not.toBeNull()

    // Then the author fixes the typo → the next build succeeds and recovers
    // IN PLACE (remount called) with the overlay cleared — no reload.
    await remountView(
      d,
      'foo',
      '2',
      { fetchSource: () => Promise.resolve('good'), evalModule: () => Promise.resolve(okModule) },
    )
    expect(d.remount).toHaveBeenCalledWith(okModule)
    expect(overlay()).toBeNull()
  })

  it('a hydrate failure surfaces the error (phase `hydrate`) and still does not reload', async () => {
    const d = deps(vi.fn().mockRejectedValue(new Error('hydrate boom')))
    const io: RemountIo = {
      fetchSource: () => Promise.resolve('good'),
      evalModule: () => Promise.resolve(okModule),
    }
    await remountView(d, 'foo', '1', io)

    expect(d.remount).toHaveBeenCalled() // remount was attempted
    expect(overlay()?.dataset.phase).toBe('hydrate')
    expect(overlay()?.textContent).toContain('hydrate boom')
  })
})
