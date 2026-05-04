import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-table/sui-table.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

describe('<sui-table>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-table')).toBeDefined()
  })

  it('has a shadow root with .wrap', () => {
    const { shadow } = mount(reg, 'sui-table', {})
    expect(shadow!.querySelector('.wrap')).not.toBeNull()
  })

  it('slot mode renders thead/tbody in shadow DOM', () => {
    const { shadow } = mount(reg, 'sui-table', {})
    expect(shadow!.querySelector('thead')).not.toBeNull()
    expect(shadow!.querySelector('tbody')).not.toBeNull()
  })

  it('native mode renders wrap without inner table in shadow', () => {
    const { shadow } = mount(reg, 'sui-table', { variant: 'native' }, '<table><tbody><tr><td>A</td></tr></tbody></table>')
    expect(shadow!.querySelector('table')).toBeNull()
    expect(shadow!.querySelector('.wrap')).not.toBeNull()
  })

  it('reflects dense attribute', () => {
    const { el } = mount(reg, 'sui-table', { dense: true })
    expect(el.hasAttribute('dense')).toBe(true)
  })

  it('hoverable defaults true — not set to false initially', () => {
    const { el } = mount(reg, 'sui-table', {})
    // The runtime skips the initial reflect pass; attribute is absent by default.
    // The light-DOM CSS uses :not([hoverable='false']) so absent = hoverable enabled.
    expect(el.getAttribute('hoverable')).not.toBe('false')
  })

  it('reflects sticky-first-col attribute', () => {
    const { el } = mount(reg, 'sui-table', { 'sticky-first-col': true })
    expect(el.hasAttribute('sticky-first-col')).toBe(true)
  })

  it('reflects gutter attribute', () => {
    const { el } = mount(reg, 'sui-table', { gutter: true })
    expect(el.hasAttribute('gutter')).toBe(true)
  })

  it('reflects density attribute', () => {
    const { el } = mount(reg, 'sui-table', { density: 'compact' })
    expect(el.getAttribute('density')).toBe('compact')
  })

  it('sui-row-click fires when a data-row-id row td is clicked', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><thead><tr><th>Name</th></tr></thead><tbody><tr data-row-id="abc"><td>Alpha</td></tr></tbody></table>',
    )
    await flush()
    let detail: { id: string; index: number } | null = null
    el.addEventListener('sui-row-click', (e) => { detail = (e as CustomEvent).detail })
    ;(el.querySelector('td') as HTMLTableCellElement).click()
    expect(detail).not.toBeNull()
    expect(detail!.id).toBe('abc')
  })

  it('sui-row-click does not fire for sui-table-gutter-cell', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><tbody><tr data-row-id="abc"><td class="sui-table-gutter-cell"><input type="checkbox"></td><td>Name</td></tr></tbody></table>',
    )
    await flush()
    let fired = false
    el.addEventListener('sui-row-click', () => { fired = true })
    ;(el.querySelector('.sui-table-gutter-cell') as HTMLTableCellElement).click()
    expect(fired).toBe(false)
  })

  it('sui-row-click does not fire for rows without data-row-id', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><tbody><tr><td>No ID</td></tr></tbody></table>',
    )
    await flush()
    let fired = false
    el.addEventListener('sui-row-click', () => { fired = true })
    ;(el.querySelector('td') as HTMLTableCellElement).click()
    expect(fired).toBe(false)
  })

  it('enhances th[data-sort-key] with aria-sort, tabindex, role', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><thead><tr><th data-sort-key="name">Name</th></tr></thead><tbody><tr><td>Alice</td></tr></tbody></table>',
    )
    await flush()
    const th = el.querySelector('th[data-sort-key]') as HTMLTableCellElement
    expect(th.getAttribute('aria-sort')).toBe('none')
    expect(th.getAttribute('tabindex')).toBe('0')
    expect(th.getAttribute('role')).toBe('columnheader')
  })

  it('sui-sort fires and cycles direction on sortable th click', async () => {
    const { el } = mount(
      reg,
      'sui-table',
      { variant: 'native' },
      '<table><thead><tr><th data-sort-key="name">Name</th></tr></thead><tbody><tr><td>Alice</td></tr></tbody></table>',
    )
    await flush()
    const events: { key: string; direction: string }[] = []
    el.addEventListener('sui-sort', (e) => events.push((e as CustomEvent).detail))
    const th = el.querySelector('th[data-sort-key]') as HTMLTableCellElement
    th.click()
    expect(events[0]).toEqual({ key: 'name', direction: 'ascending' })
    th.click()
    expect(events[1]).toEqual({ key: 'name', direction: 'descending' })
    th.click()
    expect(events[2]).toEqual({ key: 'name', direction: 'none' })
  })

  it('slot mode has head slot in shadow thead', () => {
    const { shadow } = mount(reg, 'sui-table', {})
    const headSlot = shadow!.querySelector('slot[name="head"]')
    expect(headSlot).not.toBeNull()
    expect(headSlot!.closest('thead')).not.toBeNull()
  })
})
