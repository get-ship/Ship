import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import './ship-view-chrome'
import type { ShipViewChrome } from './ship-view-chrome'
import { cleanupTestDom } from './util'

function installShellChrome() {
  document.body.insertAdjacentHTML('beforeend', `
    <span id="breadcrumbs-title"></span>
    <span id="view-topbar-actions"></span>
    <span id="status-branch"></span>
    <span id="status-ab"></span>
    <span id="view-status-start"></span>
    <span id="view-status-end"></span>
    <span class="status-dot" id="status-dot"></span>
  `)
}

describe('ship-view-chrome', () => {
  beforeEach(installShellChrome)
  afterEach(cleanupTestDom)

  it('writes title/branch/ahead/behind into shell targets', async () => {
    document.body.insertAdjacentHTML('beforeend',
      '<ship-view-chrome title="Git" branch="main" ahead="2" behind="1"></ship-view-chrome>')
    const el = document.body.querySelector('ship-view-chrome') as ShipViewChrome
    await el.updateComplete
    await new Promise(r => queueMicrotask(() => r(null)))
    expect(document.getElementById('breadcrumbs-title')?.textContent).toBe('Git')
    expect(document.getElementById('status-branch')?.textContent).toBe('main')
    expect(document.getElementById('status-ab')?.textContent).toBe('↑2 ↓1')
  })

  it('moves slotted children into shell containers and restores on disconnect', async () => {
    document.body.insertAdjacentHTML('beforeend', `
      <ship-view-chrome branch="main">
        <button slot="actions" id="a1">Refresh</button>
        <span slot="status-end" id="s1">22 changed</span>
      </ship-view-chrome>
    `)
    const el = document.body.querySelector('ship-view-chrome') as ShipViewChrome
    await el.updateComplete
    await new Promise(r => queueMicrotask(() => r(null)))
    expect(document.getElementById('a1')?.parentElement?.id).toBe('view-topbar-actions')
    expect(document.getElementById('s1')?.parentElement?.id).toBe('view-status-end')
    el.remove()
    expect(document.getElementById('a1')).toBeNull()
    expect(document.getElementById('s1')).toBeNull()
    expect(document.getElementById('status-branch')?.textContent).toBe('')
  })

  it('no-ops gracefully when shell chrome containers are missing', async () => {
    document.getElementById('view-topbar-actions')?.remove()
    document.body.insertAdjacentHTML('beforeend', `
      <ship-view-chrome branch="main">
        <button slot="actions" id="orphan">x</button>
      </ship-view-chrome>
    `)
    const el = document.body.querySelector('ship-view-chrome') as ShipViewChrome
    await el.updateComplete
    await new Promise(r => queueMicrotask(() => r(null)))
    // Slot child is left where it was (inside the chrome element) when target is absent.
    expect(document.getElementById('orphan')).not.toBeNull()
    // Other targets still get updated.
    expect(document.getElementById('status-branch')?.textContent).toBe('main')
  })
})
