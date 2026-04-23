//! Kernel-event bridge. One SSE socket to `/api/events/stream`,
//! multiplexed to any number of subscribers by event-namespace prefix.
//!
//! Connects lazily on the first subscribe; closes 30s after the last
//! unsubscribe. Reconnects with linear backoff on drop.

import type { EventHandler, KernelEvent, UnsubscribeFn } from "./types";

const CLOSE_GRACE_MS = 30_000;
const RECONNECT_BASE_MS = 1_000;
const RECONNECT_MAX_MS = 15_000;

export class KernelBridge {
  private socket: EventSource | null = null;
  private handlers = new Map<string, Set<EventHandler>>();
  private closeTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectAttempts = 0;

  subscribe(namespace: string, handler: EventHandler): UnsubscribeFn {
    const set = this.handlers.get(namespace) ?? new Set();
    set.add(handler);
    this.handlers.set(namespace, set);
    this.cancelClose();
    if (!this.socket) this.connect();
    return () => {
      const s = this.handlers.get(namespace);
      if (!s) return;
      s.delete(handler);
      if (s.size === 0) this.handlers.delete(namespace);
      if (this.handlers.size === 0) this.scheduleClose();
    };
  }

  private connect(): void {
    const src = new EventSource("/api/events/stream");
    this.socket = src;
    src.onmessage = (e) => {
      let ev: KernelEvent;
      try {
        ev = JSON.parse(e.data) as KernelEvent;
      } catch {
        return;
      }
      this.fanOut(ev);
    };
    src.onopen = () => {
      this.reconnectAttempts = 0;
    };
    src.onerror = () => {
      // EventSource auto-reconnects on some errors. Close + retry with
      // backoff so our state stays consistent.
      src.close();
      this.socket = null;
      if (this.handlers.size === 0) return;
      const delay = Math.min(
        RECONNECT_BASE_MS * 2 ** this.reconnectAttempts,
        RECONNECT_MAX_MS,
      );
      this.reconnectAttempts++;
      setTimeout(() => {
        if (this.handlers.size > 0 && !this.socket) this.connect();
      }, delay);
    };
  }

  private fanOut(event: KernelEvent): void {
    // Prefix match — manifest.events semantics. A handler registered at
    // `"job."` fires on every `job.created`, `job.failed`, etc. A handler
    // at `""` fires on everything.
    for (const [ns, handlers] of this.handlers) {
      if (ns === "" || event.event_type.startsWith(ns)) {
        for (const h of handlers) {
          try {
            h(event);
          } catch (err) {
            // Isolate subscriber errors so one bad handler doesn't break
            // the whole fan-out.
            // eslint-disable-next-line no-console
            console.error("[ship-shell] kernel subscriber threw:", err);
          }
        }
      }
    }
  }

  private scheduleClose(): void {
    this.cancelClose();
    this.closeTimer = setTimeout(() => {
      this.closeTimer = null;
      if (this.handlers.size === 0 && this.socket) {
        this.socket.close();
        this.socket = null;
      }
    }, CLOSE_GRACE_MS);
  }

  private cancelClose(): void {
    if (this.closeTimer) {
      clearTimeout(this.closeTimer);
      this.closeTimer = null;
    }
  }
}
