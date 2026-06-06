// chrome/terminal-bridge.ts — connects the dock's <sui-terminal> to a
// real shell PTY session on the daemon.
//
// Lifecycle: bridge once on first activation (dock open or terminal-tab
// activate), tear down on dock close. Keystrokes and resize events from
// xterm forward to the daemon; SSE `bytes` events (base64-encoded raw
// PTY output) decode and write straight into the terminal engine.
//
// Backend contract (apps/shipd/src/sessions/api):
//   POST /api/v1/sessions/spawn  { workspace_id, kind:'shell' } → { session_id }
//   GET  /api/v1/sessions/{id}/stream                            ← SSE bytes (base64)
//   POST /api/v1/sessions/{id}/input  { data }
//   POST /api/v1/sessions/{id}/resize { cols, rows }
//   DELETE /api/v1/sessions/{id}

interface TerminalEl extends HTMLElement {
  write?: (data: string | Uint8Array) => void
  writeln?: (data: string) => void
  clear?: () => void
  terminal?: { cols: number; rows: number } | null
}

import { ops } from '../ops'

let bridging: Promise<void> | null = null
let sessionId: string | null = null
let sse: EventSource | null = null
let dataListener: ((e: Event) => void) | null = null
let resizeListener: ((e: Event) => void) | null = null
let readyListener: ((e: Event) => void) | null = null
let boundEl: TerminalEl | null = null
let pendingBytes: Uint8Array[] = []
let pendingInput: string[] = []
let pendingResize: { cols: number; rows: number } | null = null
let termReady = false

function findTerminal(): TerminalEl | null {
  return document.querySelector<TerminalEl>('sui-terminal[slot="terminal"]')
    ?? document.getElementById('ship-panel-dock')?.querySelector<TerminalEl>(':scope > sui-terminal')
    ?? null
}

async function getActiveWorkspaceId(): Promise<string> {
  // Fallback to 'default' if no project is active — the daemon's
  // work_dir_for() ignores workspace_id and uses cwd anyway, so the
  // shell still spawns in a sensible place. Pre-fix, a missing active
  // project caused the whole spawn to short-circuit silently.
  try {
    const id = (await ops.space.active({})).active?.id
    if (id) return id
  } catch { /* network error → fall through */ }
  return 'default'
}

async function spawnShell(): Promise<{ id: string } | { error: string }> {
  const workspace_id = await getActiveWorkspaceId()
  try {
    const r = await fetch('/api/v1/sessions/spawn', {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ workspace_id, kind: 'shell' }),
    })
    if (!r.ok) {
      const text = await r.text().catch(() => '')
      return { error: `HTTP ${r.status}${text ? `: ${text}` : ''}` }
    }
    const b = await r.json() as { session_id?: string }
    if (!b.session_id) return { error: 'spawn response missing session_id' }
    return { id: b.session_id }
  } catch (e) {
    return { error: e instanceof Error ? e.message : String(e) }
  }
}

function decodeBase64(b64: string): Uint8Array {
  // EventSource yields data as a string; base64-decode to raw bytes so
  // PTY escape sequences (including bare \r used for cursor returns)
  // arrive byte-exact rather than line-folded by SSE.
  if (!b64) return new Uint8Array(0)
  const bin = atob(b64)
  const out = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i += 1) out[i] = bin.charCodeAt(i)
  return out
}

function writeToTerm(bytes: Uint8Array): void {
  if (!boundEl) return
  if (!termReady) {
    pendingBytes.push(bytes)
    return
  }
  try { boundEl.write?.(bytes) } catch { /* ignore xterm internal */ }
}

function flushPending(): void {
  if (!boundEl) return
  for (const chunk of pendingBytes) {
    try { boundEl.write?.(chunk) } catch { /* ignore */ }
  }
  pendingBytes = []
}

function openStream(id: string): EventSource {
  const es = new EventSource(`/api/v1/sessions/${encodeURIComponent(id)}/stream`)
  const handle = (e: Event): void => {
    const data = (e as MessageEvent).data
    if (typeof data !== 'string') return
    writeToTerm(decodeBase64(data))
  }
  es.addEventListener('bytes', handle)
  es.onerror = () => {
    // EventSource auto-reconnects on transient drops; surface a brief
    // hint so the user knows why output paused. The next reconnect
    // re-runs the replay block on the server side.
    writeToTerm(new TextEncoder().encode('\r\n[stream disconnected — reconnecting]\r\n'))
  }
  return es
}

async function sendInput(id: string, data: string): Promise<void> {
  try {
    await fetch(`/api/v1/sessions/${encodeURIComponent(id)}/input`, {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ data }),
    })
  } catch { /* fire-and-forget */ }
}

async function sendResize(id: string, cols: number, rows: number): Promise<void> {
  // Clamp to sane PTY bounds. xterm.js + FitAddon can report ridiculous
  // values (cols=131072, rows=1) before the container actually has
  // layout — pushing those into the PTY makes the kernel reject the
  // ioctl with `your <W>x<H> screen size is bogus. expect trouble` and
  // subsequent stdin writes silently never reach the shell.
  if (!Number.isFinite(cols) || !Number.isFinite(rows)) return
  if (cols < 2 || cols > 1000 || rows < 2 || rows > 500) return
  try {
    await fetch(`/api/v1/sessions/${encodeURIComponent(id)}/resize`, {
      method: 'POST',
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ cols, rows }),
    })
  } catch { /* fire-and-forget */ }
}

export async function bridgeDockTerminal(): Promise<void> {
  if (bridging) return bridging
  if (sessionId) return
  const termEl = findTerminal()
  if (!termEl) return
  boundEl = termEl
  termReady = false
  pendingBytes = []
  pendingInput = []
  pendingResize = null

  // Wire input + resize listeners eagerly — before /spawn returns —
  // so any keystrokes the user lands on the prompt while the PTY is
  // still booting get buffered and flushed on session-ready. Pre-fix,
  // the listener attached only after spawn so the user's first chars
  // could be silently dropped.
  dataListener = (e: Event) => {
    const d = (e as CustomEvent<{ data?: string }>).detail?.data
    if (typeof d !== 'string') return
    if (sessionId) void sendInput(sessionId, d)
    else pendingInput.push(d)
  }
  resizeListener = (e: Event) => {
    const detail = (e as CustomEvent<{ cols?: number; rows?: number }>).detail
    if (!detail) return
    if (sessionId) {
      void sendResize(sessionId, detail.cols ?? 0, detail.rows ?? 0)
    } else {
      pendingResize = { cols: detail.cols ?? 0, rows: detail.rows ?? 0 }
    }
  }
  termEl.addEventListener('sui-terminal-data', dataListener)
  termEl.addEventListener('sui-terminal-resize', resizeListener)

  // If the engine hasn't mounted yet (script island still loading), wait
  // for the `sui-terminal-ready` CustomEvent before flushing buffered
  // bytes — writing to an unupgraded element silently no-ops because
  // the `write` method only exists post-controller-mount.
  readyListener = () => {
    termReady = true
    flushPending()
    // Push our initial cols/rows so the PTY isn't stuck at 80×24 when
    // the dock is taller / narrower than that default.
    if (sessionId && boundEl?.terminal) {
      void sendResize(sessionId, boundEl.terminal.cols, boundEl.terminal.rows)
    }
  }
  termEl.addEventListener('sui-terminal-ready', readyListener, { once: true })
  // Belt-and-braces: if the engine already mounted before we attached
  // (race on second bridge call), inspect `el.terminal` and flush
  // immediately so we don't deadlock on a ready event that already fired.
  if (termEl.terminal) readyListener({} as Event)

  bridging = (async (): Promise<void> => {
    const outcome = await spawnShell()
    if ('error' in outcome) {
      const msg = `\r\n[no shell — spawn failed: ${outcome.error}]\r\n`
      writeToTerm(new TextEncoder().encode(msg))
      bridging = null
      return
    }
    sessionId = outcome.id
    sse = openStream(outcome.id)
    // Flush anything the user typed / any resize that landed while the
    // spawn round-trip was in flight.
    for (const d of pendingInput) void sendInput(outcome.id, d)
    pendingInput = []
    // Explicit annotations: pendingResize/boundEl are reassigned only inside
    // listener closures, which TS's CFA can't order before this IIFE — so it
    // narrows the captured `let`s to their last in-scope value (null) and the
    // truthy checks below collapse to `never`. Annotating restores the real type.
    const queuedResize = pendingResize as { cols: number; rows: number } | null
    const termDims = boundEl?.terminal as { cols: number; rows: number } | null | undefined
    if (queuedResize) {
      void sendResize(outcome.id, queuedResize.cols, queuedResize.rows)
      pendingResize = null
    } else if (termDims) {
      void sendResize(outcome.id, termDims.cols, termDims.rows)
    }
    bridging = null
  })()
  return bridging
}

export async function disposeDockTerminal(): Promise<void> {
  if (!sessionId && !bridging) return
  if (bridging) {
    try { await bridging } catch { /* ignore */ }
  }
  const id = sessionId
  sessionId = null
  const el = boundEl
  boundEl = null
  termReady = false
  pendingBytes = []
  pendingInput = []
  pendingResize = null
  if (el && dataListener) el.removeEventListener('sui-terminal-data', dataListener)
  if (el && resizeListener) el.removeEventListener('sui-terminal-resize', resizeListener)
  if (el && readyListener) el.removeEventListener('sui-terminal-ready', readyListener)
  dataListener = null
  resizeListener = null
  readyListener = null
  try { el?.clear?.() } catch { /* ignore */ }
  sse?.close()
  sse = null
  if (id) {
    try {
      await fetch(`/api/v1/sessions/${encodeURIComponent(id)}`, {
        method: 'DELETE',
        credentials: 'same-origin',
      })
    } catch { /* best-effort */ }
  }
}
