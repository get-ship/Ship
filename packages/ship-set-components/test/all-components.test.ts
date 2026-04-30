// Smoke test for every shipped `<set-*>` component.
//
// Each port is registered via `registerAll()`, then mounted with a
// minimum viable attribute set, and asserted on three things:
//   1. The custom element registered (tag is in the registry).
//   2. The shadow root is attached and contains a recognizable part.
//   3. Setting a reflected attribute updates the host's DOM attribute
//      (proves the prop pipeline runs).
//
// Per-component deeper tests (slot content, conditional ARIA, complex
// derived state) live in dedicated test files where they earn their
// existence; this file is the regression net for "did we break a
// port."

import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { registerAll } from '../src/index'
import { createContainerRegistry, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

beforeAll(() => {
  // `registerAll` is idempotent; calling here ensures the test file
  // doesn't depend on the index.ts side-effect import order.
  registerAll()
})

interface PortCase {
  /** Tag suffix; the test mounts `<set-<name>>`. */
  tag: string
  /** A reflected attr to set + assert echoes back on the host. */
  attr: string
  /** Value to set for the reflect check. */
  value: string
  /** A part name (or class selector) that should be present in the shadow. */
  shadowPart: string
}

const ports: PortCase[] = [
  { tag: 'set-status-dot', attr: 'tone',  value: 'primary', shadowPart: '.dot' },
  { tag: 'set-spinner',    attr: 'size',  value: 'lg',      shadowPart: '.ring' },
  { tag: 'set-badge',      attr: 'tone',  value: 'success', shadowPart: '.root' },
  { tag: 'set-chip',       attr: 'tone',  value: 'primary', shadowPart: '.root' },
  { tag: 'set-card',       attr: 'padding', value: 'lg',    shadowPart: '.root' },
  { tag: 'set-link',       attr: 'tone',  value: 'primary', shadowPart: 'a' },
  { tag: 'set-divider',    attr: 'orientation', value: 'vertical', shadowPart: '.line' },
  { tag: 'set-skeleton',   attr: 'shape', value: 'circle',  shadowPart: '.sk' },
  { tag: 'set-stat',       attr: 'size',  value: 'lg',      shadowPart: '.root' },
  { tag: 'set-mark',       attr: 'tone',  value: 'success', shadowPart: 'mark' },
  { tag: 'set-kbd',        attr: 'size',  value: 'sm',      shadowPart: 'kbd' },
  { tag: 'set-avatar',     attr: 'size',  value: 'lg',      shadowPart: '.root' },
  { tag: 'set-progress',   attr: 'tone',  value: 'success', shadowPart: '.track' },
  { tag: 'set-button',     attr: 'tone',  value: 'primary', shadowPart: 'button' },
  { tag: 'set-tabs',       attr: 'variant', value: 'pill',  shadowPart: '.strip' },
  { tag: 'set-tab',        attr: 'name',  value: 'one',     shadowPart: 'button' },
  { tag: 'set-empty-state', attr: 'tone', value: 'muted',   shadowPart: '.title' },
  { tag: 'set-input',      attr: 'size',  value: 'lg',      shadowPart: 'input' },
]

describe.each(ports)('<$tag>', ({ tag, attr, value, shadowPart }) => {
  it('registers as a custom element', () => {
    expect(customElements.get(tag)).toBeDefined()
  })

  it('mounts with a shadow root containing the expected part', () => {
    const { el, shadow } = mount(reg, tag, { [attr]: value })
    expect(shadow).not.toBeNull()
    expect(shadow!.querySelector(shadowPart)).not.toBeNull()
    expect(el.getAttribute(attr)).toBe(value)
  })
})
