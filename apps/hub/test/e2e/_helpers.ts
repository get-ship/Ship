/**
 * Shared helpers for Ship UI e2e tests.
 *
 * Port detection order:
 *   1. SHIPD_URL env var (CI / explicit override)
 *   2. ~/.ship/machine/daemons/<hash>/port (devx-04 per-worktree daemon)
 *   3. Fallback: http://localhost:9315
 *
 * The hash is the first 8 hex chars of sha256(realpath(workspace_root)),
 * matching the algorithm in `runtime::home::path_hash`.
 */
import crypto from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'
import os from 'node:os'
import type { Page } from '@playwright/test'

declare global {
  interface Window {
    __suiDispose?: () => void
  }
}

const WORKSPACE_ROOT = path.resolve(__dirname, '../..')

/** Synchronously resolve the daemon base URL. */
export function getDaemonUrl(): string {
  if (process.env.SHIPD_URL) return process.env.SHIPD_URL
  try {
    const root = fs.realpathSync(WORKSPACE_ROOT)
    const hash = crypto.createHash('sha256').update(root).digest('hex').slice(0, 8)
    const portFile = path.join(os.homedir(), '.ship', 'machine', 'daemons', hash, 'port')
    const port = fs.readFileSync(portFile, 'utf8').trim()
    return `http://localhost:${port}`
  } catch {
    return 'http://localhost:9315'
  }
}

/**
 * Detect a running daemon and return its port.
 * Throws if unreachable — tests depend on `just dev` being active.
 */
export async function startDaemon(): Promise<{ port: number; dispose: () => Promise<void> }> {
  const url = getDaemonUrl()
  const parsed = new URL(url)
  const port = parseInt(parsed.port, 10) || 9315
  try {
    const res = await fetch(`${url}/api/health`)
    if (res.ok) return { port, dispose: async () => {} }
  } catch {
    // fall through
  }
  throw new Error(`No daemon at ${url}. Run 'just dev' first.`)
}

const SCREENSHOTS_DIR = path.join(__dirname, 'screenshots')

/** Capture a debug screenshot to tests/e2e/screenshots/<name>.png. */
export async function screenshot(page: Page, name: string): Promise<void> {
  fs.mkdirSync(SCREENSHOTS_DIR, { recursive: true })
  await page.screenshot({ path: path.join(SCREENSHOTS_DIR, `${name}.png`) })
}

/** Wait for attach() to complete (boot script sets window.__suiDispose). */
export async function waitForAttach(page: Page, timeout = 10_000): Promise<void> {
  await page.waitForFunction(
    () => typeof window.__suiDispose !== 'undefined',
    { timeout },
  )
}
