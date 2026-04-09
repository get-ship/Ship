import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './e2e-playwright',
  timeout: 30_000,
  webServer: {
    command: 'pnpm dev --port 3000',
    port: 3000,
    reuseExistingServer: true,
    timeout: 60_000,
  },
  use: {
    baseURL: 'http://localhost:3000',
  },
  expect: {
    timeout: 15_000,
  },
})
