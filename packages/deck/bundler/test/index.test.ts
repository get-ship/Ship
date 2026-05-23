// Bun.test suite for the bundler worker protocol.
//
// Tests spawn the worker as a child process and communicate via stdin/stdout
// using the same JSON-line protocol that shipd uses.

import { describe, test, expect, beforeAll, afterAll } from 'bun:test'
import { resolve, join } from 'node:path'
import { mkdtempSync, writeFileSync, mkdirSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'

const workerPath = resolve(import.meta.dir, '../index.ts')
const repoRoot = resolve(import.meta.dir, '../../../..')

// ── Test helpers ──────────────────────────────────────────────────────────────

type WorkerProc = {
  proc: ReturnType<typeof Bun.spawn>
  send: (msg: object) => void
  next: () => Promise<object>
  kill: () => void
}

async function spawnWorker(opts?: {
  env?: Record<string, string>
  args?: string[]
}): Promise<WorkerProc> {
  // Use the bun binary that's running this test rather than searching PATH —
  // CI / fresh-clone environments may not have bun globally installed but
  // they do have the one that just spawned the test runner.
  const bunBin = process.execPath
  const proc = Bun.spawn({
    cmd: [bunBin, 'run', workerPath, ...(opts?.args ?? [])],
    stdin: 'pipe',
    stdout: 'pipe',
    stderr: 'pipe',
    cwd: repoRoot,
    env: { ...process.env, SHIP_REPO_ROOT: repoRoot, ...(opts?.env ?? {}) },
  })

  const lines: string[] = []
  const waiters: Array<(line: string) => void> = []

  // Stream stdout line by line
  ;(async () => {
    const reader = proc.stdout.getReader()
    const dec = new TextDecoder()
    let buf = ''
    while (true) {
      const { value, done } = await reader.read()
      if (done) break
      buf += dec.decode(value, { stream: true })
      let nl: number
      while ((nl = buf.indexOf('\n')) !== -1) {
        const line = buf.slice(0, nl).trim()
        buf = buf.slice(nl + 1)
        if (!line) continue
        if (waiters.length > 0) {
          waiters.shift()!(line)
        } else {
          lines.push(line)
        }
      }
    }
  })()

  const next = (): Promise<object> =>
    new Promise((resolve) => {
      if (lines.length > 0) {
        resolve(JSON.parse(lines.shift()!))
      } else {
        waiters.push((line) => resolve(JSON.parse(line)))
      }
    })

  const send = (msg: object) => {
    // Bun.spawn's stdin is a FileSink — call .write() + .flush() directly.
    // (No getWriter() / WritableStream contract.)
    const sink = proc.stdin as any
    sink.write(JSON.stringify(msg) + '\n')
    sink.flush()
  }

  const kill = () => proc.kill()

  // Wait for the initial 'ready' event
  const ready = await Promise.race([
    next(),
    new Promise<never>((_, reject) => setTimeout(() => reject(new Error('worker did not emit ready')), 10_000)),
  ])
  expect((ready as any).kind).toBe('ready')

  return { proc, send, next, kill }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe('bundler worker', () => {
  test('build sui-runtime target → bundle response with non-empty outPath', async () => {
    const worker = await spawnWorker()
    try {
      worker.send({ id: 'r1', kind: 'build', target: 'sui-runtime' })

      const resp = await Promise.race([
        worker.next(),
        new Promise<never>((_, rej) => setTimeout(() => rej(new Error('timeout')), 60_000)),
      ]) as any

      expect(resp.id).toBe('r1')
      expect(resp.kind).toBe('bundle')
      expect(resp.target).toBe('sui-runtime')
      expect(typeof resp.bytes).toBe('number')
      expect(resp.bytes).toBeGreaterThan(0)
      expect(resp.outPath).toBeTruthy()
      expect(resp.manifest).toBeDefined()
    } finally {
      worker.kill()
    }
  }, 90_000)

  test('build unknown target → error response', async () => {
    const worker = await spawnWorker()
    try {
      worker.send({ id: 'e1', kind: 'build', target: 'does-not-exist' })
      const resp = await worker.next() as any
      expect(resp.id).toBe('e1')
      expect(resp.kind).toBe('error')
      expect(resp.message).toMatch(/unknown target/)
    } finally {
      worker.kill()
    }
  }, 15_000)

  test('watch_start and watch_stop → ack responses', async () => {
    const worker = await spawnWorker()
    try {
      worker.send({ id: 'w1', kind: 'watch_start' })
      const startAck = await worker.next() as any
      expect(startAck.id).toBe('w1')
      expect(startAck.kind).toBe('ack')

      worker.send({ id: 'w2', kind: 'watch_stop' })
      const stopAck = await worker.next() as any
      expect(stopAck.id).toBe('w2')
      expect(stopAck.kind).toBe('ack')
    } finally {
      worker.kill()
    }
  }, 15_000)

  test('shutdown → ack then process exits', async () => {
    const worker = await spawnWorker()

    worker.send({ id: 's1', kind: 'shutdown' })
    const ack = await worker.next() as any
    expect(ack.id).toBe('s1')
    expect(ack.kind).toBe('ack')

    const code = await Promise.race([
      worker.proc.exited,
      new Promise<never>((_, rej) => setTimeout(() => rej(new Error('process did not exit')), 5_000)),
    ])
    expect(code).toBe(0)
  }, 15_000)

  test('--workspace overlay: edit `<workspace>/views/<name>/page.sui` → module_changed', async () => {
    // End-user scenario: shipd is attached to a project outside the
    // contributor monorepo. The bundler is still rooted at the monorepo
    // for built-ins (sui-runtime, etc.) but must additionally watch the
    // workspace's `views/` so HMR fires for the project the daemon serves.
    const wsDir = mkdtempSync(join(tmpdir(), 'bundler-ws-'))
    const viewDir = join(wsDir, 'views', 'probe')
    const pageSui = join(viewDir, 'page.sui')
    mkdirSync(viewDir, { recursive: true })
    // Minimal SUI template — no controller, no data block, just a static
    // element so sui-compiler emits a valid bundle.
    writeFileSync(pageSui, '<div class="probe">probe</div>\n')

    const worker = await spawnWorker({ args: ['--workspace', wsDir] })
    try {
      worker.send({ id: 'w1', kind: 'watch_start' })
      const ack = await worker.next() as any
      expect(ack.kind).toBe('ack')

      // Touch the file. mtime change + node:fs watch fires `change`.
      writeFileSync(pageSui, '<div class="probe">probe v2</div>\n')

      const event = await Promise.race([
        (async () => {
          while (true) {
            const msg = (await worker.next()) as any
            if (msg.kind === 'module_changed' && msg.target === 'views/probe/view') return msg
            if (msg.kind === 'compile_error' && msg.target === 'views/probe/view') {
              throw new Error(`compile_error for views/probe/view: ${msg.message}`)
            }
          }
        })(),
        new Promise<never>((_, rej) =>
          setTimeout(() => rej(new Error('module_changed timeout')), 30_000),
        ),
      ]) as any

      expect(event.kind).toBe('module_changed')
      expect(event.target).toBe('views/probe/view')
      expect(event.bytes).toBeGreaterThan(0)
    } finally {
      worker.kill()
      rmSync(wsDir, { recursive: true, force: true })
    }
  }, 60_000)

  test('edit a .sui source → module_changed event arrives within 2s', async () => {
    // Create a throwaway .sui view in a temp dir and point a minimal worker
    // at a repo with just the sui-runtime target modified to watch our temp file.
    //
    // This test requires the daemon to actually be watching, so we use a real
    // source file in the views tree that already exists.
    const suiFile = resolve(repoRoot, 'views/git/page.sui')
    const worker = await spawnWorker()

    try {
      // Start watching
      worker.send({ id: 'w1', kind: 'watch_start' })
      const ack = await worker.next() as any
      expect(ack.kind).toBe('ack')

      // Touch the file (append a whitespace comment to modify mtime without
      // changing semantics — the bundler only checks mtime for precompile)
      const original = await Bun.file(suiFile).text()
      await Bun.write(suiFile, original + '\n')

      const event = await Promise.race([
        (async () => {
          let msg: any
          while (true) {
            msg = await worker.next()
            if ((msg as any).kind === 'module_changed') return msg
          }
        })(),
        new Promise<never>((_, rej) => setTimeout(() => rej(new Error('module_changed timeout')), 2_000)),
      ]) as any

      expect(event.kind).toBe('module_changed')
      expect(typeof event.target).toBe('string')
      expect(event.bytes).toBeGreaterThan(0)

      // Restore the file
      await Bun.write(suiFile, original)
    } finally {
      worker.kill()
    }
  }, 60_000)
})
