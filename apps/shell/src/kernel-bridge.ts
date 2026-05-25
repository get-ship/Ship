//! Kernel-event bridge. One SSE socket to `/api/v1/events/stream`,
//! multiplexed to any number of subscribers by event-namespace prefix.
//!
//! Connects lazily on the first subscribe; closes 30s after the last
//! unsubscribe. Reconnects with linear backoff on drop.

const CLOSE_GRACE_MS = 30_000
const RECONNECT_BASE_MS = 1_000
const RECONNECT_MAX_MS = 15_000

interface KernelEvent {
  event_type: string
  entity: string
  actor?: string | undefined
  timestamp: string
  payload: unknown
  [k: string]: unknown
}

/** Wire envelope emitted by the daemon SSE endpoint
 *  (`apps/shipd/src/routes/runtime_api/events.rs`). `payload_json` is a
 *  string-wrapped JSON blob — the bridge parses it once into `payload`. */
interface WireEnvelope {
  event_type: string
  entity_id?: string
  actor?: string
  payload_json?: string
  created_at?: string
  [k: string]: unknown
}

type EventHandler = (event: KernelEvent) => void
type UnsubscribeFn = () => void

export class KernelBridge {
  private socket: EventSource | null = null
  private handlers = new Map<string, Set<EventHandler>>()
  private closeTimer: ReturnType<typeof setTimeout> | null = null
  private reconnectAttempts = 0

  subscribe(namespace: string, handler: EventHandler): UnsubscribeFn {
    const set = this.handlers.get(namespace) ?? new Set()
    set.add(handler)
    this.handlers.set(namespace, set)
    this.cancelClose()
    if (!this.socket) this.connect()
    return () => {
      const s = this.handlers.get(namespace)
      if (!s) return
      s.delete(handler)
      if (s.size === 0) this.handlers.delete(namespace)
      if (this.handlers.size === 0) this.scheduleClose()
    }
  }

  private connect(): void {
    const src = new EventSource('/api/v1/events/stream')
    this.socket = src
    // Daemon emits `event: ship.event` so EventSource.onmessage (which
    // only fires for the default `message` type) never delivers — bind
    // the named-event handler explicitly. See
    // apps/shipd/src/routes/runtime_api/events.rs.
    const onWire = (e: MessageEvent): void => {
      let envelope: WireEnvelope
      try { envelope = JSON.parse(e.data) as WireEnvelope } catch { return }
      let payload: unknown = null
      if (typeof envelope.payload_json === 'string' && envelope.payload_json.length > 0) {
        try { payload = JSON.parse(envelope.payload_json) } catch { payload = envelope.payload_json }
      }
      this.fanOut({
        event_type: envelope.event_type,
        entity: (envelope.entity_id as string | undefined) ?? '',
        actor: envelope.actor as string | undefined,
        timestamp: (envelope.created_at as string | undefined) ?? '',
        payload,
      })
    }
    src.addEventListener('ship.event', onWire)
    src.onmessage = onWire
    src.onopen = () => {
      this.reconnectAttempts = 0
    }
    src.onerror = () => {
      src.close()
      this.socket = null
      if (this.handlers.size === 0) return
      const delay = Math.min(
        RECONNECT_BASE_MS * 2 ** this.reconnectAttempts,
        RECONNECT_MAX_MS,
      )
      this.reconnectAttempts++
      setTimeout(() => {
        if (this.handlers.size > 0 && !this.socket) this.connect()
      }, delay)
    }
  }

  private fanOut(event: KernelEvent): void {
    for (const [ns, handlers] of this.handlers) {
      if (ns === '' || event.event_type.startsWith(ns)) {
        for (const h of handlers) {
          try { h(event) } catch (err) {
            console.error('[kernel-bridge] subscriber error:', err)
          }
        }
      }
    }
  }

  private scheduleClose(): void {
    this.cancelClose()
    this.closeTimer = setTimeout(() => {
      this.closeTimer = null
      if (this.handlers.size === 0 && this.socket) {
        this.socket.close()
        this.socket = null
      }
    }, CLOSE_GRACE_MS)
  }

  private cancelClose(): void {
    if (this.closeTimer) {
      clearTimeout(this.closeTimer)
      this.closeTimer = null
    }
  }
}
