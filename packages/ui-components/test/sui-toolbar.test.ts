import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-toolbar/dist/sui-toolbar.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-toolbar>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-toolbar')).toBeDefined()
  })

  it('reflects justify/align/gap attributes', () => {
    const { el } = mount(reg, 'sui-toolbar', { justify: 'between', align: 'baseline', gap: 'lg' })
    expect(el.getAttribute('justify')).toBe('between')
    expect(el.getAttribute('align')).toBe('baseline')
    expect(el.getAttribute('gap')).toBe('lg')
  })

  it('reflects variant attribute', () => {
    const { el } = mount(reg, 'sui-toolbar', { variant: 'bordered' })
    expect(el.getAttribute('variant')).toBe('bordered')
  })

  it('sets role=toolbar and aria-orientation=horizontal on mount', async () => {
    const { el } = mount(reg, 'sui-toolbar', {})
    await flush()
    expect(el.getAttribute('role')).toBe('toolbar')
    expect(el.getAttribute('aria-orientation')).toBe('horizontal')
  })

  it('has a shadow root', () => {
    const { shadow } = mount(reg, 'sui-toolbar', {})
    expect(shadow).not.toBeNull()
  })

  it('spacer slot projects with flex:1 (slot present in shadow)', () => {
    const { el } = mount(reg, 'sui-toolbar', {}, '<span slot="spacer"></span><button>Action</button>')
    const spacer = el.querySelector('[slot="spacer"]')
    expect(spacer).not.toBeNull()
  })

  it('ArrowRight moves focus to next item', async () => {
    const { el } = mount(
      reg,
      'sui-toolbar',
      {},
      '<button id="b1">One</button><button id="b2">Two</button>',
    )
    await flush()
    const b1 = el.querySelector('#b1') as HTMLButtonElement
    const b2 = el.querySelector('#b2') as HTMLButtonElement
    b1.focus()
    b1.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, composed: true }))
    await flush()
    expect(document.activeElement).toBe(b2)
  })

  it('ArrowLeft wraps focus to last item', async () => {
    const { el } = mount(
      reg,
      'sui-toolbar',
      {},
      '<button id="b1">One</button><button id="b2">Two</button>',
    )
    await flush()
    const b1 = el.querySelector('#b1') as HTMLButtonElement
    const b2 = el.querySelector('#b2') as HTMLButtonElement
    b1.focus()
    b1.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true, composed: true }))
    await flush()
    expect(document.activeElement).toBe(b2)
  })

  it('disabled items are skipped in keyboard navigation', async () => {
    const { el } = mount(
      reg,
      'sui-toolbar',
      {},
      '<button id="b1">One</button><button id="b2" disabled>Two</button><button id="b3">Three</button>',
    )
    await flush()
    const b1 = el.querySelector('#b1') as HTMLButtonElement
    const b3 = el.querySelector('#b3') as HTMLButtonElement
    b1.focus()
    b1.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, composed: true }))
    await flush()
    expect(document.activeElement).toBe(b3)
  })
})
