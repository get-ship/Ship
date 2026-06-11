import { defineConfig, devices } from '@playwright/test'
import { getDaemonUrl } from './_helpers'

/**
 * Two projects split by environment requirement (CI runs `--project=hermetic`):
 *
 * `hermetic` — specs that own their entire environment (boot a fresh `shipd`
 * against a temp SHIP_HOME, tests/e2e/_daemon.ts). Verified green on a bare
 * machine and on a detached-HEAD checkout; this is the CI surface.
 *   - full-loop.spec.ts
 *
 * `dev-environment` — everything else. Each spec needs a live server this
 * config cannot provide hermetically (verified 2026-06-11: 61 tests fail and
 * 65 skip without one). Per spec, the server it needs:
 *   - agents.spec.ts            dev daemon (`just dev`) serving /agents with workspace state
 *   - counter.spec.ts           dev daemon serving the counter demo view (skips when absent)
 *   - git.spec.ts               dev daemon serving this workspace's live git state
 *   - git-live.spec.ts          dev daemon SSE patch channel over live git state
 *   - studio.spec.ts            dev daemon's UI Studio catalog
 *   - cmdk-create.spec.ts       shell vite dev server (`bun -F shell dev`)
 *   - setup-wizard.spec.ts      shell vite dev server
 *   - shell-agent-detail.spec.ts shell vite dev server
 *   - shell-artifacts.spec.ts   shell vite dev server
 *   - shell-keyboard.spec.ts    shell vite dev server
 *   - shell-rail.spec.ts        shell vite dev server
 *   - shell-settings.spec.ts    shell vite dev server
 *   - shell-skills.spec.ts      shell vite dev server
 *   - shell-views-smoke.spec.ts shell vite dev server
 *   - sui-view-picker.spec.ts   shell vite dev server
 *   - terminal-revive-probe.spec.ts shell vite dev server + a revivable session
 *   - theme-aware-icon.spec.ts  shell vite dev server
 *   - view-authoring-frontend.spec.ts shell vite dev server
 *   - visual/*.spec.ts          dev daemon + screenshot baselines (own config:
 *                               tests/e2e/visual/playwright.config.ts)
 */
const HERMETIC = /full-loop\.spec\.ts/

export default defineConfig({
  testDir: '.',
  timeout: 30_000,
  retries: 0,
  reporter: [
    ['list'],
    ['html', { outputFolder: 'tests/e2e/report', open: 'never' }],
  ],
  use: {
    baseURL: getDaemonUrl(),
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'hermetic',
      testMatch: HERMETIC,
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'dev-environment',
      testIgnore: HERMETIC,
      use: { ...devices['Desktop Chrome'] },
    },
  ],
})
