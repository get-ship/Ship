// Smoke test for `<set-button>` registration + the declarative
// `<slot reflect-empty="data-icon-only">` wiring path.
//
// The reflect-empty content-toggle behavior depends on shadow-DOM
// slotting, which JSDOM only partially implements: `assignedNodes`
// returns `[]` regardless of light-DOM content because no real
// distribution algorithm runs. So the unit tests in
// `packages/ui-runtime/test/slot-empty.test.ts` cover the
// `bindSlotEmpty` helper directly with stubbed `assignedNodes`,
// and the in-browser probe at `.artifacts/set-button-probe.html`
// exercises the end-to-end shadow-tree behavior.
//
// What this file verifies:
//   - the custom element registers
//   - the shadow root is attached and the rendered <button> exists
//   - prop-driven attribute pipelines still work post-port (aria-disabled)
//   - the bindSlotEmpty wirer ran (the host carries data-icon-only,
//     since JSDOM reports the default slot as empty)

import { afterEach, describe, expect, it } from 'vitest'
import '../src/set-button/set-button.compiled.js'

const containers: HTMLElement[] = []
const mkContainer = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

describe('<set-button>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('set-button')).toBeDefined()
  })

  it('attaches a shadow root with the rendered <button>', () => {
    const container = mkContainer()
    container.innerHTML = '<set-button>Save</set-button>'
    const el = container.querySelector('set-button')!
    expect(el.shadowRoot).not.toBeNull()
    expect(el.shadowRoot!.querySelector('button')).not.toBeNull()
  })

  it('reflects aria-disabled when disabled is set', () => {
    const container = mkContainer()
    container.innerHTML = '<set-button disabled>Save</set-button>'
    const el = container.querySelector('set-button')!
    const btn = el.shadowRoot!.querySelector('button')!
    expect(btn.getAttribute('aria-disabled')).toBe('true')
  })

  it('runs the bindSlotEmpty wirer (host carries data-icon-only when JSDOM reports empty slot)', () => {
    // JSDOM does not distribute light DOM into the default slot, so
    // assignedNodes returns []. The reflect-empty wirer therefore sets
    // data-icon-only — which proves the helper is wired up by the
    // component's compiled setup, even though full slotting behavior
    // requires a real browser to verify (see .artifacts probe).
    const container = mkContainer()
    container.innerHTML =
      '<set-button><span slot="leading">⚙</span></set-button>'
    const el = container.querySelector('set-button')!
    expect(el.hasAttribute('data-icon-only')).toBe(true)
  })
})
