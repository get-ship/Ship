/**
 * Visual baseline suite — configured separately from tests/e2e so the
 * baseline images, output dir, and snapshot threshold are scoped here.
 *
 * Pre-req: daemon running. Discovery via `helpers/daemon.ts` (SHIPD_URL
 * env, then per-worktree port file, then localhost:9392 fallback).
 */
import path from 'node:path'
import { defineConfig, devices } from '@playwright/test'
import { getDaemonUrl } from './helpers/daemon'

export default defineConfig({
  testDir: __dirname,
  testMatch: /.*\.spec\.ts$/,
  // Generous: component harness routes plus paint settling.
  timeout: 60_000,
  expect: {
    // Per-spec override is preferred; this is the global floor.
    timeout: 10_000,
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.01,
      animations: 'disabled',
      caret: 'hide',
      threshold: 0.2,
    },
  },
  // Single worker — daemon shares one DB, parallel tabs can race on the
  // same view's data subscriptions and produce flaky pixels.
  workers: 1,
  retries: 0,
  fullyParallel: false,
  reporter: [
    ['list'],
    ['html', { outputFolder: path.join(__dirname, 'report'), open: 'never' }],
  ],
  outputDir: path.join(__dirname, 'output'),
  snapshotDir: path.join(__dirname, 'baselines'),
  snapshotPathTemplate: '{snapshotDir}/{arg}{ext}',
  use: {
    baseURL: getDaemonUrl(),
    viewport: { width: 1280, height: 800 },
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    reducedMotion: 'reduce',
    colorScheme: 'light',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
})
