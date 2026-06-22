/**
 * Hermetic daemon bootstrap for the full-loop e2e spec.
 *
 * Unlike `_helpers.ts` (which only *detects* an already-running `just dev`
 * daemon), this module BOOTS a fresh `shipd` against a throwaway SHIP_HOME so
 * a spec can prove the read + mutation loop end-to-end without polluting the
 * operator's prod kernel state. The contract:
 *
 *   - SHIP_HOME  → a fresh `mkdtemp` dir. ALL daemon kernel state (the settings
 *     table this spec mutates, the per-worktree daemon dir, sockets) lands
 *     here and is removed on teardown. This is the surface the prod-guard
 *     protects, so we assert it is NOT ~/.ship before spawning.
 *   - workspace  → the repo root. The shell + views + baked `.deck` assets the
 *     daemon serves are THIS checkout's build; they cannot be relocated to a
 *     temp dir. Scoping to the repo reads its (read-only) project config; it
 *     never writes prod state, which lives under SHIP_HOME.
 *
 * The daemon runs in `--mode production` so deck serves the baked shell from
 * `.deck/` (the dev shell route expects a vite HMR server we don't run here),
 * and `--dev` for relaxed auth + auto-pair.
 */
import { spawn, spawnSync, type ChildProcess } from 'node:child_process'
import { createServer } from 'node:net'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { assertNotProdHome } from '../_shared/prod-guard.mjs'

/** Repo root — the workspace the hermetic daemon scopes to (serves the shell). */
export const REPO_ROOT = path.resolve(__dirname, '..', '..')
const SHIPD_BIN = path.join(REPO_ROOT, 'target', 'debug', 'shipd')

export interface DaemonHandle {
  baseUrl: string
  port: number
  token: string
  /** The temp SHIP_HOME — proves teardown leaves nothing under ~/.ship. */
  home: string
  dispose: () => Promise<void>
}

/** Ask the OS for a free TCP port (bind to :0, read the assignment, release). */
function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const srv = createServer()
    srv.on('error', reject)
    srv.listen(0, '127.0.0.1', () => {
      const addr = srv.address()
      if (addr && typeof addr === 'object') {
        const p = addr.port
        srv.close(() => resolve(p))
      } else {
        srv.close(() => reject(new Error('no port assigned')))
      }
    })
  })
}

/** Build `shipd` only when the debug binary is absent (CI / clean tree). */
function ensureBinary(): void {
  if (fs.existsSync(SHIPD_BIN)) return
  const r = spawnSync('cargo', ['build', '-p', 'shipd'], {
    cwd: REPO_ROOT,
    stdio: 'inherit',
  })
  if (r.status !== 0) throw new Error('cargo build -p shipd failed')
  if (!fs.existsSync(SHIPD_BIN)) throw new Error(`shipd not at ${SHIPD_BIN} after build`)
}

async function waitForHealth(baseUrl: string, deadlineMs: number): Promise<void> {
  const deadline = Date.now() + deadlineMs
  let lastErr = 'never attempted'
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`${baseUrl}/api/health`, { signal: AbortSignal.timeout(1000) })
      if (res.ok) return
      lastErr = `health ${res.status}`
    } catch (e) {
      lastErr = e instanceof Error ? e.message : String(e)
    }
    await new Promise((r) => setTimeout(r, 200))
  }
  throw new Error(`daemon /api/health never went green within ${deadlineMs}ms (last: ${lastErr})`)
}

/**
 * Boot a hermetic daemon and wait for it to answer /api/health. Throws with an
 * honest message (never a silent hang) if the binary, port file, or health
 * check fails. `bootMs` bounds the wait; default 60s covers a cold daemon.
 */
export async function bootDaemon(opts: { bootMs?: number } = {}): Promise<DaemonHandle> {
  ensureBinary()

  const home = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), 'ship-e2e-'))
  // Hard refusal: the loop spec mutates kernel settings. Never against ~/.ship.
  assertNotProdHome(home)

  const port = await freePort()
  const baseUrl = `http://127.0.0.1:${port}`
  const logPath = path.join(home, 'shipd.log')
  const log = fs.openSync(logPath, 'a')

  const child: ChildProcess = spawn(
    SHIPD_BIN,
    ['--dev', '--mode', 'production', '--workspace', REPO_ROOT, '--port', String(port)],
    { env: { ...process.env, SHIP_HOME: home }, stdio: ['ignore', log, log], detached: false },
  )
  // Throwing inside the event callback would crash the worker process instead
  // of failing the test; record it and let the health wait surface it.
  let spawnErr: Error | null = null
  child.on('error', (e) => {
    spawnErr = new Error(`failed to spawn shipd: ${e.message}`)
  })

  const dispose = async (): Promise<void> => {
    if (child.pid && !child.killed) {
      child.kill('SIGTERM')
      const deadline = Date.now() + 5000
      while (child.exitCode === null && Date.now() < deadline) {
        await new Promise((r) => setTimeout(r, 100))
      }
      if (child.exitCode === null) child.kill('SIGKILL')
    }
    try {
      fs.closeSync(log)
    } catch { /* already closed */ }
    fs.rmSync(home, { recursive: true, force: true })
  }

  try {
    await waitForHealth(baseUrl, opts.bootMs ?? 60_000)
  } catch (e) {
    const tail = fs.existsSync(logPath) ? fs.readFileSync(logPath, 'utf8').slice(-2000) : '(no log)'
    await dispose()
    const spawnNote = spawnErr ? `\n${(spawnErr as Error).message}` : ''
    throw new Error(`${(e as Error).message}${spawnNote}\n--- shipd.log tail ---\n${tail}`)
  }

  // The per-worktree daemon dir under the temp home holds the bearer token.
  const token = readToken(home)
  return { baseUrl, port, token, home, dispose }
}

/** Read the daemon bearer token from the per-worktree daemon dir's token file. */
function readToken(home: string): string {
  const daemonsRoot = path.join(home, 'machine', 'daemons')
  const dirs = fs.existsSync(daemonsRoot) ? fs.readdirSync(daemonsRoot) : []
  for (const d of dirs) {
    const tf = path.join(daemonsRoot, d, 'token')
    if (fs.existsSync(tf)) return fs.readFileSync(tf, 'utf8').trim()
  }
  throw new Error(`no token file under ${daemonsRoot}`)
}

// ── SettingsService over Connect (HTTP+JSON) ────────────────────────────────
// Read-back / seed path that bypasses the UI, so the spec can prove daemon
// state independently of what the browser rendered.

const RUNTIME_SCOPE = 'SETTING_SCOPE_RUNTIME'

async function settingsRpc(
  h: DaemonHandle,
  method: 'Get' | 'Set',
  body: Record<string, unknown>,
): Promise<{ valueJson?: string }> {
  const res = await fetch(`${h.baseUrl}/ship.ops.v1.SettingsService/${method}`, {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${h.token}`,
      'Content-Type': 'application/json',
      'Connect-Protocol-Version': '1',
    },
    body: JSON.stringify(body),
  })
  if (res.status === 404 && method === 'Get') {
    // No stored row for the key — the daemon 404s rather than echoing a default.
    return { valueJson: undefined }
  }
  if (!res.ok) {
    throw new Error(`SettingsService/${method} → ${res.status}: ${await res.text()}`)
  }
  return (await res.json()) as { valueJson?: string }
}

/** Read a RUNTIME setting straight from the daemon, JSON-decoded. `null` if unset. */
export async function getRuntimeSetting<T = unknown>(
  h: DaemonHandle,
  key: string,
): Promise<T | null> {
  const out = await settingsRpc(h, 'Get', { key, scope: RUNTIME_SCOPE })
  if (!out.valueJson) return null
  const decoded = Buffer.from(out.valueJson, 'base64').toString('utf8')
  if (decoded.length === 0) return null
  return JSON.parse(decoded) as T
}

/** Write a RUNTIME setting straight to the daemon (UI-independent seed). */
export async function setRuntimeSetting(
  h: DaemonHandle,
  key: string,
  value: unknown,
): Promise<void> {
  const valueJson = Buffer.from(JSON.stringify(value), 'utf8').toString('base64')
  await settingsRpc(h, 'Set', { key, valueJson, scope: RUNTIME_SCOPE })
}
