/**
 * Daemon helpers — port discovery + URL builders for the visual suite.
 *
 * Discovery order:
 *   1. SHIPD_URL env var (CI override)
 *   2. ~/.ship/machine/daemons/<hash>/port — per-worktree daemon launched
 *      by `just dev`. Hash is sha256(realpath(workspace_root))[0..8].
 *   3. Fallback: http://localhost:9392 (current dev default)
 */
import crypto from 'node:crypto'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'

const WORKSPACE_ROOT = path.resolve(__dirname, '../../../..')

export function getDaemonUrl(): string {
  if (process.env.SHIPD_URL) return process.env.SHIPD_URL
  try {
    const root = fs.realpathSync(WORKSPACE_ROOT)
    const hash = crypto.createHash('sha256').update(root).digest('hex').slice(0, 8)
    const portFile = path.join(os.homedir(), '.ship', 'machine', 'daemons', hash, 'port')
    const port = fs.readFileSync(portFile, 'utf8').trim()
    return `http://localhost:${port}`
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
