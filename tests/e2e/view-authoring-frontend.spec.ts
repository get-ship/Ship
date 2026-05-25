/**
 * View-authoring frontend e2e — covers the dev-panel surfaces wired in
 * `apps/shell/src/chrome/edit-mode.ts`, the new view-authoring island
 * tabs, the shared component picker, and the annotation pin overlay.
 *
 * Strategy: drive shell's vite dev server through Playwright. Each
 * spec scopes its work to a sample view (`/agents` is always
 * registered) so we don't depend on a scaffold flow that may not yet
 * exist server-side. Edit-mode is per-view, persisted in localStorage,
 * so we reset that storage in `beforeEach` to isolate cases.
 *
 * Mocked dependencies (initViewMocks in main.ts):
 *   - window.__viewMcp.readSource / writeSource (sample .sui source)
 *   - window.__viewMcp.fireAnnotation (synchronous annotation event)
 *   - the 5s background annotation timer (left running, not asserted)
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

const MOD = process.platform === 'darwin' ? 'Meta' : 'Control'

async function clearEditModeStorage(page: import('@playwright/test').Page): Promise<void> {
  await page.evaluate(() => {
    for (let i = localStorage.length - 1; i >= 0; i--) {
      const k = localStorage.key(i)
      if (k && k.startsWith('ship.edit-mode.')) localStorage.removeItem(k)
    }
  })
}

async function waitForChrome(page: import('@playwright/test').Page): Promise<void> {
  await page.waitForFunction(
    () => {
      const dock = document.getElementById('ship-panel-dock') as
        (HTMLElement & { toggle?: unknown; tabs?: unknown }) | null
      return Boolean(dock && typeof dock.toggle === 'function' && Array.isArray(dock.tabs))
    },
    null,
    { timeout: 10_000 },
  )
}

async function enterEditMode(page: import('@playwright/test').Page): Promise<void> {
  await page.keyboard.press(`${MOD}+Shift+E`)
  await expect.poll(
    () => page.evaluate(() => document.getElementById('ship-panel-dock')?.hasAttribute('edit-mode-on')),
    { timeout: 5000 },
  ).toBe(true)
}

test.describe('view-authoring frontend', () => {
  skipUnlessShell2()

  test.beforeEach(async ({ page }) => {
    await page.goto(`${SHELL2_URL}/agents`, { waitUntil: 'load' })
    await waitForChrome(page)
    await clearEditModeStorage(page)
  })

  test('Ctrl+Shift+E toggles edit-mode → 5 view-authoring tabs + dock opens', async ({ page }) => {
    await enterEditMode(page)

    // Dock body open.
    await expect.poll(
      () => page.evaluate(() => document.body.classList.contains('dock-open')),
    ).toBe(true)

    // VA tab set rendered.
    const tabs = await page.evaluate(() => {
      const dock = document.getElementById('ship-panel-dock')!
      return Array.from(dock.shadowRoot!.querySelectorAll('.tab')).map((t) => (t as HTMLElement).dataset.tab ?? '')
    })
    expect(tabs).toEqual(['va-source', 'va-components', 'va-activity', 'va-compile', 'terminal'])

    // Edit-mode pill is visible + ON.
    const pillState = await page.evaluate(() => {
      const dock = document.getElementById('ship-panel-dock')!
      const pill = dock.shadowRoot!.querySelector('.edit-pill') as HTMLElement | null
      return { present: Boolean(pill), checked: pill?.getAttribute('aria-checked') }
    })
    expect(pillState.present).toBe(true)
    expect(pillState.checked).toBe('true')
  })

  test('toggling off collapses the dock surface; toggling back restores tabs', async ({ page }) => {
    await enterEditMode(page)
    await page.keyboard.press(`${MOD}+Shift+E`)
    await expect.poll(
      () => page.evaluate(() => document.getElementById('ship-panel-dock')?.hasAttribute('edit-mode-on')),
    ).toBe(false)

    // Default dock tab set returns.
    const defaults = await page.evaluate(() => {
      const dock = document.getElementById('ship-panel-dock')!
      return Array.from(dock.shadowRoot!.querySelectorAll('.tab')).map((t) => (t as HTMLElement).dataset.tab ?? '')
    })
    expect(defaults).toContain('terminal')
    expect(defaults).not.toContain('va-source')

    // Toggle back on.
    await page.keyboard.press(`${MOD}+Shift+E`)
    await expect.poll(
      () => page.evaluate(() => document.getElementById('ship-panel-dock')?.hasAttribute('edit-mode-on')),
    ).toBe(true)
  })

  test('edit-mode is per-view: turning on for /agents does not turn on for /skills', async ({ page }) => {
    await enterEditMode(page)
    await page.evaluate(() => history.pushState({}, '', '/studio/skills'))
    await page.evaluate(() => window.dispatchEvent(new PopStateEvent('popstate')))
    await waitForChrome(page)
    // Different view → edit-mode-on bit clears.
    await expect.poll(
      () => page.evaluate(() => document.getElementById('ship-panel-dock')?.hasAttribute('edit-mode-on')),
      { timeout: 5000 },
    ).toBe(false)

    // Going back to /agents restores it (persisted).
    await page.evaluate(() => history.pushState({}, '', '/agents'))
    await page.evaluate(() => window.dispatchEvent(new PopStateEvent('popstate')))
    await waitForChrome(page)
    await expect.poll(
      () => page.evaluate(() => document.getElementById('ship-panel-dock')?.hasAttribute('edit-mode-on')),
      { timeout: 5000 },
    ).toBe(true)
  })

  test('Components tab lists the components present in the mocked source', async ({ page }) => {
    await enterEditMode(page)
    // Activate Components tab.
    await page.evaluate(() => (document.getElementById('ship-panel-dock') as HTMLElement & { activate(t: string): void }).activate('va-components'))
    // Mocked source contains <sui-card>, <sui-progress>.
    await expect.poll(async () => {
      return page.evaluate(() => {
        const el = document.querySelector('sui-va-components')
        if (!el) return []
        return Array.from(el.shadowRoot!.querySelectorAll('.chip .tag')).map((n) => n.textContent ?? '')
      })
    }, { timeout: 7000 }).toEqual(expect.arrayContaining(['<sui-card>', '<sui-progress>']))
  })

  test('picker tile click fires sui:component:picked with the tag', async ({ page }) => {
    await enterEditMode(page)
    await page.evaluate(() => (document.getElementById('ship-panel-dock') as HTMLElement & { activate(t: string): void }).activate('va-components'))

    // Wait for picker entries to load and pick the first.
    const picked = await page.evaluate(async () => {
      const tagPromise = new Promise<string[]>((resolve) => {
        document.addEventListener('sui:component:picked', (e) => {
          resolve(((e as CustomEvent).detail?.tags ?? []) as string[])
        }, { once: true })
      })
      const compTab = document.querySelector('sui-va-components')!
      // Wait briefly for the picker shadow to populate.
      const picker = compTab.shadowRoot!.querySelector('sui-component-picker') as HTMLElement & { refresh?(): void }
      for (let i = 0; i < 50; i++) {
        const tile = picker.shadowRoot?.querySelector('.tile') as HTMLButtonElement | null
        if (tile) { tile.click(); break }
        await new Promise((r) => setTimeout(r, 100))
      }
      const tags = await Promise.race([
        tagPromise,
        new Promise<string[]>((resolve) => setTimeout(() => resolve([]), 2000)),
      ])
      return tags
    })
    expect(picked.length).toBeGreaterThanOrEqual(1)
    expect(picked[0]).toMatch(/^sui-/)
  })

  test('Source tab loads mocked source; edit + Cmd+S writes via the stub', async ({ page }) => {
    await enterEditMode(page)
    await page.evaluate(() => (document.getElementById('ship-panel-dock') as HTMLElement & { activate(t: string): void }).activate('va-source'))

    // Wait for editor to receive its value (mock returns sample source).
    await expect.poll(async () => {
      return page.evaluate(() => {
        const va = document.querySelector('sui-va-source')
        const ce = va?.shadowRoot?.querySelector('sui-code-editor') as (HTMLElement & { value?: string }) | null
        return ce?.value ?? ''
      })
    }, { timeout: 7000 }).toContain('sui-card')

    // Save via Cmd+S → invoke writeSource stub directly (mock returns ok).
    const wrote = await page.evaluate(async () => {
      const va = document.querySelector('sui-va-source') as (HTMLElement & { reload?: () => void }) | null
      const ce = va?.shadowRoot?.querySelector('sui-code-editor') as (HTMLElement & { value?: string }) | null
      if (!ce) return false
      ce.value = '<sui-card>edited</sui-card>'
      // Synthetic sui-save event — controller listens via on:sui-save.
      ce.dispatchEvent(new CustomEvent('sui-save', { detail: { value: ce.value }, bubbles: true, composed: true }))
      // Allow the async write to resolve.
      await new Promise((r) => setTimeout(r, 200))
      const state = va!.shadowRoot!.querySelector('.state') as HTMLElement
      return state.dataset['saved'] === 'true' || state.textContent?.trim() === 'saved'
    })
    expect(wrote).toBe(true)
  })

  test('Activity tab renders entries from a synthetic annotation event', async ({ page }) => {
    await enterEditMode(page)
    await page.evaluate(() => (document.getElementById('ship-panel-dock') as HTMLElement & { activate(t: string): void }).activate('va-activity'))

    await page.evaluate(() => {
      window.__viewMcp?.fireAnnotation('sui-card', 'Test annotation')
    })

    await expect.poll(async () => {
      return page.evaluate(() => {
        const a = document.querySelector('sui-va-activity')
        return a?.shadowRoot?.querySelectorAll('details').length ?? 0
      })
    }, { timeout: 3000 }).toBeGreaterThanOrEqual(1)
  })

  test('annotation pin appears on mock SSE event', async ({ page }) => {
    await enterEditMode(page)
    // The pin overlay is body-level; the mocked annotation needs a
    // selector that resolves under #ship-content. The agents view
    // renders something — pick a robust selector that exists.
    const ok = await page.evaluate(async () => {
      // Make sure pins layer is upgraded.
      const layer = document.getElementById('sui-va-pins') as (HTMLElement & { pinCount?: number }) | null
      if (!layer) return false
      window.__viewMcp?.fireAnnotation('main#ship-content > *', 'Mocked')
      // Wait briefly for the pin to render.
      for (let i = 0; i < 25; i++) {
        const c = layer.pinCount ?? 0
        if (c >= 1) return true
        await new Promise((r) => setTimeout(r, 100))
      }
      return false
    })
    expect(ok).toBe(true)
  })
})
