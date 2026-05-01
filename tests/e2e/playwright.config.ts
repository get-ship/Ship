import { defineConfig, devices } from '@playwright/test'
import { getDaemonUrl } from './_helpers'

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
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
})
