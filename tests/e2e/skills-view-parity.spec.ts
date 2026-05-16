/**
 * Skills view parity — regression guard for the OG j2 → SUI port.
 *
 * Covers the bug the user explicitly called out ("Skills details do
 * not load tiptap and are seriously degraded from the og shell"):
 *   - the `<sui-markdown-editor>` island MUST mount on the detail route
 *   - its TipTap ProseMirror surface MUST be present + editable
 *   - typing into it MUST be saveable via the daemon
 *
 * Also covers the harvest's PRESERVE/AVOID expectations: filter chips
 * with live counts, list rows are real anchors (so right-click open
 * works), and routing to /skills/:id wires up the IDE shell.
 *
 * Runs against shell's vite dev server; skipped when unreachable.
 */
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'
import { getDaemonUrl } from './_helpers'

test.describe('skills view parity', () => {
  skipUnlessShell2()

  test('skills list renders with explorer chrome + filter chips', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/skills`, { waitUntil: 'load' })

    await expect(page.locator('.skills-explorer')).toBeVisible({ timeout: 15_000 })
    await expect(page.locator('input[aria-label="Search skills"]')).toBeVisible()
    // Filter chips (PRESERVE: "All N, Smart N, Docs N" with counts).
    const chips = page.locator('.se-pill')
    await expect(chips.first()).toBeVisible({ timeout: 10_000 })
    expect(await chips.count()).toBeGreaterThanOrEqual(2)

    // Skill rows must be present.
    const rows = page.locator('.se-skill')
    await expect(rows.first()).toBeVisible({ timeout: 15_000 })
    expect(await rows.count()).toBeGreaterThan(0)
  })

  test('skill rows are real anchors to /skills/:id (harvest AVOID compliance)', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/skills`, { waitUntil: 'load' })
    const firstRow = page.locator('.se-skill').first()
    await expect(firstRow).toBeVisible({ timeout: 15_000 })

    // Must be an <a href> — not a <button>.
    const tagName = await firstRow.evaluate((el) => el.tagName.toLowerCase())
    expect(tagName).toBe('a')
    const href = await firstRow.getAttribute('href')
    expect(href).toMatch(/^\/skills\/[^/]+$/)
  })

  test('clicking a skill routes to detail + mounts the markdown editor (tiptap regression)', async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (err) => errors.push(err.message))

    await page.goto(`${SHELL2_URL}/skills`, { waitUntil: 'load' })
    const firstRow = page.locator('.se-skill').first()
    await expect(firstRow).toBeVisible({ timeout: 15_000 })

    const href = await firstRow.getAttribute('href')
    await firstRow.click()
    await expect(page).toHaveURL(new RegExp(href!.replace(/[/]/g, '\\/') + '$'), { timeout: 10_000 })

    // Detail shell: explorer + breadcrumb + info panel.
    await expect(page.locator('.skills-explorer')).toBeVisible()
    await expect(page.locator('.se-breadcrumb')).toBeVisible({ timeout: 10_000 })
    await expect(page.locator('.skills-info')).toBeVisible({ timeout: 10_000 })

    // The user's complaint: tiptap didn't load. Assert it does — both the
    // island element AND its ProseMirror surface.
    const editor = page.locator('sui-markdown-editor[data-testid="skill-editor"]')
    await expect(editor).toBeVisible({ timeout: 15_000 })

    // ProseMirror node lives in shadow DOM; reach it via locator chain.
    const prose = editor.locator('.ProseMirror')
    await expect(prose).toBeVisible({ timeout: 15_000 })
    await expect(prose).toHaveAttribute('contenteditable', 'true')

    expect(errors, errors.join('\n')).toHaveLength(0)
  })

  test('editing the SKILL.md draft + saving round-trips through the daemon', async ({ page }) => {
    await page.goto(`${SHELL2_URL}/skills`, { waitUntil: 'load' })
    const firstRow = page.locator('.se-skill').first()
    await expect(firstRow).toBeVisible({ timeout: 15_000 })
    const href = await firstRow.getAttribute('href')
    const skillId = decodeURIComponent(href!.replace(/^\/skills\//, ''))
    await firstRow.click()

    const editor = page.locator('sui-markdown-editor[data-testid="skill-editor"]')
    const prose = editor.locator('.ProseMirror')
    await expect(prose).toBeVisible({ timeout: 15_000 })

    // Capture original body via daemon so we can restore at the end.
    const daemon = getDaemonUrl()
    const detailUrl = `${daemon}/_sui/data?expr=${encodeURIComponent(`ship.skills.detail(${JSON.stringify({ id: skillId })})`)}`
    const before = await readSseFirstPatch(detailUrl) as { body?: string }
    const originalBody = typeof before?.body === 'string' ? before.body : ''

    // Type a sentinel into the editor; the Save button should enable.
    await prose.click()
    await page.keyboard.press('End')
    const sentinel = `\n\nparity-test-${Date.now()}`
    await page.keyboard.type(sentinel)

    const saveBtn = page.locator('[data-testid="skill-save"]')
    await expect(saveBtn).toBeEnabled({ timeout: 5_000 })
    await saveBtn.click()

    // Banner flips to success.
    await expect(page.locator('.sd-save-banner')).toContainText('Saved', { timeout: 10_000 })

    // Daemon now serves the new body.
    const after = await readSseFirstPatch(detailUrl) as { body?: string }
    expect(after?.body ?? '').toContain('parity-test-')

    // Restore.
    const restore = await fetch(`${daemon}/api/v1/skills/${encodeURIComponent(skillId)}/body`, {
      method: 'PUT',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ body: originalBody }),
    })
    expect(restore.ok, `restore failed: ${restore.status}`).toBe(true)
  })
})

/**
 * Reads the first `set` patch off a `/_sui/data?expr=…` SSE channel and
 * returns its `value`. Times out at 5 s.
 */
async function readSseFirstPatch(url: string): Promise<unknown> {
  const resp = await fetch(url, { headers: { accept: 'text/event-stream' } })
  if (!resp.ok || !resp.body) throw new Error(`SSE ${resp.status}`)
  const reader = resp.body.getReader()
  const decoder = new TextDecoder()
  let buf = ''
  const deadline = Date.now() + 5000
  try {
    while (Date.now() < deadline) {
      const { value, done } = await reader.read()
      if (done) break
      buf += decoder.decode(value, { stream: true })
      let idx: number
      while ((idx = buf.indexOf('\n\n')) >= 0) {
        const frame = buf.slice(0, idx)
        buf = buf.slice(idx + 2)
        const dataLine = frame.split('\n').find((l) => l.startsWith('data:'))
        if (!dataLine) continue
        const json = dataLine.slice(5).trim()
        try {
          const patch = JSON.parse(json) as { op?: string; value?: unknown }
          if (patch.op === 'set') return patch.value
        } catch { /* ignore non-JSON frames */ }
      }
    }
    throw new Error('SSE timeout')
  } finally {
    try { await reader.cancel() } catch { /* noop */ }
  }
}
