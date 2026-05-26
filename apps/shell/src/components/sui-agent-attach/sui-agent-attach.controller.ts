/**
 * Paired controller for <sui-agent-attach>.
 *
 * Mounts the WebSocket attach bridge against the embedded <sui-terminal>
 * once the ref resolves, and tears it down on unmount. Exposes
 * `host.connect()` / `host.disconnect()` for views that flip auto-connect
 * off and drive the lifecycle themselves.
 */

import { controller, effect, ref, signal } from '@ship/sui-runtime'
import type { Signal, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-agent-attach.sui'

import { attachAgentSession, type AgentSessionAttachment } from './attach'

interface TerminalEl extends HTMLElement {
  write?: (data: string | Uint8Array) => void
  clear?: () => void
  fit?: () => void
  terminal?: { cols: number; rows: number } | null
}

type HostEl = HTMLElement & {
  connect?: (() => void) | undefined
  disconnect?: (() => void) | undefined
}

export default controller({
  // Component-level signals the template renders (overlay state). Stored
  // on the scope so the template re-renders when they change; the bridge
  // assigns them as the WS lifecycle transitions.
  scope: (_ctx: ViewScope<Scope>) => {
    const connected = signal(false)
    const disconnectReason = signal('')
    const termRef = ref<TerminalEl>(null)
    return { connected, disconnectReason, termRef }
  },

  mount(state, host) {
    // `state` is the writable proxy; ref-typed scope keys are Signals at
    // runtime — cast to Scope to call them. Non-scope attrs (slug,
    // autoConnect) are host attributes, read as Signals via the proxy.
    const s = state as unknown as Scope & {
      slug: Signal<string>
      autoConnect: Signal<boolean>
    }

    let attachment: AgentSessionAttachment | null = null
    const open = (): void => {
      attachment?.dispose()
      attachment = null
      const term = s.termRef()
      const slug = s.slug()
      if (!term || !slug) return
      state.disconnectReason = ''
      attachment = attachAgentSession(term, slug, {
        onOpen: () => {
          state.connected = true
          host.dispatchEvent(new CustomEvent('sui-agent-attach-open', {
            bubbles: true, composed: true,
          }))
        },
        onDisconnect: (reason: string) => {
          state.connected = false
          state.disconnectReason = reason
          host.dispatchEvent(new CustomEvent('sui-agent-attach-disconnect', {
            detail: { reason }, bubbles: true, composed: true,
          }))
        },
      })
    }

    // Imperative API for callers that flip auto-connect=false.
    const hostEl = host as HostEl
    hostEl.connect = () => open()
    hostEl.disconnect = () => {
      attachment?.dispose()
      attachment = null
      state.connected = false
    }

    const disposers: Array<() => void> = []

    // Re-open the bridge whenever slug changes (e.g. row click hands a
    // new actor to the same drawer instance) so the consumer doesn't
    // need to thrash mount cycles by toggling the element in/out.
    disposers.push(effect(() => {
      const slug = s.slug()
      const auto = s.autoConnect()
      // Subscribe to the term ref so we re-run after bind:this resolves.
      void s.termRef()
      if (auto && slug) {
        open()
      } else {
        attachment?.dispose()
        attachment = null
        state.connected = false
      }
    }))

    // <sui-terminal> calls fit() inside a single rAF after mount, which
    // measures the container BEFORE our wrapper's layout (flex sizing in
    // a freshly-attached element) has settled. xterm sticks at a few
    // rows even though the container is full-size. Refit explicitly:
    //   • two rAFs after the term ref resolves (layout settles between
    //     the two frames in Chromium)
    //   • on every container resize via ResizeObserver
    //   • on every term resize as well — the first fit() inside <sui-terminal>
    //     might land mid-layout; observing the term element catches the
    //     follow-up size change once flex sizing kicks in.
    let observer: ResizeObserver | null = null
    disposers.push(effect(() => {
      const term = s.termRef()
      if (!term) return
      const refit = (): void => {
        try { term.fit?.() } catch { /* mid-teardown */ }
      }
      requestAnimationFrame(() => requestAnimationFrame(refit))
      if (typeof ResizeObserver !== 'undefined') {
        observer?.disconnect()
        observer = new ResizeObserver(refit)
        observer.observe(host)
        observer.observe(term)
      }
    }))

    return () => {
      for (const d of disposers) d()
      observer?.disconnect()
      observer = null
      attachment?.dispose()
      attachment = null
      hostEl.connect = undefined
      hostEl.disconnect = undefined
    }
  },
})
