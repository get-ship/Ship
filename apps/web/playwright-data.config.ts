import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './e2e-playwright',
  testMatch: ['data-view.spec.ts', 'ssr-screenshot.spec.ts'],
  timeout: 30_000,
  use: {
    baseURL: 'http://127.0.0.1:9315',
  },
})
