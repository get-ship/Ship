/**
 * Full-loop e2e: a hermetic `shipd` + the real deck-served shell + real data.
 *
 * Every other e2e spec here either drives the ship-free deck-demo or talks to a
 * shell vite dev server, and skips when neither is up. This one OWNS its daemon
 * — it boots a fresh `shipd` against a throwaway SHIP_HOME (see `_daemon.ts`),
 * drives the shell deck serves at `/`, and proves the loop closes against real
 * daemon state. No `just dev` required; nothing touches the operator's `~/.ship`.
 *
 * Coverage:
 *   1. HERMETIC — the daemon's home is a temp dir, never `~/.ship` (the
 *      prod-guard the loop suites share). Boot already refuses prod; this
 *      re-asserts it as a visible guarantee and teardown removes the dir.
 *   2. READ (daemon → UI) — the shell's persistent bottom strip
 *      (`<sui-shell-bottom>`) renders the daemon's live git branch (the daemon
 *      read the workspace's git status and streamed it). We assert the rendered
 *      branch against what `git` reports for this checkout, so the assertion is
 *      derived from real data the daemon sourced, not incidental static text.
 *   3. MUTATION (UI → daemon) — picking a theme in the shell's theme picker
 *      writes `shell.theme` to the daemon's RUNTIME settings table
 *      (`persistTheme` → theme transport → `SettingsService/Set`, see
 *      apps/shell/src/ops.ts). We assert the daemon row over Connect —
 *      UI-independent read-back — and that the UI reflected the choice live
 *      (html[data-theme] + .dark flip with no reload). Honest note: that
 *      immediate flip is the local-first apply; the daemon→UI push direction
 *      is what test 2 proves (git status streamed into the DOM).
 *   4. FORM MUTATION (UI → daemon → UI) — the /settings page hydrates its
 *      fields from the flat SettingsService rows (#298 fix), a
 *      `<form mutation="ship.settings.update">` blur-submit lands in the
 *      daemon's RUNTIME table, and the Saved badge flips on success.
 */
import { spawnSync } from 'node:child_process'
import os from 'node:os'
import { test, expect } from '@playwright/test'
import {
  bootDaemon,
  getRuntimeSetting,
  setRuntimeSetting,
  REPO_ROOT,
  type DaemonHandle,
} from './_daemon'

function git(args: string[]): string {
  return spawnSync('git', args, { cwd: REPO_ROOT, encoding: 'utf8' }).stdout.trim()
}

test.describe('full loop: hermetic daemon + deck-served shell', () => {
  // Boot + a real browser load + bounded live-data polls outlast the 30s
  // default; give each test room without masking a genuine hang.
  test.describe.configure({ timeout: 60_000 })

  let d: DaemonHandle

  test.beforeAll(async () => {
    const t0 = Date.now()
    d = await bootDaemon()
    // Surfaced for the CI-cost decision: how long a cold spec pays for boot.
    console.log(`[full-loop] shipd boot → /api/health green in ${Date.now() - t0}ms`)
  })
  test.afterAll(async () => {
    await d?.dispose()
  })

  test('hermetic: the daemon home is a temp dir, never ~/.ship', () => {
    const prodHome = `${os.homedir()}/.ship`
    expect(d.home).not.toBe(prodHome)
    expect(d.home.startsWith(prodHome)).toBe(false)
    // mkdtemp under the OS temp root — the prod-guard's contract.
    expect(d.home.includes('ship-e2e-')).toBe(true)
  })

  test('read: the shell bottom strip renders the daemon live git branch', async ({ page }) => {
    // Truth source: what git itself reports for this checkout. The daemon read
    // the same workspace, so the UI must match — proving real daemon data
    // reached the DOM (not a static string baked into the shell).
    const expectedBranch = git(['rev-parse', '--abbrev-ref', 'HEAD'])
    expect(expectedBranch.length).toBeGreaterThan(0)

    // `domcontentloaded`, not `load`: the shell keeps slow/404 resources in
    // flight that can hold `load` past 30s. The poll below waits on the real
    // condition (daemon data in the DOM), which is what this test is about.
    await page.goto(`${d.baseUrl}/`, { waitUntil: 'domcontentloaded' })

    // The bottom strip's branch cell only fills once the daemon git status
    // resolves and feeds the prop. Poll its text until it carries the expected
    // branch — the real condition, no fixed sleep. (The cell can mount empty
    // for a tick before the prop lands, so we poll text rather than presence.)
    await expect
      .poll(
        async () => {
          const t = await page
            .locator('sui-shell-bottom [part="branch"]')
            .first()
            .textContent()
            .catch(() => null)
          return (t ?? '').trim()
        },
        { timeout: 25_000, message: `bottom-strip branch never showed "${expectedBranch}"` },
      )
      .toContain(expectedBranch)
  })

  test('mutate: picking a theme in the UI persists to the daemon and reflects live', async ({
    page,
  }) => {
    // Prior state we control: a fresh SHIP_HOME has no stored theme row, so
    // the write below provably originates from this UI gesture.
    expect(await getRuntimeSetting<string>(d, 'shell.theme')).toBeNull()

    await page.goto(`${d.baseUrl}/`, { waitUntil: 'domcontentloaded' })

    const html = page.locator('html')
    // The boot path applies the default before any choice exists.
    await expect(html).toHaveAttribute('data-theme', 'ship-light', { timeout: 20_000 })

    // Open the picker (Playwright pierces the open shadow root) and pick
    // ship-dark — a non-default, so the assertion can't pass vacuously.
    await page.locator('sui-theme-picker button.trigger').click()
    const card = page.locator('sui-theme-picker [data-theme-card="ship-dark"]')
    await expect(card).toBeVisible({ timeout: 5_000 })
    await card.click()

    // UI reflected the choice live — same page, no reload.
    await expect(html).toHaveAttribute('data-theme', 'ship-dark')
    await expect(html).toHaveClass(/\bdark\b/)
    await expect(card).toHaveAttribute('aria-selected', 'true')

    // Daemon state changed — read the RUNTIME settings row back over Connect,
    // independent of anything the browser rendered.
    await expect
      .poll(() => getRuntimeSetting<string>(d, 'shell.theme'), {
        timeout: 10_000,
        message:
          'shell.theme never landed in the daemon RUNTIME settings table — the UI applied locally but the Connect write-through failed',
      })
      .toBe('ship-dark')
  })

  // ── settings-form mutation ──────────────────────────────────────────────
  //
  // The /settings page hydrates from the flat SettingsService RUNTIME rows
  // (#298 fix in apps/shell/src/settings/). The fields are native
  // `<input class="s-input">` elements — NOT `<sui-input>`: the arcade pack
  // also names a `sui-input` tag and the per-tag resolver's first-wins
  // collision serves the arcade `display:none` stub to the shell.
  test(
    'mutate: settings worktree.base_dir round-trips through the UI and reflects live',
    async ({ page }) => {
      // Seed a known prior value so the read shows seeded data, then change it.
      const seeded = `/seed/wt-${Date.now()}`
      await setRuntimeSetting(d, 'worktree.base_dir', seeded)

      await page.goto(`${d.baseUrl}/settings`, { waitUntil: 'domcontentloaded' })
      const inner = page.locator('input.s-input[aria-labelledby="s-label-worktree"]')
      await expect(inner).toBeVisible({ timeout: 20_000 })

      // Read path: the seeded value is what the field shows.
      await expect(inner).toHaveValue(seeded)

      // Drive the mutation: type a new value + blur (the form submits on blur).
      const next = `/mut/wt-${Date.now()}`
      await inner.fill(next)
      await inner.blur()

      // Daemon state changed — read it back over Connect, UI-independent.
      await expect
        .poll(() => getRuntimeSetting<string>(d, 'worktree.base_dir'), { timeout: 10_000 })
        .toBe(next)

      // UI reflected it live (the "Saved" badge flips on mutation success),
      // not via a manual reload.
      const savedBadge = page
        .locator('form:has(input[value="worktree.base_dir"]) .s-save-badge')
        .first()
      await expect(savedBadge).toHaveClass(/visible/, { timeout: 5_000 })
      await expect(inner).toHaveValue(next)
    },
  )
})
