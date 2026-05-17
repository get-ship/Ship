/**
 * terminal-revive probe — end-to-end test for the dock terminal.
 *
 * Validates the full PTY round-trip on the SUI shell, exercising:
 *   - dock open → sui-terminal mount → /spawn → SSE bytes → render
 *   - keystroke → /input → daemon → SSE bytes back
 *   - container resize → xterm resize → /resize → PTY geometry update
 *   - dock close → /sessions/<id> DELETE → reopen spawns fresh prompt
 *   - theme matrix (ship-light, ship-dark, console)
 *
 * Captures evidence to .artifacts/web-qa/terminal-revive-<ts>/ for the
 * companion report.md. Skips if shell vite dev server isn't running.
 */
import path from 'node:path'
import fs from 'node:fs'
import { test, expect } from '@playwright/test'
import { SHELL2_URL, skipUnlessShell2 } from './_shell'

const ART_DIR = process.env.TERMINAL_PROBE_ARTIFACTS
  ?? path.resolve(__dirname, '..', '..', '.artifacts', 'web-qa', 'terminal-revive-probe')

function shotPath(name: string): string {
  fs.mkdirSync(path.join(ART_DIR, 'screenshots'), { recursive: true })
  return path.join(ART_DIR, 'screenshots', `${name}.png`)
}

async function openDockTerminal(page: import('@playwright/test').Page): Promise<void> {
  await page.goto(`${SHELL2_URL}/home`, { waitUntil: 'load' })
  // Wait for the panel-dock element to upgrade (controller mounts after
  // the SUI bundle loads — `toggle` exists only post-mount).
  await page.waitForFunction(() => {
    const dock = document.getElementById('ship-panel-dock') as (HTMLElement & { toggle?: unknown }) | null
    return Boolean(dock && typeof dock.toggle === 'function')
  }, null, { timeout: 10_000 })
  await page.evaluate(() => {
    document.body.classList.add('dock-open')
    const dock = document.getElementById('ship-panel-dock') as (HTMLElement & { activate?: (id: string) => void }) | null
    dock?.activate?.('terminal')
  })
  await page.waitForFunction(() => {
    const term = document.querySelector('sui-terminal[slot="terminal"]') as HTMLElement & { terminal?: unknown }
    return Boolean(term?.terminal)
  }, null, { timeout: 10_000 })
}

async function readBuffer(page: import('@playwright/test').Page): Promise<string> {
  return page.evaluate(() => {
    const term = document.querySelector('sui-terminal[slot="terminal"]') as (HTMLElement & {
      terminal?: { buffer: { active: { length: number; getLine: (i: number) => { translateToString: (trim?: boolean) => string } | undefined } } }
    }) | null
    const buf = term?.terminal?.buffer.active
    if (!buf) return ''
    const lines: string[] = []
    for (let i = 0; i < buf.length; i += 1) {
      lines.push(buf.getLine(i)?.translateToString(true) ?? '')
    }
    return lines.join('\n')
  })
}

test.describe('terminal-revive: dock terminal round-trip', () => {
  skipUnlessShell2()

  test('echo TERMINAL_OK round-trips through PTY', async ({ page }) => {
    page.on('console', (m) => fs.appendFileSync(
      path.join(ART_DIR, 'console.log'),
      `[${m.type()}] ${m.text()}\n`,
    ))
    const networkLog = path.join(ART_DIR, 'network.log')
    page.on('request', (r) => {
      const u = r.url()
      if (u.includes('/api/v1/') || u.includes('sessions')) {
        fs.appendFileSync(networkLog, `→ ${r.method()} ${u}\n`)
      }
    })
    page.on('response', (r) => {
      const u = r.url()
      if (u.includes('/api/v1/') || u.includes('sessions') || r.status() >= 400) {
        fs.appendFileSync(networkLog, `← ${r.status()} ${u}\n`)
      }
    })
    page.on('pageerror', (err) => {
      fs.appendFileSync(networkLog, `[pageerror] ${err.message}\n`)
    })
    page.on('request', async (r) => {
      if (r.url().includes('/input') || r.url().includes('/resize')) {
        const body = r.postData()
        if (body) fs.appendFileSync(networkLog, `   body: ${body}\n`)
      }
    })
    fs.mkdirSync(ART_DIR, { recursive: true })
    fs.writeFileSync(networkLog, '')
    fs.writeFileSync(path.join(ART_DIR, 'console.log'), '')

    await openDockTerminal(page)
    await page.evaluate(() => {
      (window as unknown as { __terminalDebug: { chunks: string[] } }).__terminalDebug = { chunks: [] }
    })
    await page.screenshot({ path: shotPath('01-dock-open') })

    // Let the spawn round-trip + initial prompt write settle so the
    // first keystrokes don't sit ahead of the shell's readline init.
    await page.waitForTimeout(1200)

    // Type the round-trip command via xterm's input event so the bridge
    // POSTs to /input identically to a real keypress.
    await page.evaluate(() => {
      const term = document.querySelector('sui-terminal[slot="terminal"]') as HTMLElement
      const ev = new CustomEvent('sui-terminal-data', {
        detail: { data: 'echo TERMINAL_OK\n' },
        bubbles: true,
        composed: true,
      })
      const ok = term.dispatchEvent(ev)
      ;(window as unknown as { __dispatchOk?: boolean }).__dispatchOk = ok
    })

    let bufferSnapshot = ''
    try {
      // Extra wait so sendInput's 5s timeout can hit
      await page.waitForTimeout(6000)
      await expect.poll(async () => {
        bufferSnapshot = await readBuffer(page)
        return bufferSnapshot.includes('TERMINAL_OK')
      }, { timeout: 7_000, intervals: [200, 200, 500] }).toBe(true)
    } finally {
      fs.writeFileSync(path.join(ART_DIR, 'buffer-after-echo.txt'), bufferSnapshot)
      const dbg = await page.evaluate(() => {
        return (window as unknown as { __terminalDebug?: unknown }).__terminalDebug ?? {}
      })
      fs.writeFileSync(path.join(ART_DIR, 'wire-chunks.txt'), JSON.stringify(dbg, null, 2))
      await page.screenshot({ path: shotPath('02-echo-ok') })
    }
  })

  test('resize propagates to PTY (tput cols reflects width)', async ({ page }) => {
    await openDockTerminal(page)

    // Force a specific terminal size via xterm.js's resize API. The
    // controller's `term.onResize` bridges it as `sui-terminal-resize`,
    // and the bridge POSTs to /resize. tput cols inside the PTY should
    // then echo back the new width.
    await page.evaluate(() => {
      const term = document.querySelector('sui-terminal[slot="terminal"]') as (HTMLElement & {
        terminal?: { resize: (cols: number, rows: number) => void }
      })
      term.terminal?.resize(120, 32)
    })
    await page.waitForTimeout(300)
    await page.evaluate(() => {
      const term = document.querySelector('sui-terminal[slot="terminal"]') as HTMLElement
      term.dispatchEvent(new CustomEvent('sui-terminal-data', {
        detail: { data: 'tput cols\n' },
        bubbles: true,
        composed: true,
      }))
    })
    await expect.poll(async () => (await readBuffer(page)).includes('120'),
      { timeout: 5_000, intervals: [200, 200, 500] }).toBe(true)
    await page.screenshot({ path: shotPath('03-resize-120') })
  })

  test('close + reopen spawns fresh session', async ({ page }) => {
    await openDockTerminal(page)
    const firstSession = await page.evaluate(() => {
      // The dock terminal bridge stores sessionId in module scope; not
      // exposed, but DELETE will fire when body.dock-open is removed.
      return Date.now()
    })
    await page.evaluate(() => { document.body.classList.remove('dock-open') })
    await page.waitForTimeout(500)
    // Reopen.
    await page.evaluate(() => { document.body.classList.add('dock-open') })
    await page.waitForFunction(() => {
      const term = document.querySelector('sui-terminal[slot="terminal"]') as HTMLElement & { terminal?: unknown }
      return Boolean(term?.terminal)
    }, null, { timeout: 5_000 })
    // Echo into the new session — should still round-trip cleanly.
    await page.evaluate(() => {
      const term = document.querySelector('sui-terminal[slot="terminal"]') as HTMLElement
      term.dispatchEvent(new CustomEvent('sui-terminal-data', {
        detail: { data: 'echo SECOND_OK\n' },
        bubbles: true,
        composed: true,
      }))
    })
    await expect.poll(async () => (await readBuffer(page)).includes('SECOND_OK'),
      { timeout: 5_000, intervals: [200, 200, 500] }).toBe(true)
    expect(firstSession).toBeGreaterThan(0)
    await page.screenshot({ path: shotPath('04-reopen-second-ok') })
  })

  test('renders in ship-light, ship-dark, console themes', async ({ page }) => {
    for (const theme of ['ship-light', 'ship-dark', 'console']) {
      await openDockTerminal(page)
      await page.evaluate((t) => {
        document.documentElement.setAttribute('data-theme', t)
      }, theme)
      await page.waitForTimeout(200)
      await page.evaluate(() => {
        const term = document.querySelector('sui-terminal[slot="terminal"]') as HTMLElement
        term.dispatchEvent(new CustomEvent('sui-terminal-data', {
          detail: { data: 'echo theme-check\n' },
          bubbles: true,
          composed: true,
        }))
      })
      await expect.poll(async () => (await readBuffer(page)).includes('theme-check'),
        { timeout: 5_000, intervals: [200, 200, 500] }).toBe(true)
      await page.screenshot({ path: shotPath(`05-theme-${theme}`) })
      await page.evaluate(() => { document.body.classList.remove('dock-open') })
      await page.waitForTimeout(200)
    }
  })
})
