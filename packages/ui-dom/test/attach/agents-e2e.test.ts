// Phase E-2 acceptance criterion 6: agents view end-to-end.
//
// Mounts the SSR'd loading-state DOM, attaches a synthetic compiled
// module shaped like the agents view, mocks the daemon's fragment-
// render endpoint with the HTML it would actually return for the ok
// branch, flips the `agents` envelope to `ok`, and asserts that:
//   * the swap fires
//   * the empty-state DOM is mounted (has `data-test="empty"`)
//   * a leaf binding (the `on:click` New Agent button) is wired and
//     fires `navigate('/agents/create')` when clicked.

import { afterEach, describe, expect, it, vi } from 'vitest'
import type { LiveSignal } from '@ship/ui-runtime'
import { attach } from '../../src/attach'
import { clearContainers, flush, mountSsr } from './helpers'

afterEach(() => {
  clearContainers()
  vi.restoreAllMocks()
})

describe('attach/agents end-to-end (Phase E-2)', () => {
  it('flips loading → ok, swaps to empty-state, and the button works', async () => {
    // Loading-state SSR shape: outer if has no taken branch (kind ===
    // 'loading'), no else, so it's a placeholder template.
    const loadingHtml = `<div class="agents-view"><sui-if key="b0" hidden></sui-if></div>`
    const root = mountSsr(loadingHtml, {
      agents: { kind: 'loading' },
    })

    let navigateCalls: string[] = []
    let agents: LiveSignal<{ items: unknown[] }> | undefined

    // Synthetic compiled-module shape matching what `compile()` would
    // emit for the agents view's outer if + on:click handler.
    const mod = {
      default: {
        setup(ctx: import('../../src/attach').SetupContext): Record<string, unknown> {
          agents = ctx.scope!.agents as unknown as LiveSignal<{ items: unknown[] }>
          // Derived signals + handlers the on:click expression reads:
          return {
            newAgent: () => ctx.navigate?.('/agents/create'),
          }
        },
      },
      bindings: [
        {
          key: 'b0',
          kind: 'if' as const,
          expr: "agents().kind === 'ok'",
          deps: ['agents'],
        },
        {
          key: 'b1',
          kind: 'on:click' as const,
          expr: 'newAgent',
          deps: ['newAgent'],
        },
      ],
    }

    // The fragment HTML the daemon would return for the ok-branch
    // swap. Carries the empty-state element + the New-agent button
    // with its on:click marker.
    const okFragment =
      '<ship-empty-state data-test="empty"><span slot="title">No agents yet</span></ship-empty-state>' +
      '<button data-test="new-agent" on:click="b1">New agent</button>'

    const fetchMock = vi.fn().mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => ({ html: okFragment }),
    })
    vi.stubGlobal('fetch', fetchMock)

    const dispose = attach(root, {
      module: mod,
      viewName: 'agents',
      navigate: (p: string) => {
        navigateCalls.push(p)
      },
    })

    // Loading state: nothing rendered inside the boundary.
    expect(root.querySelector('[data-test="empty"]')).toBeNull()
    expect(fetchMock).not.toHaveBeenCalled()

    // Flip the data signal — simulates an SSE patch arriving.
    agents!.set({ kind: 'ok', value: { items: [] } })
    await flush()
    await flush()

    expect(fetchMock).toHaveBeenCalledTimes(1)
    const [url, init] = fetchMock.mock.calls[0]!
    expect(url).toBe('/_ship-set/render-fragment/agents')
    const body = JSON.parse((init as RequestInit).body as string) as {
      init: Record<string, unknown>
      key: string
    }
    expect(body.key).toBe('b0')
    expect(body.init.agents).toEqual({ kind: 'ok', value: { items: [] } })

    // Empty-state DOM is mounted.
    const empty = root.querySelector('[data-test="empty"]')
    expect(empty).not.toBeNull()

    // The on:click binding inside the freshly mounted subtree is wired
    // — clicking the button invokes `newAgent`, which calls navigate.
    const button = root.querySelector(
      'button[data-test="new-agent"]',
    ) as HTMLButtonElement | null
    expect(button).not.toBeNull()
    button!.click()
    expect(navigateCalls).toEqual(['/agents/create'])

    dispose()
  })
})
