/**
 * Git view parity tests — exercise the PRESERVE behaviors carried
 * forward from the OG j2 git view per
 * `.artifacts/launch-arc/reports/j2-shell-harvest.md` §Git.
 *
 * Harvest PRESERVE items (verbatim):
 *   1. Intent: tabs + split panes for status / diff / log / branches.
 *   2. Inline `ship.git.*` SDK calls — view fetches data directly via
 *      the typed SDK, not via a custom endpoint.
 *
 * The harvest's behavioral list was thin because the j2 view never
 * mounted in-app; we use the j2 source partials as the reference shape
 * and verify the SUI port matches that shape end-to-end.
 *
 * Tests are tolerant of an empty/clean repo: when there are no changes
 * the Changes panel shows the clean hero rather than the split pane.
 */
import { test, expect, type Locator } from '@playwright/test'
import { waitForAttach } from './_helpers'

async function gotoGit(page: import('@playwright/test').Page): Promise<void> {
  await page.goto('/sui/git')
  await waitForAttach(page)
  await page.locator('.git-head').waitFor({ timeout: 20_000 })
}

// ── PRESERVE 1: tabs + split panes ───────────────────────────────────────

test('PRESERVE: tabs are Changes / History / Branches and switch via on:click', async ({ page }) => {
  await gotoGit(page)
  const row = page.locator('.git-tabs-row')
  await expect(row).toBeVisible()
  await expect(row.locator('[name="changes"]')).toBeVisible()
  await expect(row.locator('[name="history"]')).toBeVisible()
  await expect(row.locator('[name="branches"]')).toBeVisible()

  await row.locator('[name="history"]').click()
  await expect(page.locator('.git-history')).toBeVisible()
  await row.locator('[name="branches"]').click()
  await expect(page.locator('.git-branches-split, sui-empty-state')).toBeVisible()
  await row.locator('[name="changes"]').click()
  await expect(page.locator('.git-body .git-clean, .git-body .git-split')).toBeVisible()
})

test('PRESERVE: keyboard 1/2/3 switches Changes/History/Branches', async ({ page }) => {
  await gotoGit(page)
  await page.keyboard.press('2')
  await expect(page.locator('.git-history')).toBeVisible()
  await page.keyboard.press('3')
  await expect(page.locator('.git-branches-split, .git-body sui-empty-state')).toBeVisible()
  await page.keyboard.press('1')
  await expect(page.locator('.git-body .git-clean, .git-body .git-split')).toBeVisible()
})

test('PRESERVE: Changes is a split pane when working tree is dirty', async ({ page }) => {
  await gotoGit(page)
  const split = page.locator('.git-split')
  const clean = page.locator('.git-clean')
  const hasSplit = (await split.count()) > 0
  if (!hasSplit) {
    // Repo is clean — the clean hero is the canonical shape per j2
    // partial `panel_changes_clean`. The split shape is exercised by
    // the next test once a file row is rendered.
    await expect(clean).toBeVisible()
    return
  }
  await expect(split).toBeVisible()
  await expect(split.locator('.git-left')).toBeVisible()
  await expect(split.locator('.git-right')).toBeVisible()
})

test('PRESERVE: selecting a file opens its diff in the right pane', async ({ page }) => {
  await gotoGit(page)
  const rows = page.locator('[data-test="git-file-row"]')
  const count = await rows.count()
  test.skip(count === 0, 'no file changes in this repo to drive selection')

  const first = rows.first()
  const path = (await first.textContent())?.trim().replace(/^[SMARD?]\s*/, '') ?? ''
  await first.click()
  await expect(first).toHaveAttribute('data-active', '')
  await expect(page.locator('[data-test="diff-path"]')).toHaveText(path, { timeout: 10_000 })
  await expect(page.locator('[data-test="diff-body"]')).toBeVisible()
})

test('PRESERVE: Branches is a split pane (rail + detail) when branches exist', async ({ page }) => {
  await gotoGit(page)
  await page.locator('.git-tabs-row [name="branches"]').click()
  const tiles = page.locator('[data-test="git-branch-tile"]')
  const count = await tiles.count()
  test.skip(count === 0, 'no branches in this repo')
  await expect(page.locator('.git-branches-split')).toBeVisible()
  await expect(page.locator('.git-bleft')).toBeVisible()
  await expect(page.locator('[data-test="git-branch-detail"]')).toBeVisible()
})

test('PRESERVE: clicking a branch tile drives the detail pane', async ({ page }) => {
  await gotoGit(page)
  await page.locator('.git-tabs-row [name="branches"]').click()
  const tiles = page.locator('[data-test="git-branch-tile"]')
  const count = await tiles.count()
  test.skip(count < 2, 'need ≥2 branches to verify selection updates detail')
  const target = tiles.nth(count - 1)
  const targetName = (await target.locator('.git-branch-name').textContent())?.trim() ?? ''
  await target.click()
  await expect(page.locator('[data-test="git-branch-detail"] .git-branch-detail-name'))
    .toHaveText(targetName)
})

// ── PRESERVE 2: inline ship.git.* SDK calls ──────────────────────────────

test('PRESERVE: overview is fed by ship.git.overview() over /_sui/data SSE', async ({ page, request }) => {
  // The init island carries the SSE subscribe URL.
  const html = await (await request.get('/sui/git')).text()
  const match = html.match(/<script[^>]*id="__set_init__"[^>]*>([^<]*)<\/script>/)
  expect(match).not.toBeNull()
  const init = JSON.parse(match![1]) as Record<string, unknown>
  expect(init.__subscribe__).toBe('/_sui/views/git/sse')

  // And after attach the view rendered the header from that payload —
  // proof the SSE channel resolved (vs the stale-state j2 AVOID bug).
  await gotoGit(page)
  await expect(page.locator('.git-title')).toHaveText('Git')
})

test('PRESERVE: selecting a commit issues a ship.git.diff SSE request', async ({ page }) => {
  await gotoGit(page)
  await page.locator('.git-tabs-row [name="history"]').click()
  const rows = page.locator('[data-test="history-rows"] sui-history-row')
  const count = await rows.count()
  test.skip(count === 0, 'no commits in this repo')

  // Watch for the diff SSE channel opening.
  const diffOpen = page.waitForRequest(
    (req) => req.url().includes('/_sui/data') && /ship\.git\.diff/.test(decodeURIComponent(req.url())),
    { timeout: 10_000 },
  )
  await rows.first().click()
  await diffOpen
  await expect(page.locator('.git-body .git-clean, .git-body .git-split')).toBeVisible()
})

// ── Behavioral: refresh + diff toolbar ──────────────────────────────────

test('Refresh action re-issues the overview SSE subscription', async ({ page }) => {
  await gotoGit(page)
  const reopen = page.waitForRequest(
    (req) => req.url().includes('/_sui/data') && /ship\.git\.overview/.test(decodeURIComponent(req.url())),
    { timeout: 10_000 },
  )
  await page.locator('[data-test="git-refresh"]').click()
  await reopen
})

test('Diff toolbar exposes unified/split segmented control', async ({ page }) => {
  await gotoGit(page)
  const rows = page.locator('[data-test="git-file-row"]')
  const n = await rows.count()
  test.skip(n === 0, 'no file changes to populate the diff')
  await rows.first().click()
  const seg = page.locator('.git-diff-toolbar sui-segmented')
  await expect(seg).toBeVisible()
  await expect(seg.locator('sui-segmented-option[value="unified"]')).toBeVisible()
  await expect(seg.locator('sui-segmented-option[value="split"]')).toBeVisible()
})

// ── AVOID-list checks ───────────────────────────────────────────────────

test('AVOID: navigation to /sui/git produces fresh content, not stale state', async ({ page, request }) => {
  // Visit setup, then git — the j2 AVOID bug was that the second route
  // didn't re-render. SUI's view-host issues a fresh /_sui/render for
  // each navigation; verify the git body appears regardless of prior
  // history.
  await page.goto('/sui/setup').catch(() => {})
  // setup may not exist as a SUI route in every checkout; ignore.
  await gotoGit(page)
  await expect(page.locator('.git-title')).toHaveText('Git')
  // Ensure setup wizard content isn't leaking in.
  await expect(page.locator('body')).not.toContainText('Welcome to SHIP')
})

// Helper: a Locator returning text content, trimmed.
async function text(loc: Locator): Promise<string> {
  return (await loc.textContent())?.trim() ?? ''
}
void text // exported for parity with shell2 helpers; unused here
