import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-step-segments/sui-step-segments.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const getRow = (el: HTMLElement) => shadow(el).querySelector('.row') as HTMLElement | null
const getSegs = (el: HTMLElement) => Array.from(shadow(el).querySelectorAll('.seg'))

describe('<sui-step-segments>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-step-segments')).toBeDefined()
  })

  it('renders a shadow root containing a .row element', () => {
    const { el } = mount(reg, 'sui-step-segments')
    expect(shadow(el)).not.toBeNull()
    expect(getRow(el)).not.toBeNull()
  })

  it('renders no segments when values is omitted', async () => {
    const { el } = mount(reg, 'sui-step-segments')
    await flush()
    expect(getSegs(el).length).toBe(0)
  })

  it('renders one segment per comma-separated value', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done,failed,running,pending' })
    await flush()
    expect(getSegs(el).length).toBe(4)
  })

  it('sets data-state=done on a done segment', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done' })
    await flush()
    expect(getSegs(el)[0].getAttribute('data-state')).toBe('done')
  })

  it('sets data-state=failed on a failed segment', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'failed' })
    await flush()
    expect(getSegs(el)[0].getAttribute('data-state')).toBe('failed')
  })

  it('sets data-state=running on a running segment', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'running' })
    await flush()
    expect(getSegs(el)[0].getAttribute('data-state')).toBe('running')
  })

  it('sets data-state=pending on a pending segment', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'pending' })
    await flush()
    expect(getSegs(el)[0].getAttribute('data-state')).toBe('pending')
  })

  it('coerces unknown tokens to pending', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done,wat,banana' })
    await flush()
    const all = getSegs(el)
    expect(all.length).toBe(3)
    expect(all[1].getAttribute('data-state')).toBe('pending')
    expect(all[2].getAttribute('data-state')).toBe('pending')
  })

  it('trims whitespace and skips empty entries', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: ' done , , failed ' })
    await flush()
    const all = getSegs(el)
    expect(all.length).toBe(2)
    expect(all[0].getAttribute('data-state')).toBe('done')
    expect(all[1].getAttribute('data-state')).toBe('failed')
  })

  it('sets role=img on the row when segments are present', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done' })
    await flush()
    expect(getRow(el)?.getAttribute('role')).toBe('img')
  })

  it('sets aria-label to plural count', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done,failed' })
    await flush()
    expect(getRow(el)?.getAttribute('aria-label')).toBe('2 step outcomes')
  })

  it('sets aria-label to singular count', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done' })
    await flush()
    expect(getRow(el)?.getAttribute('aria-label')).toBe('1 step outcome')
  })

  it('clears role and aria-label when values becomes empty', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done,failed' })
    await flush()
    el.setAttribute('values', '')
    await flush()
    expect(getRow(el)?.hasAttribute('role')).toBe(false)
    expect(getRow(el)?.hasAttribute('aria-label')).toBe(false)
  })

  it('each segment has part=seg', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done,running,failed' })
    await flush()
    for (const seg of getSegs(el)) {
      expect(seg.getAttribute('part')).toBe('seg')
    }
  })

  it('the row has part=row', async () => {
    const { el } = mount(reg, 'sui-step-segments')
    expect(getRow(el)?.getAttribute('part')).toBe('row')
  })

  it('reflects the values attribute on the host', () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done,failed' })
    expect(el.getAttribute('values')).toBe('done,failed')
  })

  it('reflects the size attribute on the host', () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done', size: 'md' })
    expect(el.getAttribute('size')).toBe('md')
  })

  it('re-renders segments when the values attribute changes', async () => {
    const { el } = mount(reg, 'sui-step-segments', { values: 'done' })
    await flush()
    expect(getSegs(el).length).toBe(1)
    el.setAttribute('values', 'done,failed,running')
    await flush()
    expect(getSegs(el).length).toBe(3)
    expect(getSegs(el)[2].getAttribute('data-state')).toBe('running')
  })
})
