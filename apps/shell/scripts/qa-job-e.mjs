/**
 * Job E theme-personality QA.
 * Captures the C13 component gallery, the agents view, and an open
 * launcher for each of: ship-light, ship-dark, paper, halo, console,
 * contrast.
 *
 * Run: node scripts/qa-job-e.mjs <out-dir>
 * Requires shipd on 127.0.0.1:9315.
 */
import { chromium } from 'playwright'
import { writeFile } from 'node:fs/promises'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const REPO = resolve(__dirname, '..', '..', '..')
const BUNDLE_PATH = resolve(REPO, 'apps/shipd/assets/ship-elements.js')

const OUT = process.argv[2]
if (!OUT) {
  console.error('usage: node scripts/qa-job-e.mjs <out-dir>')
  process.exit(1)
}

const DAEMON = 'http://127.0.0.1:9315'
const THEMES = ['ship-light', 'ship-dark', 'paper', 'halo', 'console', 'contrast']
const DARK = new Set(['ship-dark', 'contrast', 'halo', 'console'])

const galleryHtml = (theme) => `<!doctype html>
<html lang="en" data-theme="${theme}"${DARK.has(theme) ? ' class="dark"' : ''}>
<head>
  <meta charset="utf-8" />
  <title>Job E — ${theme}</title>
  <link rel="stylesheet" href="${DAEMON}/api/assets/view-tokens.css" />
  <style>
    html, body { margin: 0; padding: 0; min-height: 100vh; }
    body {
      background: var(--surface-0);
      color: var(--foreground);
      font-family: var(--body-font);
      padding: 32px;
    }
    h1 { font-family: var(--heading-font); font-weight: var(--heading-weight); margin: 0 0 8px; }
    h2 { font-family: var(--heading-font); font-weight: var(--heading-weight); margin: 32px 0 12px; font-size: 1.1rem; }
    .row { display: flex; flex-wrap: wrap; gap: 12px; align-items: center; margin-bottom: 8px; }
    .grid-cards { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; max-width: 1200px; }
    .swatch {
      padding: 16px; border-radius: var(--radius-lg);
      background: var(--surface-1); border: 1px solid var(--border);
      box-shadow: var(--shadow-card-rest);
    }
    .swatch.raised { background: var(--surface-2); box-shadow: var(--shadow-md); }
    .swatch h3 { margin: 0 0 4px; font-size: var(--text-sm); font-family: var(--heading-font); font-weight: var(--heading-weight); }
    .swatch p  { margin: 0; color: var(--muted-foreground); font-size: var(--text-xs); }
    .label { color: var(--muted-foreground); font-family: var(--label-font); font-weight: var(--label-weight); letter-spacing: var(--label-tracking); font-size: var(--text-xs); text-transform: uppercase; min-width: 96px; }
    .pill {
      display: inline-flex; align-items: center; padding: 4px 10px;
      border-radius: var(--radius-full); font-size: var(--text-xs); font-weight: 600;
      background: var(--muted); color: var(--muted-foreground); border: 1px solid var(--border);
    }
    .input { padding: 8px 12px; border-radius: var(--radius-md); border: 1px solid var(--border);
             background: var(--surface-1); color: var(--foreground); font-family: var(--body-font);
             font-size: var(--text-sm); }
  </style>
</head>
<body>
  <h1>Component gallery — ${theme}</h1>
  <p style="color:var(--muted-foreground);margin:0 0 24px">Theme personality QA. Background, type role, radii, shadows, interaction.</p>

  <h2>Buttons</h2>
  <div class="row">
    <span class="label">Solid</span>
    <ship-button variant="solid" tone="primary">Primary</ship-button>
    <ship-button variant="solid" tone="accent">Accent</ship-button>
    <ship-button variant="solid" tone="success">Success</ship-button>
    <ship-button variant="solid" tone="warning">Warning</ship-button>
    <ship-button variant="solid" tone="destructive">Destroy</ship-button>
  </div>
  <div class="row">
    <span class="label">Soft</span>
    <ship-button variant="soft" tone="primary">Primary</ship-button>
    <ship-button variant="soft" tone="accent">Accent</ship-button>
    <ship-button variant="soft" tone="info">Info</ship-button>
  </div>
  <div class="row">
    <span class="label">Outline</span>
    <ship-button variant="outline" tone="primary">Primary</ship-button>
    <ship-button variant="outline">Default</ship-button>
    <ship-button variant="ghost">Ghost</ship-button>
    <ship-button variant="solid" tone="primary" disabled>Disabled</ship-button>
  </div>

  <h2>Badges &amp; chips</h2>
  <div class="row">
    <ship-badge tone="default">default</ship-badge>
    <ship-badge tone="primary">primary</ship-badge>
    <ship-badge tone="success">success</ship-badge>
    <ship-badge tone="warning">warning</ship-badge>
    <ship-badge tone="destructive">destroy</ship-badge>
    <ship-badge tone="info">info</ship-badge>
    <span class="pill">muted pill</span>
  </div>

  <h2>Inputs</h2>
  <div class="row">
    <input class="input" placeholder="Search…" />
    <input class="input" value="prefilled" />
    <input class="input" placeholder="disabled" disabled />
  </div>

  <h2>Cards</h2>
  <div class="grid-cards">
    <div class="swatch">
      <h3>Resting card</h3>
      <p>surface-1, shadow-card-rest, theme radius.</p>
    </div>
    <div class="swatch raised">
      <h3>Raised panel</h3>
      <p>surface-2, shadow-md.</p>
    </div>
    <div class="swatch" style="border-color:var(--border-strong);box-shadow:var(--shadow-hover)">
      <h3>Hover-state card</h3>
      <p>border-strong, shadow-hover.</p>
    </div>
  </div>

  <h2>List tiles</h2>
  <div style="max-width:560px;display:flex;flex-direction:column;gap:6px">
    <ship-list-tile variant="row" interactive>
      <ship-icon slot="leading" name="bell"></ship-icon>
      Notifications
      <span slot="meta">Email me when a pipeline fails</span>
      <ship-badge slot="trailing" tone="success">on</ship-badge>
    </ship-list-tile>
    <ship-list-tile variant="row" interactive selected>
      <ship-icon slot="leading" name="key"></ship-icon>
      API token
      <span slot="meta">Last rotated 14 days ago</span>
      <ship-badge slot="trailing" tone="warning">stale</ship-badge>
    </ship-list-tile>
    <ship-list-tile variant="row" interactive>
      <ship-icon slot="leading" name="user"></ship-icon>
      Account
      <span slot="meta">micah@example.com</span>
    </ship-list-tile>
  </div>
</body>
</html>`

const agentsHtml = (theme) => `<!doctype html>
<html lang="en" data-theme="${theme}"${DARK.has(theme) ? ' class="dark"' : ''}>
<head>
  <meta charset="utf-8" />
  <title>Agents — ${theme}</title>
  <link rel="stylesheet" href="${DAEMON}/api/assets/view-tokens.css" />
  <style>
    html, body { margin: 0; min-height: 100vh; }
    body { background: var(--surface-0); color: var(--foreground); font-family: var(--body-font); padding: 32px; }
    h1 { font-family: var(--heading-font); margin: 0 0 4px; }
    .sub { color: var(--muted-foreground); font-size: var(--text-sm); margin-bottom: 24px; }
    .panel { background: var(--surface-1); border: 1px solid var(--border); border-radius: var(--radius-lg);
             box-shadow: var(--shadow-card-rest); max-width: 880px; padding: 8px; }
    .row { display: flex; align-items: center; gap: 12px; padding: 14px 14px;
           border-bottom: 1px solid var(--border-subtle); }
    .row:last-child { border-bottom: 0; }
    .row .name { font-weight: 600; flex: 0 0 auto; min-width: 180px; font-family: var(--heading-font); }
    .row .meta { color: var(--muted-foreground); font-size: var(--text-sm); flex: 1; }
    .dot { width: 8px; height: 8px; border-radius: 999px; background: var(--success); box-shadow: 0 0 0 3px color-mix(in oklch, var(--success) 30%, transparent); }
    .dot.idle { background: var(--muted-foreground); box-shadow: none; }
  </style>
</head>
<body>
  <h1>Agents</h1>
  <div class="sub">Live actors and their recent activity.</div>
  <div class="panel">
    <div class="row"><span class="dot"></span><span class="name">pipeline.composer</span><span class="meta">Composing pipeline-spec → 3 steps</span><ship-badge tone="primary">running</ship-badge></div>
    <div class="row"><span class="dot"></span><span class="name">chat.loop</span><span class="meta">Awaiting reply on chat-1042</span><ship-badge tone="info">idle</ship-badge></div>
    <div class="row"><span class="dot idle"></span><span class="name">git.observer</span><span class="meta">No diff changes since 2 min ago</span><ship-badge tone="default">quiet</ship-badge></div>
    <div class="row"><span class="dot"></span><span class="name">events.tail</span><span class="meta">42 events in last minute</span><ship-badge tone="success">healthy</ship-badge></div>
    <div class="row"><span class="dot idle"></span><span class="name">canvas.worker</span><span class="meta">Disconnected — reconnecting</span><ship-badge tone="warning">reconnect</ship-badge></div>
  </div>
</body>
</html>`

const launcherHtml = (theme) => `<!doctype html>
<html lang="en" data-theme="${theme}"${DARK.has(theme) ? ' class="dark"' : ''}>
<head>
  <meta charset="utf-8" />
  <title>Launcher — ${theme}</title>
  <link rel="stylesheet" href="${DAEMON}/api/assets/view-tokens.css" />
  <style>
    html, body { margin: 0; min-height: 100vh; }
    body { background: var(--surface-0); color: var(--foreground); font-family: var(--body-font); }
    .scrim { position: fixed; inset: 0; background: var(--scrim, color-mix(in oklch, black 36%, transparent)); }
    .modal { position: fixed; left: 50%; top: 12vh; transform: translateX(-50%);
             width: min(640px, 92vw); background: var(--popover); color: var(--popover-foreground);
             border-radius: var(--radius-xl); box-shadow: var(--shadow-xl); border: 1px solid var(--border);
             overflow: hidden; }
    .input-wrap { padding: 14px 18px; border-bottom: 1px solid var(--border-subtle); }
    .input { width: 100%; box-sizing: border-box; padding: 10px 14px; border-radius: var(--radius-md);
             border: 1px solid var(--border); background: var(--surface-1); color: var(--foreground);
             font-family: var(--body-font); font-size: var(--text-sm); }
    .group { padding: 8px 0; }
    .group h4 { margin: 8px 18px; color: var(--muted-foreground); font-family: var(--label-font);
                font-weight: var(--label-weight); letter-spacing: var(--label-tracking);
                font-size: var(--text-xs); text-transform: uppercase; }
    .item { display: flex; align-items: center; gap: 12px; padding: 10px 18px; cursor: pointer; }
    .item:hover { background: color-mix(in oklch, var(--foreground) var(--hover-strength, 6%), transparent); }
    .item.active { background: color-mix(in oklch, var(--primary) var(--selected-strength, 10%), transparent); }
    .item .name { flex: 1; }
    .item .hint { color: var(--muted-foreground); font-family: var(--mono-font); font-size: var(--text-xs); }
    .footer { padding: 10px 18px; border-top: 1px solid var(--border-subtle);
              color: var(--muted-foreground); font-size: var(--text-xs); display: flex; gap: 18px; }
    .footer kbd { font-family: var(--mono-font); padding: 2px 6px; background: var(--surface-2);
                  border-radius: var(--radius-sm); border: 1px solid var(--border); }
  </style>
</head>
<body>
  <div class="scrim"></div>
  <div class="modal">
    <div class="input-wrap"><input class="input" value="" placeholder="Type a command or search…" autofocus /></div>
    <div class="group">
      <h4>Suggested</h4>
      <div class="item active">
        <ship-icon name="rocket"></ship-icon>
        <span class="name">Run pipeline…</span>
        <span class="hint">⌘ R</span>
      </div>
      <div class="item">
        <ship-icon name="bell"></ship-icon>
        <span class="name">Open events</span>
        <span class="hint">⌘ E</span>
      </div>
      <div class="item">
        <ship-icon name="user"></ship-icon>
        <span class="name">Switch agent</span>
        <span class="hint">⌘ A</span>
      </div>
    </div>
    <div class="group">
      <h4>Recent</h4>
      <div class="item"><ship-icon name="git-branch"></ship-icon><span class="name">git: view-polish</span><span class="hint">2m ago</span></div>
      <div class="item"><ship-icon name="file"></ship-icon><span class="name">Open spec: arc-actors-and-events</span><span class="hint">12m ago</span></div>
    </div>
    <div class="footer"><span><kbd>↵</kbd> select</span><span><kbd>esc</kbd> close</span><span><kbd>⌘ K</kbd> reopen</span></div>
  </div>
</body>
</html>`

const browser = await chromium.launch()
const ctx = await browser.newContext({ viewport: { width: 1400, height: 1100 }, deviceScaleFactor: 1 })
const page = await ctx.newPage()
const consoleLog = []
page.on('console', (m) => consoleLog.push(`[${m.type()}] ${m.text()}`))
page.on('pageerror', (e) => consoleLog.push(`[pageerror] ${e.message}`))

async function shoot(name, html) {
  await page.setContent(html, { waitUntil: 'domcontentloaded' })
  // Load the ship-elements bundle from disk to avoid CORS on null origin.
  await page.addScriptTag({ path: BUNDLE_PATH, type: 'module' })
  await page.waitForTimeout(1200)
  await page.screenshot({ path: `${OUT}/${name}.png`, fullPage: false })
  console.log('captured', name)
}

for (const t of THEMES) {
  await shoot(`gallery-${t}`, galleryHtml(t))
  await shoot(`agents-${t}`, agentsHtml(t))
  await shoot(`launcher-${t}`, launcherHtml(t))
}

await writeFile(`${OUT}/console.log.txt`, consoleLog.join('\n'))
await browser.close()
console.log('done')
