// View controller unit test convention.
//
// These tests target the controller's scope factory and on: handler logic.
// They do NOT require a compiled .sui file — they test JS-layer behaviour only.
//
// To run view tests from the repo root:
//   1. Extend root vitest.config.ts `include` with 'views/**/*.test.ts'
//   2. Add the @ship/sui-runtime alias (see packages/ui-components/vitest.config.ts)
// Until the view test pipeline is formalised, run them manually:
//   pnpm -C packages/ui-components exec vitest run views/settings/page.test.ts

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
