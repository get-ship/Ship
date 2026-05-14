/**
 * Shared helpers for shell2 (vite dev server) e2e specs.
 *
 * shell2 is the SUI-native shell served by its own vite dev server, on a
 * port derived from this worktree's daemon port (vite = daemon + offset,
 * per apps/shell2/vite.config.ts). Specs that drive shell2 in a real
 * browser skip themselves when the dev server isn't reachable, so
 * `bun run test:e2e` keeps passing on machines that only run the daemon.
 */
import path from 'node:path'
import fs from 'node:fs'
import crypto from 'node:crypto'
import os from 'node:os'
import { test } from '@playwright/test'

/** Resolve shell2's vite dev-server base URL for this worktree. */
export function resolveShell2Url(): string {
  if (process.env.SHELL2_URL) return process.env.SHELL2_URL
  const workspaceRoot = path.resolve(__dirname, '..', '..')
  try {
    const real = fs.realpathSync(workspaceRoot)
    const hash = crypto.createHash('sha256').update(real).digest('hex').slice(0, 8)
    const port = fs
      .readFileSync(path.join(os.homedir(), '.ship', 'machine', 'daemons', hash, 'port'), 'utf8')
      .trim()
    const daemon = Number(port)
    if (Number.isFinite(daemon) && daemon >= 9315) {
      return `http://127.0.0.1:${3200 + (daemon - 9315)}`
    }
  } catch { /* fall through */ }
  return 'http://127.0.0.1:3200'
}

export const SHELL2_URL = resolveShell2Url()

/** True when shell2's vite dev server answers on its derived port. */
export async function shell2Reachable(): Promise<boolean> {
  try {
    const res = await fetch(SHELL2_URL, { signal: AbortSignal.timeout(1500) })
    return res.ok || res.status === 304
  } catch {
    return false
  }
}

/**
 * `beforeAll` guard: skip the whole describe block when shell2's dev
 * server isn't up. Call inside `test.describe(...)`.
 */
export function skipUnlessShell2(): void {
  test.beforeAll(async () => {
    if (!(await shell2Reachable())) {
      test.skip(
        true,
        `shell2 vite not reachable at ${SHELL2_URL} — start it with \`bun -F shell2 dev\``,
      )
    }
  })
}
