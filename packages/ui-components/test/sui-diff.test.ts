import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-diff/dist/sui-diff.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-diff>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-diff')).toBeDefined()
  })

  it('computes LCS diff from from/to props', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'a\nb\nc', to: 'a\nB\nc' })
    await flush()
    expect(shadow(el).querySelectorAll('.line.del').length).toBe(1)
    expect(shadow(el).querySelectorAll('.line.add').length).toBe(1)
    expect(shadow(el).querySelectorAll('.line.ctx').length).toBe(2)
  })

  it('parses raw unified-diff value', async () => {
    const { el } = mount(reg, 'sui-diff', { value: '@@ -1,2 +1,2 @@\n-old\n+new\n ctx' })
    await flush()
    expect(shadow(el).querySelector('.line.hunk')).not.toBeNull()
    expect(shadow(el).querySelector('.line.add')?.textContent).toContain('new')
    expect(shadow(el).querySelector('.line.del')?.textContent).toContain('old')
  })

  it('value prop takes precedence over from/to', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'x', to: 'y', value: '+only-this' })
    await flush()
    expect(shadow(el).querySelectorAll('.line').length).toBe(1)
    expect(shadow(el).querySelector('.line.add')).not.toBeNull()
  })

  it('add lines have + sign', async () => {
    const { el } = mount(reg, 'sui-diff', { from: '', to: 'new' })
    await flush()
    const addLine = shadow(el).querySelector('.line.add')!
    expect(addLine.querySelector('.sign')?.textContent).toBe('+')
  })

  it('del lines have - sign', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'old', to: '' })
    await flush()
    const delLine = shadow(el).querySelector('.line.del')!
    expect(delLine.querySelector('.sign')?.textContent).toBe('-')
  })

  it('hunk lines have @ sign', async () => {
    const { el } = mount(reg, 'sui-diff', { value: '@@ -1 +1 @@\n ctx' })
    await flush()
    const hunkLine = shadow(el).querySelector('.line.hunk')!
    expect(hunkLine.querySelector('.sign')?.textContent).toBe('@')
  })

  it('reacts to prop changes', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'x', to: 'y' })
    await flush()
    expect(shadow(el).querySelectorAll('.line').length).toBe(2)
    el.setAttribute('from', 'same')
    el.setAttribute('to', 'same')
    await flush()
    expect(shadow(el).querySelectorAll('.line.ctx').length).toBe(1)
    expect(shadow(el).querySelectorAll('.line.add').length).toBe(0)
  })

  it('empty props render no lines', async () => {
    const { el } = mount(reg, 'sui-diff')
    await flush()
    expect(shadow(el).querySelectorAll('.line').length).toBe(0)
  })

  it('host has role=region', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'a', to: 'b' })
    await flush()
    expect(el.getAttribute('role')).toBe('region')
  })

  it('host has aria-label="diff"', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'a', to: 'b' })
    await flush()
    expect(el.getAttribute('aria-label')).toBe('diff')
  })

  it('each line has part=line', async () => {
    const { el } = mount(reg, 'sui-diff', { from: 'x', to: 'y' })
    await flush()
    for (const line of shadow(el).querySelectorAll('.line')) {
      expect(line.getAttribute('part')).toBe('line')
    }
  })
})
