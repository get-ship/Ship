/**
 * Visual verification for <sui-agent-attach> + xterm rendering.
 *
 * The route + bridge are already covered by Rust + vitest. This spec
 * proves the component renders correctly when bytes flow:
 *   1. mount <sui-agent-attach slug="probe"> in a daemon-served harness
 *   2. intercept the WS upgrade via routeWebSocket; push fake PTY bytes
 *   3. assert xterm decodes them into the visible buffer
 *   4. snapshot the rendered terminal for visual diff
 *
 * No daemon-side PTY needed — the route handler is bypassed by
 * routeWebSocket, which is the seam between client and server.
 */
import fs from 'node:fs'
import path from 'node:path'
import { test, expect, type Page } from '@playwright/test'
import { getDaemonUrl } from './helpers/daemon'
import { injectStability } from './helpers/render'

/// Every theme registered in packages/ui-components/src/tokens/themes.css.
/// Captured so the user can eyeball legibility per theme — most are light,
/// ship-dark + console are dark, halo + glass are tinted.
const THEMES = [
  'paper',
  'ship-light',
  'ship-dark',
  'console',
  'halo',
  'glass',
  'contrast',
] as const
type ThemeId = (typeof THEMES)[number]

const DARK_THEMES = new Set<ThemeId>(['ship-dark', 'console'])

/// Pin the `data-theme` attribute before the harness boots so token CSS
/// resolves correctly on first paint. Mirrors render.ts::pinTheme but
/// accepts every registered theme, not just the visual-baseline pair.
async function pinAnyTheme(page: Page, theme: ThemeId): Promise<void> {
  const dark = DARK_THEMES.has(theme)
  await page.addInitScript(
    ({ theme, dark }) => {
      const apply = (): void => {
        const root = document.documentElement
        root.dataset.theme = theme
        root.classList.toggle('dark', dark)
      }
      if (document.documentElement) apply()
      document.addEventListener('DOMContentLoaded', apply, { once: true })
    },
    { theme, dark },
  )
}

// Persist outside the playwright outputDir so passing-run evidence
// survives between invocations. Mirrors terminal-revive-probe.spec.ts.
const ART_DIR = path.resolve(__dirname, '..', '..', '..', '.artifacts', 'web-qa', 'agent-attach')

const REPO_ROOT = path.resolve(__dirname, '..', '..', '..')
const ATTACH_BUNDLE = path.join(
  REPO_ROOT,
  'crates/sui/ship-sui-server/assets/sui-components/sui-agent-attach.js',
)
// <sui-agent-attach> calls host.fit() on the inner <sui-terminal>, which
// only exists on the freshly-built bundle. Serve that from disk too so a
// daemon from another worktree doesn't shadow the new method with a
// stale copy.
const TERMINAL_BUNDLE = path.join(
  REPO_ROOT,
  'crates/sui/ship-sui-server/assets/sui-components/sui-terminal.js',
)

/// Synthesized harness that loads the sui-terminal + sui-agent-attach
/// lazy bundles directly (components.js doesn't include either) and
/// mounts a single <sui-agent-attach> with a known slug.
function attachHarnessHtml(slug: string, theme: ThemeId): string {
  // The `.dark` class carries the dark palette (--background etc.);
  // `data-theme` only overrides theme-specific extras. Both must be on
  // the root element at parse time so the cascade is correct BEFORE the
  // module scripts execute and <sui-terminal> reads tokens at mount.
  const cls = DARK_THEMES.has(theme) ? 'dark' : ''
  return `<!doctype html>
<html lang="en" data-theme="${theme}" class="${cls}">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>sui-agent-attach harness</title>
  <link rel="stylesheet" href="/api/assets/view-tokens.css" />
  <script type="importmap">
  {
    "imports": {
      "@ship/sui-runtime": "/_sui/runtime.js",
      "@ship/sui-dom":     "/_sui/dom.js",
      "@ship/sdk-client":  "/_sui/sdk-client.js"
    }
  }
  </script>
  <script type="module" src="/_sui/components.js"></script>
  <script type="module" src="/_sui/sui-components/sui-terminal.js"></script>
  <script type="module" src="/_sui/sui-components/sui-agent-attach.js"></script>
  <style>
    html, body { margin: 0; padding: 0; background: var(--background, #fff); color: var(--foreground, #111); font-family: var(--font-sans, system-ui, sans-serif); }
    #harness { padding: 24px; height: 100vh; box-sizing: border-box; }
    sui-agent-attach { width: 100%; height: 100%; border-radius: 12px; overflow: hidden; }
  </style>
</head>
<body>
  <div id="harness">
    <sui-agent-attach slug="${slug}" auto-connect></sui-agent-attach>
  </div>
  <script>
    (function () {
      var deadline = Date.now() + 6000
      function tick () {
        var el = document.querySelector('sui-agent-attach')
        var ready = el && customElements.get('sui-agent-attach') && customElements.get('sui-terminal')
        if (ready || Date.now() > deadline) {
          window.__harnessReady = true
          return
        }
        requestAnimationFrame(tick)
      }
      tick()
    })()
  </script>
</body>
</html>`
}

async function gotoAttachHarness(page: Page, slug: string, theme: ThemeId): Promise<void> {
  await pinAnyTheme(page, theme)
  await page.route('**/__attach_harness', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'text/html; charset=utf-8',
      body: attachHarnessHtml(slug, theme),
    }),
  )
  // Serve the just-built sui-agent-attach.js from this worktree even when
  // the daemon under test is from a different workspace and doesn't have
  // the bundle yet. The other bundles (components.js, sui-terminal.js,
  // runtime.js, ...) come from the daemon.
  const serveBundle = (file: string) => async (route: import('@playwright/test').Route) => {
    if (!fs.existsSync(file)) {
      await route.fulfill({ status: 404, body: `bundle ${file} not built; run \`bun run build:bundles\`` })
      return
    }
    console.log(`[route] serving ${path.basename(file)} (${fs.statSync(file).size}b) for ${route.request().url()}`)
    await route.fulfill({
      status: 200,
      contentType: 'application/javascript; charset=utf-8',
      body: fs.readFileSync(file, 'utf8'),
    })
  }
  await page.route('**/_sui/sui-components/sui-agent-attach.js', serveBundle(ATTACH_BUNDLE))
  await page.route('**/_sui/sui-components/sui-terminal.js', serveBundle(TERMINAL_BUNDLE))
  await page.goto('/__attach_harness')
  await page.waitForFunction(
    () => (window as unknown as { __harnessReady?: boolean }).__harnessReady === true,
    null,
    { timeout: 15_000 },
  )
  await injectStability(page)
}

/// Drain the xterm.js buffer inside the inner <sui-terminal> into a flat
/// string so assertions can match what the user would visually see.
async function readTerminalBuffer(page: Page): Promise<string> {
  return page.evaluate(() => {
    const attach = document.querySelector('sui-agent-attach') as HTMLElement | null
    if (!attach) return ''
    const shadow = attach.shadowRoot
    const term = (shadow?.querySelector('sui-terminal') ?? null) as (HTMLElement & {
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

test.describe('sui-agent-attach visual + WS round-trip', () => {
  test('xterm renders bytes pushed over the attach WebSocket', async ({ page }) => {
    page.on('console', (m) => console.log(`[browser:${m.type()}] ${m.text()}`))
    page.on('pageerror', (e) => console.error(`[pageerror] ${e.message}`))
    page.on('websocket', (ws) => console.log(`[ws] ${ws.url()}`))

    // Route the WS upgrade before navigation. The bridge opens
    // ws://<host>/api/v1/agent_sessions/probe/attach on mount; routeWebSocket
    // intercepts and gives us a server-side handle to push frames.
    let serverSocket: import('@playwright/test').WebSocketRoute | null = null
    await page.routeWebSocket(/agent_sessions\/probe\/attach/, (ws) => {
      serverSocket = ws
    })

    await gotoAttachHarness(page, 'probe', 'paper')

    // Wait for the bridge to upgrade the WS — the component dispatches
    // sui-agent-attach-open once `connected` flips true. We don't have
    // direct access to the event in the test runner, so poll for the
    // server-side handle instead.
    await page.waitForFunction(() => true, null, { timeout: 100 })
    expect(await pollUntil(() => serverSocket !== null, 5000))
      .toBe(true)

    // Push a fake PTY banner — the bridge writes raw bytes to xterm.
    const banner = '\x1b[32mship-probe\x1b[0m: hello from the attach socket\r\n'
    serverSocket!.send(Buffer.from(banner, 'utf8'))

    // xterm decodes on the next animation frame. Poll the buffer.
    const ok = await pollUntil(async () => {
      const text = await readTerminalBuffer(page)
      return text.includes('hello from the attach socket')
    }, 5000)
    expect(ok, 'xterm buffer must contain the pushed banner').toBe(true)

    // Confirm xterm sized itself to fill the container (not stuck at the
    // default 24×80) — this is the regression the e2e exists to catch.
    const dims = await page.evaluate(() => {
      const attach = document.querySelector('sui-agent-attach') as HTMLElement | null
      const term = (attach?.shadowRoot?.querySelector('sui-terminal') ?? null) as (HTMLElement & {
        terminal?: { rows: number; cols: number } | null
      }) | null
      return {
        rows: term?.terminal?.rows ?? 0,
        cols: term?.terminal?.cols ?? 0,
      }
    })
    expect(dims.rows, 'xterm should fit its container, not the 24-row default').toBeGreaterThan(30)
    expect(dims.cols, 'xterm should fit its container width').toBeGreaterThan(80)

    fs.mkdirSync(ART_DIR, { recursive: true })
    await page.screenshot({
      path: path.join(ART_DIR, 'attached-paper.png'),
      fullPage: true,
    })
    await page.locator('sui-agent-attach').screenshot({
      path: path.join(ART_DIR, 'attached-paper-cropped.png'),
    })
  })

  // Theme matrix: same banner pushed across every registered theme so the
  // user can verify xterm + the overlay card stay legible. Captures one
  // screenshot per theme; no pixel-diff assertion — eyeball check.
  for (const theme of THEMES) {
    test(`xterm legibility — theme: ${theme}`, async ({ page }) => {
      let serverSocket: import('@playwright/test').WebSocketRoute | null = null
      await page.routeWebSocket(/agent_sessions\/probe-theme\/attach/, (ws) => {
        serverSocket = ws
      })
      await gotoAttachHarness(page, 'probe-theme', theme)
      expect(await pollUntil(() => serverSocket !== null, 5000)).toBe(true)

      // Banner: a green-success line, a yellow-warning line, a red-error
      // line, and plain foreground. Picks up four token colors so the
      // theme's ANSI palette is exercised, not just the default fg/bg.
      const banner = [
        '\x1b[32m✓ ok\x1b[0m — successful step',
        '\x1b[33m! warn\x1b[0m — non-fatal advisory',
        '\x1b[31m✗ err\x1b[0m — something broke',
        'plain foreground line',
        '$ ',
      ].join('\r\n') + '\r\n'
      serverSocket!.send(Buffer.from(banner, 'utf8'))

      await pollUntil(async () => (await readTerminalBuffer(page)).includes('successful step'), 5000)
      // Wait for fit() to settle — paper sometimes snapshots before the
      // ResizeObserver-driven refit fires, leaving xterm at its initial
      // small row count. Polling rows > 30 makes the capture deterministic.
      await pollUntil(async () => {
        const rows = await page.evaluate(() => {
          const attach = document.querySelector('sui-agent-attach') as HTMLElement | null
          const term = attach?.shadowRoot?.querySelector('sui-terminal') as (HTMLElement & {
            terminal?: { rows: number } | null
          }) | null
          return term?.terminal?.rows ?? 0
        })
        return rows > 30
      }, 5000)
      // Even after rows updates, xterm's canvas needs a few frames to
      // repaint at the new geometry. Without this the paper screenshot
      // captures the pre-fit small render despite the row count being
      // correct.
      await page.evaluate(
        () => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => requestAnimationFrame(r)))),
      )
      await page.waitForTimeout(120)

      const xtermTheme = await page.evaluate(() => {
        const attach = document.querySelector('sui-agent-attach') as HTMLElement | null
        const term = (attach?.shadowRoot?.querySelector('sui-terminal') ?? null) as (HTMLElement & {
          terminal?: { options?: { theme?: Record<string, string> } } | null
        }) | null
        const t = term?.terminal?.options?.theme ?? {}
        const live = term ? getComputedStyle(term).getPropertyValue('--background').trim() : ''
        return {
          dataTheme: document.documentElement.dataset.theme,
          liveBg: live,
          xtermBg: t.background, xtermFg: t.foreground,
        }
      })
      console.log(`[theme=${theme}] ${JSON.stringify(xtermTheme)}`)

      fs.mkdirSync(ART_DIR, { recursive: true })
      await page.locator('sui-agent-attach').screenshot({
        path: path.join(ART_DIR, `theme-${theme}.png`),
      })
    })
  }

  test('renders disconnected overlay when the WS closes', async ({ page }) => {
    let serverSocket: import('@playwright/test').WebSocketRoute | null = null
    await page.routeWebSocket(/agent_sessions\/probe-disc\/attach/, (ws) => {
      serverSocket = ws
    })

    await gotoAttachHarness(page, 'probe-disc', 'paper')
    expect(await pollUntil(() => serverSocket !== null, 5000)).toBe(true)

    // Server hangs up — the bridge fires onDisconnect and the component
    // flips into the overlay state.
    serverSocket!.close({ code: 1011, reason: 'server bounced' })

    const overlay = page.locator('sui-agent-attach').locator('[data-test="disconnected"]')
    await expect(overlay).toBeVisible({ timeout: 5000 })
    fs.mkdirSync(ART_DIR, { recursive: true })
    await page.screenshot({
      path: path.join(ART_DIR, 'disconnected-paper.png'),
      fullPage: true,
    })
    await page.locator('sui-agent-attach').screenshot({
      path: path.join(ART_DIR, 'disconnected-paper-cropped.png'),
    })
  })
})

/// Spin until `pred()` returns truthy or the deadline lapses. Returns
/// whether it ever succeeded. The callbacks can be sync or async.
async function pollUntil(pred: () => boolean | Promise<boolean>, ms: number): Promise<boolean> {
  const deadline = Date.now() + ms
  while (Date.now() < deadline) {
    if (await pred()) return true
    await new Promise((r) => setTimeout(r, 50))
  }
  return false
}

// Keep getDaemonUrl referenced so tsc doesn't strip the import — the
// playwright config baseURL pulls from it. (Defensive: ts-strict in this
// repo errors on unused imports.)
void getDaemonUrl
