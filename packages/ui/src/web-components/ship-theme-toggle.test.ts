import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import './ship-theme-toggle'
import type { ShipThemeToggle } from './ship-theme-toggle'
import { cleanupTestDom } from './util'

describe('ship-theme-toggle', () => {
  beforeEach(() => {
    document.documentElement.classList.remove('dark')
    localStorage.clear()
  })
  afterEach(cleanupTestDom)

  it('toggles .dark on html root + persists to localStorage', async () => {
    document.body.insertAdjacentHTML('beforeend', '<ship-theme-toggle></ship-theme-toggle>')
    const el = document.body.lastElementChild as ShipThemeToggle
    await el.updateComplete
    const startDark = document.documentElement.classList.contains('dark')
    const btn = el.shadowRoot!.querySelector('button')!
    btn.click()
    expect(document.documentElement.classList.contains('dark')).toBe(!startDark)
    expect(localStorage.getItem('ship-theme')).toMatch(/^(light|dark)$/)
  })

  it('emits ship-theme-change', async () => {
    document.body.insertAdjacentHTML('beforeend', '<ship-theme-toggle></ship-theme-toggle>')
    const el = document.body.lastElementChild as ShipThemeToggle
    await el.updateComplete
    let detail: { theme: string } | null = null
    el.addEventListener('ship-theme-change', (e) => { detail = (e as CustomEvent).detail })
    el.shadowRoot!.querySelector('button')!.click()
    expect(detail).toBeTruthy()
  })
})
