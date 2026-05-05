import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-pagination/dist/sui-pagination.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const prevBtn = (el: HTMLElement) =>
  shadow(el).querySelector<HTMLButtonElement>('[part="prev"]')!
const nextBtn = (el: HTMLElement) =>
  shadow(el).querySelector<HTMLButtonElement>('[part="next"]')!
const infoEl = (el: HTMLElement) =>
  shadow(el).querySelector<HTMLElement>('[part="info"]')!

describe('<sui-pagination>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-pagination')).toBeDefined()
  })

  it('renders a nav with aria-label="Pagination"', () => {
    const { el } = mount(reg, 'sui-pagination')
    const nav = shadow(el).querySelector('nav')!
    expect(nav.getAttribute('aria-label')).toBe('Pagination')
  })

  it('renders prev and next buttons', () => {
    const { el } = mount(reg, 'sui-pagination')
    expect(prevBtn(el)).not.toBeNull()
    expect(nextBtn(el)).not.toBeNull()
  })

  it('prev button is disabled on page 1', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: '1', total: '100' })
    await flush()
    expect(prevBtn(el).disabled).toBe(true)
    expect(prevBtn(el).getAttribute('aria-disabled')).toBe('true')
  })

  it('next button is disabled on the last page', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: '4', total: '100', 'per-page': '25' })
    await flush()
    expect(nextBtn(el).disabled).toBe(true)
    expect(nextBtn(el).getAttribute('aria-disabled')).toBe('true')
  })

  it('both buttons enabled on a middle page', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: '2', total: '100', 'per-page': '25' })
    await flush()
    expect(prevBtn(el).disabled).toBe(false)
    expect(nextBtn(el).disabled).toBe(false)
  })

  it('shows page info text', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: '2', total: '100', 'per-page': '25' })
    await flush()
    const info = infoEl(el).textContent!
    expect(info).toContain('Page 2')
    expect(info).toContain('of 4')
    expect(info).toContain('26')
    expect(info).toContain('50')
    expect(info).toContain('100')
  })

  it('shows page info with no total', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: '1' })
    await flush()
    const info = infoEl(el).textContent!
    expect(info).toContain('Page 1 of 1')
  })

  it('page prop reflects', () => {
    const { el } = mount(reg, 'sui-pagination', { page: '3' })
    expect(el.getAttribute('page')).toBe('3')
  })

  it('per-page prop reflects', () => {
    const { el } = mount(reg, 'sui-pagination', { 'per-page': '10' })
    expect(el.getAttribute('per-page')).toBe('10')
  })

  it('clicking next fires sui-page with next page number', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: '1', total: '100', 'per-page': '25' })
    await flush()

    const events: CustomEvent[] = []
    el.addEventListener('sui-page', (e) => events.push(e as CustomEvent))

    nextBtn(el).click()
    await flush()

    expect(events.length).toBe(1)
    expect(events[0].detail).toEqual({ page: 2 })
  })

  it('clicking prev fires sui-page with previous page number', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: '3', total: '100', 'per-page': '25' })
    await flush()

    const events: CustomEvent[] = []
    el.addEventListener('sui-page', (e) => events.push(e as CustomEvent))

    prevBtn(el).click()
    await flush()

    expect(events.length).toBe(1)
    expect(events[0].detail).toEqual({ page: 2 })
  })

  it('clicking next updates the page attribute', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: '1', total: '50', 'per-page': '25' })
    await flush()

    nextBtn(el).click()
    await flush()

    expect(el.getAttribute('page')).toBe('2')
  })

  it('clicking prev on page 1 does not fire event', async () => {
    const { el } = mount(reg, 'sui-pagination', { page: '1', total: '100' })
    await flush()

    const events: CustomEvent[] = []
    el.addEventListener('sui-page', (e) => events.push(e as CustomEvent))
    prevBtn(el).click()
    await flush()

    expect(events.length).toBe(0)
  })

  it('sui-page event bubbles and is composed', async () => {
    const { el, container } = mount(reg, 'sui-pagination', { page: '1', total: '100' })
    await flush()

    const events: Event[] = []
    container.addEventListener('sui-page', (e) => events.push(e))
    nextBtn(el).click()
    await flush()

    expect(events.length).toBe(1)
    expect((events[0] as CustomEvent).bubbles).toBe(true)
  })

  it('prev/next buttons have aria-labels', () => {
    const { el } = mount(reg, 'sui-pagination')
    expect(prevBtn(el).getAttribute('aria-label')).toBe('Previous page')
    expect(nextBtn(el).getAttribute('aria-label')).toBe('Next page')
  })
})
