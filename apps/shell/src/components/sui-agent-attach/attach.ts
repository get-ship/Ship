// chrome/agent-session-attach.ts — attaches a `<sui-terminal>` element to
// a running PTY-backed agent session via the daemon's WebSocket bridge.
//
// Sibling of `terminal-bridge.ts` (shell PTY → dock terminal). Where that
// module owns the dock's terminal tab end-to-end, this one is a transport
// the caller drives — pass a `<sui-terminal>` and a slug, get back a
// disposer. Higher-level surfaces (panel-dock tab, popout window, agent
// detail page) decide WHEN to attach.
//
// Backend contract (apps/shipd/src/routes/agent_sessions_attach.rs):
//   GET /api/v1/agent_sessions/{slug}/attach   (Upgrade: websocket)
//     server → client: binary frames carrying raw PTY bytes
//     client → server: binary frames written as PTY input
//
// Multiple concurrent attaches to the same slug are supported on the
// server side — each gets its own broadcast subscription, input is
// last-writer-wins (same semantics as `tmux attach`).

interface TerminalEl extends HTMLElement {
  write?: (data: string | Uint8Array) => void
  clear?: () => void
  terminal?: { cols: number; rows: number } | null
}

export interface AgentSessionAttachOptions {
  /// Override the WebSocket scheme — defaults to `wss:` on https pages
  /// and `ws:` everywhere else. Tests inject a stub URL via `host`.
  scheme?: 'ws:' | 'wss:'
  /// Override the host portion of the URL. Defaults to window.location.host.
  host?: string
  /// Optional WebSocket constructor — lets unit tests inject a fake.
  /// Defaults to the global `WebSocket`.
  webSocketCtor?: typeof WebSocket
  /// Called once the WS open handshake completes.
  onOpen?: () => void
  /// Called when the WS closes or errors. `reason` is human-readable.
  onDisconnect?: (reason: string) => void
}

export interface AgentSessionAttachment {
  /// Tear down the WS, drop listeners, and stop bridging bytes. Idempotent.
  dispose: () => void
  /// True while the WS is open and bytes are being bridged.
  isConnected: () => boolean
}

/// Connect `termEl` to `/api/v1/agent_sessions/{slug}/attach`. The xterm
/// engine inside `<sui-terminal>` must be mounted before bytes start
/// flowing — the helper buffers server frames until the `sui-terminal-ready`
/// event fires (or until the engine is already mounted, signaled by the
/// presence of `el.terminal`).
export function attachAgentSession(
  termEl: TerminalEl,
  slug: string,
  opts: AgentSessionAttachOptions = {},
): AgentSessionAttachment {
  const scheme = opts.scheme
    ?? (typeof window !== 'undefined' && window.location.protocol === 'https:' ? 'wss:' : 'ws:')
  const host = opts.host ?? (typeof window !== 'undefined' ? window.location.host : 'localhost')
  const url = `${scheme}//${host}/api/v1/agent_sessions/${encodeURIComponent(slug)}/attach`
  const Ctor = opts.webSocketCtor ?? WebSocket

  const pending: Uint8Array[] = []
  let termReady = !!termEl.terminal
  let ws: WebSocket | null = new Ctor(url)
  ws.binaryType = 'arraybuffer'

  function flush(): void {
    while (pending.length) {
      const chunk = pending.shift()!
      try { termEl.write?.(chunk) } catch { /* ignore xterm internal */ }
    }
  }

  function writeBytes(bytes: Uint8Array): void {
    if (!termReady) {
      pending.push(bytes)
      return
    }
    try { termEl.write?.(bytes) } catch { /* ignore xterm internal */ }
  }

  const onReady = (): void => {
    termReady = true
    flush()
  }
  termEl.addEventListener('sui-terminal-ready', onReady, { once: true })

  const onData = (e: Event): void => {
    const d = (e as CustomEvent<{ data?: string }>).detail?.data
    if (typeof d !== 'string' || !ws || ws.readyState !== WebSocket.OPEN) return
    ws.send(new TextEncoder().encode(d))
  }
  termEl.addEventListener('sui-terminal-data', onData)

  ws.addEventListener('open', () => opts.onOpen?.())
  ws.addEventListener('message', (ev) => {
    const data = ev.data
    if (data instanceof ArrayBuffer) {
      writeBytes(new Uint8Array(data))
    } else if (typeof data === 'string') {
      writeBytes(new TextEncoder().encode(data))
    }
  })
  const handleClose = (reason: string): void => {
    if (!ws) return
    ws = null
    opts.onDisconnect?.(reason)
  }
  ws.addEventListener('close', (ev) => handleClose(ev.reason || `closed (${ev.code})`))
  ws.addEventListener('error', () => handleClose('error'))

  return {
    dispose: () => {
      termEl.removeEventListener('sui-terminal-data', onData)
      termEl.removeEventListener('sui-terminal-ready', onReady)
      if (ws) {
        const sock = ws
        ws = null
        try { sock.close(1000, 'client dispose') } catch { /* ignore */ }
      }
    },
    isConnected: () => ws !== null && ws.readyState === WebSocket.OPEN,
  }
}
