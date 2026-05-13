/**
 * Daemon helpers — port discovery + URL builders for the visual suite.
 *
 * Discovery order:
 *   1. SHIPD_URL env var (CI override).
 *   2. `ship daemon dir` — asks the ship binary for the daemon dir of
 *      the current workspace. Authoritative; matches how `just dev`
 *      computes it. Falls through on any error.
 *   3. ~/.ship/machine/daemons/<hash>/port — last-resort sha256(realpath)
 *      lookup. Note: this can disagree with `ship daemon dir` for git
 *      worktrees that share a .git, so it's the fallback, not primary.
 *   4. http://localhost:9392 (current dev default).
 */
import { execFileSync } from 'node:child_process'
import crypto from 'node:crypto'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'

const WORKSPACE_ROOT = path.resolve(__dirname, '../../../..')

function readPort(daemonDir: string): string {
  return fs.readFileSync(path.join(daemonDir, 'port'), 'utf8').trim()
}

export function getDaemonUrl(): string {
  if (process.env.SHIPD_URL) return process.env.SHIPD_URL
  // Authoritative: ask the ship binary for the daemon dir of this workspace.
  try {
    const dir = execFileSync('ship', ['daemon', 'dir'], {
      cwd: WORKSPACE_ROOT,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim()
    if (dir) return `http://localhost:${readPort(dir)}`
  } catch {
    // ship binary not on PATH or workspace not yet initialized.
  }
  // Fallback: hash-based lookup. Caveat — this disagrees with ship's
  // own hashing for git worktrees that share a .git.
  try {
    const root = fs.realpathSync(WORKSPACE_ROOT)
    const hash = crypto.createHash('sha256').update(root).digest('hex').slice(0, 8)
    return `http://localhost:${readPort(path.join(os.homedir(), '.ship', 'machine', 'daemons', hash))}`
  } catch {
    return 'http://localhost:9392'
  }
}

export function viewUrl(name: string): string {
  return `/sui/${name}`
}

/** Probe daemon health; throws if unreachable. */
export async function assertDaemonReachable(): Promise<void> {
  const url = getDaemonUrl()
  const res = await fetch(`${url}/api/health`).catch(() => null)
  if (!res || !res.ok) {
    throw new Error(`No daemon at ${url}. Run 'just dev' first.`)
  }
}
