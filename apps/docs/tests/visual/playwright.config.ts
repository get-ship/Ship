/**
 * Visual baselines for the docs site.
 *
 * Separate from tests/e2e/visual/ (which targets the daemon-served
 * views): this suite targets the built static docs site at
 * apps/docs/dist/, served via a Playwright `webServer` block so the
 * spec doesn't depend on a hand-started python http.server.
 *
 * Baselines committed under apps/docs/tests/visual/baselines/. Run
 * via `bun --filter @ship/docs visual` (see package.json) or
 * directly with `npx playwright test --config tests/visual/...`.
 */
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig, devices } from '@playwright/test'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const PORT = Number(process.env.DOCS_VISUAL_PORT ?? 5180)

export default defineConfig({
  testDir: __dirname,
  testMatch: /.*\.spec\.ts$/,
  timeout: 60_000,
  expect: {
    timeout: 10_000,
    toHaveScreenshot: {
      // Generous-but-not-permissive: tutorial visuals include the
      // status dot which transitions colour, and the code editor
      // selection caret. The CSS in render-helpers disables both,
      // but leaving a small ratio buffer absorbs sub-pixel
      // antialiasing differences between local + CI.
      maxDiffPixelRatio: 0.01,
      animations: 'disabled',
      caret: 'hide',
      threshold: 0.2,
    },
  },
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [
    ['list'],
    ['html', { outputFolder: path.join(__dirname, 'report'), open: 'never' }],
  ],
  outputDir: path.join(__dirname, 'output'),
  snapshotDir: path.join(__dirname, 'baselines'),
  snapshotPathTemplate: '{snapshotDir}/{arg}{ext}',
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    trace: 'on-first-retry',
  },
  webServer: {
    // Serve the built dist directly — no Astro dev server, no
    // Vite. Production-shape rendering, matching what a user sees
    // on a published docs site.
    command: `python3 -m http.server --bind 127.0.0.1 --directory dist ${PORT}`,
    cwd: path.resolve(__dirname, '../..'),
    port: PORT,
    timeout: 10_000,
    reuseExistingServer: true,
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'], viewport: { width: 1440, height: 900 } } },
  ],
})
