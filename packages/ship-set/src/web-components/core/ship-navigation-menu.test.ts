import { afterEach, describe, expect, it } from 'vitest'
import './ship-navigation-menu'
import type { ShipNavigationItem, ShipNavigationMenu } from './ship-navigation-menu'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipNavigationMenu | ShipNavigationItem
  await el.updateComplete
  return el
}

describe('ship-navigation-item', () => {
  afterEach(cleanupTestDom)

  it('renders with button by default', async () => {
    const el = await mount(`<ship-navigation-item value="home">Home</ship-navigation-item>`) as ShipNavigationItem
    const btn = el.renderRoot.querySelector('button')
    expect(btn).toBeTruthy()
    expect(btn!.getAttribute('aria-current')).toBe('false')
  })

  it('renders as anchor when href is set', async () => {
    const el = await mount(`<ship-navigation-item value="docs" href="/docs">Docs</ship-navigation-item>`) as ShipNavigationItem
    const a = el.renderRoot.querySelector('a')
    expect(a).toBeTruthy()
    expect(a!.getAttribute('href')).toBe('/docs')
  })

  it('reflects active attribute and sets aria-current', async () => {
    const el = await mount(`<ship-navigation-item value="x" active>X</ship-navigation-item>`) as ShipNavigationItem
    expect(el.active).toBe(true)
    const btn = el.renderRoot.querySelector('button')
    expect(btn!.getAttribute('aria-current')).toBe('true')
  })
})

describe('ship-navigation-menu', () => {
  afterEach(cleanupTestDom)

  it('sets role="navigation"', async () => {
    const el = await mount(`
      <ship-navigation-menu value="a">
        <ship-navigation-item value="a">A</ship-navigation-item>
        <ship-navigation-item value="b">B</ship-navigation-item>
      </ship-navigation-menu>
    `) as ShipNavigationMenu
    expect(el.getAttribute('role')).toBe('navigation')
  })

  it('syncs active state to items from value', async () => {
    const el = await mount(`
      <ship-navigation-menu value="b">
        <ship-navigation-item value="a">A</ship-navigation-item>
        <ship-navigation-item value="b">B</ship-navigation-item>
      </ship-navigation-menu>
    `) as ShipNavigationMenu
    await el.updateComplete
    const items = el.querySelectorAll('ship-navigation-item') as NodeListOf<ShipNavigationItem>
    expect(items[0].active).toBe(false)
    expect(items[1].active).toBe(true)
  })

  it('fires ship-change on item click', async () => {
    const el = await mount(`
      <ship-navigation-menu value="a">
        <ship-navigation-item value="a">A</ship-navigation-item>
        <ship-navigation-item value="b">B</ship-navigation-item>
      </ship-navigation-menu>
    `) as ShipNavigationMenu
    const items = el.querySelectorAll('ship-navigation-item') as NodeListOf<ShipNavigationItem>
    await items[1].updateComplete

    let detail: { value: string } | null = null
    el.addEventListener('ship-change', (e) => { detail = (e as CustomEvent).detail })

    items[1].renderRoot.querySelector('button')!.click()
    expect(detail).toEqual({ value: 'b' })
    expect(el.value).toBe('b')
  })
})
