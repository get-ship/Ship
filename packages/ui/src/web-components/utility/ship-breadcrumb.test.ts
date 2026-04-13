import { afterEach, describe, expect, it } from 'vitest'
import './ship-breadcrumb'
import type { ShipBreadcrumb, ShipBreadcrumbItem } from './ship-breadcrumb'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipBreadcrumb
  await el.updateComplete
  // One extra microtask cycle to flush the MutationObserver setup render.
  await new Promise(r => setTimeout(r, 0))
  await el.updateComplete
  return el
}

describe('ship-breadcrumb', () => {
  afterEach(cleanupTestDom)

  it('renders N-1 separators for N items', async () => {
    const el = await mount(`
      <ship-breadcrumb>
        <ship-breadcrumb-item href="/">Home</ship-breadcrumb-item>
        <ship-breadcrumb-item href="/a">A</ship-breadcrumb-item>
        <ship-breadcrumb-item active>B</ship-breadcrumb-item>
      </ship-breadcrumb>
    `)
    const seps = el.shadowRoot!.querySelectorAll('.sep')
    expect(seps.length).toBe(2)
  })

  it('breadcrumb-item with href renders <a>', async () => {
    document.body.insertAdjacentHTML('beforeend',
      '<ship-breadcrumb-item href="/x">X</ship-breadcrumb-item>')
    const item = document.body.lastElementChild as ShipBreadcrumbItem
    await item.updateComplete
    expect(item.shadowRoot!.querySelector('a')!.getAttribute('href')).toBe('/x')
  })

  it('active item renders label with aria-current=page', async () => {
    document.body.insertAdjacentHTML('beforeend',
      '<ship-breadcrumb-item active href="/z">Z</ship-breadcrumb-item>')
    const item = document.body.lastElementChild as ShipBreadcrumbItem
    await item.updateComplete
    const label = item.shadowRoot!.querySelector('.label')!
    expect(label.getAttribute('aria-current')).toBe('page')
  })
})
