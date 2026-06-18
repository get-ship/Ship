//! Kernel-event bridge. One Connect `StreamService.Events` server-stream
//! (the successor to the retired `/api/v1/events/stream` SSE route),
//! multiplexed to any number of subscribers by event-namespace prefix.
//!
//! Connects lazily on the first subscribe; closes 30s after the last
//! unsubscribe. Reconnects with exponential backoff on drop.

import { ops } from './ops'

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

/** Wire envelope carried in `JsonResponse.json` by the Connect
 *  `StreamService.Events` stream (`apps/shipd/src/ops_rpc.rs`) — the
 *  serialized kernel `EventEnvelope`. `payload_json` is a string-wrapped
 *  JSON blob, parsed once here into `payload`. */
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
  private abort: AbortController | null = null
  private handlers = new Map<string, Set<EventHandler>>()
  private closeTimer: ReturnType<typeof setTimeout> | null = null
  private reconnectAttempts = 0

  subscribe(namespace: string, handler: EventHandler): UnsubscribeFn {
    const set = this.handlers.get(namespace) ?? new Set()
    set.add(handler)
    this.handlers.set(namespace, set)
    this.cancelClose()
    if (!this.abort) this.connect()
    return () => {
      const s = this.handlers.get(namespace)
      if (!s) return
      s.delete(handler)
      if (s.size === 0) this.handlers.delete(namespace)
      if (this.handlers.size === 0) this.scheduleClose()
    }
  }

  private connect(): void {
    const abort = new AbortController()
    this.abort = abort
    void this.runStream(abort.signal)
  }

  /** Tail the Connect event stream; reconnect with backoff on drop. */
  private async runStream(signal: AbortSignal): Promise<void> {
    while (!signal.aborted) {
      try {
        for await (const msg of ops.stream.events({}, { signal })) {
          this.reconnectAttempts = 0
          this.dispatch(msg.json)
        }
      } catch {
        /* stream dropped — fall through to backoff + retry */
      }
      if (signal.aborted) break
      const delay = Math.min(
        RECONNECT_BASE_MS * 2 ** this.reconnectAttempts,
        RECONNECT_MAX_MS,
      )
      this.reconnectAttempts++
      await new Promise((r) => setTimeout(r, delay))
    }
  }

  /** Parse one `JsonResponse.json` envelope and fan it out to subscribers. */
  private dispatch(json: string): void {
    let envelope: WireEnvelope
    try { envelope = JSON.parse(json) as WireEnvelope } catch { return }
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
      if (this.handlers.size === 0 && this.abort) {
        this.abort.abort()
        this.abort = null
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
