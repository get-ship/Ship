/**
 * shell global keyboard shortcuts.
 *
 * main.ts wires a document-level keydown handler that maps
 * `Ctrl/Cmd + B/J/K` to the rail, panel dock, and command bar. This
 * spec locks in that contract — ported from `keyboard-probe.mjs`.
 *
 * Uses `Control` on Linux/CI; the handler matches `metaKey || ctrlKey`,
 * so `Control` exercises the same path on every platform.
 *
 * Runs against shell's vite dev server; skipped when unreachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

const MOD = process.platform === 'darwin' ? 'Meta' : 'Control'

test.describe('shell keyboard shortcuts', () => {
  skipUnlessShell2()

  test.beforeEach(async ({ page }) => {
    await page.goto(`${SHELL2_URL}/home`, { waitUntil: 'load' })
    // Wait until the chrome islands are upgraded and main.ts has wired
    // its keydown handler.
    await page.waitForFunction(
      () => {
        const bar = document.getElementById('ship-command-bar') as
          (HTMLElement & { toggle?: unknown }) | null
        const dock = document.getElementById('ship-panel-dock') as
          (HTMLElement & { toggle?: unknown }) | null
        const rail = document.getElementById('ship-rail')
        return Boolean(
          bar && typeof bar.toggle === 'function' &&
          dock && typeof dock.toggle === 'function' &&
          rail,
        )
      },
      null,
      { timeout: 10_000 },
    )
  })

  test('Ctrl/Cmd+B toggles the rail [collapsed] attribute', async ({ page }) => {
    const before = await page.evaluate(
      () => document.getElementById('ship-rail')!.hasAttribute('collapsed'),
    )
    await page.keyboard.press(`${MOD}+B`)
    await expect
      .poll(() => page.evaluate(
        () => document.getElementById('ship-rail')!.hasAttribute('collapsed'),
      ))
      .toBe(!before)

    // Toggling again restores the original state.
    await page.keyboard.press(`${MOD}+B`)
    await expect
      .poll(() => page.evaluate(
        () => document.getElementById('ship-rail')!.hasAttribute('collapsed'),
      ))
      .toBe(before)
  })

  test('Ctrl/Cmd+J toggles the panel dock (body.dock-open)', async ({ page }) => {
    const before = await page.evaluate(
      () => document.body.classList.contains('dock-open'),
    )
    await page.keyboard.press(`${MOD}+J`)
    await expect
      .poll(() => page.evaluate(() => document.body.classList.contains('dock-open')))
      .toBe(!before)

    await page.keyboard.press(`${MOD}+J`)
    await expect
      .poll(() => page.evaluate(() => document.body.classList.contains('dock-open')))
      .toBe(before)
  })

  test('Ctrl/Cmd+K opens the command bar; Escape closes it', async ({ page }) => {
    await page.keyboard.press(`${MOD}+K`)
    await expect
      .poll(() => page.evaluate(
        () => document.getElementById('ship-command-bar')!.hasAttribute('open'),
      ))
      .toBe(true)

    await page.keyboard.press('Escape')
    await expect
      .poll(() => page.evaluate(
        () => document.getElementById('ship-command-bar')!.hasAttribute('open'),
      ))
      .toBe(false)
  })
})
