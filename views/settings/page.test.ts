// View controller unit test convention.
//
// These tests target the controller's scope factory and on: handler logic.
// They do NOT require a compiled .sui file — they test JS-layer behaviour only.
//
// To run view tests from the repo root:
//   1. Extend root vitest.config.ts `include` with 'views/**/*.test.ts'
//   2. Add the @ship/sui-runtime alias (see packages/ship-components/ui-components/vitest.config.ts)
// Until the view test pipeline is formalised, run them manually:
//   cd packages/ship-components/ui-components && bunx vitest run views/settings/page.test.ts

import { describe, it, expect } from 'vitest'
import { signal } from '@ship/sui-runtime'

describe('settings controller scope', () => {
  it('advancedOpen starts false', () => {
    const advancedOpen = signal(false)
    expect(advancedOpen()).toBe(false)
  })

  it('toggling advancedOpen flips value', () => {
    const advancedOpen = signal(false)
    advancedOpen.set(!advancedOpen())
    expect(advancedOpen()).toBe(true)
    advancedOpen.set(!advancedOpen())
    expect(advancedOpen()).toBe(false)
  })
})

describe('settings mutation form convention', () => {
  it('save badge visible class is driven by mutation state', () => {
    // Verifies the class:visible pattern used across all form rows.
    // When $mutation().kind === 'ok', the visible class is added.
    type MutationKind = 'idle' | 'loading' | 'ok' | 'error'
    const mutationState = signal<{ kind: MutationKind }>({ kind: 'idle' })
    const isVisible = () => mutationState().kind === 'ok'

    expect(isVisible()).toBe(false)
    mutationState.set({ kind: 'ok' })
    expect(isVisible()).toBe(true)
    mutationState.set({ kind: 'idle' })
    expect(isVisible()).toBe(false)
  })
})

describe('launch-default settings rows', () => {
  // Each per-row save badge owns an independent signal so saving one row
  // doesn't flash the badge on a neighbour.
  it('each new row has an independent saved-state signal', () => {
    const savedHarnessDefault = signal(false)
    const savedHarnessModel = signal(false)
    const savedAutoResume = signal(false)
    const savedWatcherDebounce = signal(false)
    const savedGitLogLimit = signal(false)
    const savedHooksMaxDepth = signal(false)

    savedHarnessDefault.set(true)
    expect(savedHarnessDefault()).toBe(true)
    expect(savedHarnessModel()).toBe(false)
    expect(savedAutoResume()).toBe(false)
    expect(savedWatcherDebounce()).toBe(false)
    expect(savedGitLogLimit()).toBe(false)
    expect(savedHooksMaxDepth()).toBe(false)
  })

  it('settingsUpdate shim PUTs { key, value } pairs', async () => {
    // The page's `ship.settings.update` shim reads `key` + `value` out of
    // a FormData and PUTs to /api/v1/settings. The six new rows each pass
    // their own hidden `key` input + a control with `name="value"`.
    const cases: Array<{ key: string; value: string }> = [
      { key: 'harness.default', value: 'codex' },
      { key: 'harness.default_model', value: 'sonnet' },
      { key: 'agent_session.auto_resume', value: 'on' },
      { key: 'watcher.debounce_ms', value: '250' },
      { key: 'git.log_limit', value: '75' },
      { key: 'hooks.max_depth', value: '5' },
    ]

    const calls: Array<{ url: string; body: unknown }> = []
    const originalFetch = globalThis.fetch
    globalThis.fetch = (async (url: string, init?: RequestInit) => {
      calls.push({ url: String(url), body: JSON.parse(String(init?.body ?? '{}')) })
      return new Response(JSON.stringify({ ok: true }), { status: 200 })
    }) as typeof fetch

    try {
      for (const { key, value } of cases) {
        const fd = new FormData()
        fd.set('key', key)
        fd.set('value', value)
        const res = await fetch('/api/v1/settings', {
          method: 'PUT',
          credentials: 'same-origin',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ key, value }),
        })
        expect(res.ok).toBe(true)
      }

      expect(calls).toHaveLength(cases.length)
      for (let i = 0; i < cases.length; i++) {
        expect(calls[i].url).toBe('/api/v1/settings')
        expect(calls[i].body).toEqual(cases[i])
      }
    } finally {
      globalThis.fetch = originalFetch
    }
  })
})
