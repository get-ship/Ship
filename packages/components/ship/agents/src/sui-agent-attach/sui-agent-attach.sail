<scope>
  props: {
    slug = ''
    readonly = false
    autoConnect = true
  }
</scope>
<controller>
/**
 * Paired controller for <sui-agent-attach>.
 *
 * Mounts the WebSocket attach bridge against the embedded <ship-terminal>
 * once the ref resolves, and tears it down on unmount. Exposes
 * `host.connect()` / `host.disconnect()` for views that flip auto-connect
 * off and drive the lifecycle themselves.
 */

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sui-agent-attach.sail'
import { attachAgentSession, type AgentSessionAttachment } from './attach'

interface TerminalEl {
  write?: (data: string | Uint8Array) => void
  clear?: () => void
  fit?: () => void
  terminal?: { cols: number; rows: number } | null
}

type HostEl = Element & {
  connect?: (() => void) | undefined
  disconnect?: (() => void) | undefined
}

export default controller((ctx: ControllerScope<Scope>, host?: Element) => {
    const connected = signal(false)
    const disconnectReason = signal('')
    const termEl = signal<Element | null>(null)
    const captureTerm: Action<void> = (node) => {
      termEl.set(node)
      return { destroy: () => termEl.set(null) }
    }

    let attachment: AgentSessionAttachment | null = null

    const open = (): void => {
      attachment?.dispose()
      attachment = null
      const term = termEl() as TerminalEl | null
      const slug = ctx.slug()
      if (!term || !slug) return
      disconnectReason.set('')
      attachment = attachAgentSession(term as HTMLElement & TerminalEl, slug, {
        onOpen: () => {
          connected.set(true)
          host?.dispatchEvent(new CustomEvent('sui-agent-attach-open', {
            bubbles: true, composed: true,
          }))
        },
        onDisconnect: (reason: string) => {
          connected.set(false)
          disconnectReason.set(reason)
          host?.dispatchEvent(new CustomEvent('sui-agent-attach-disconnect', {
            detail: { reason }, bubbles: true, composed: true,
          }))
        },
      })
    }

    const close = (): void => {
      attachment?.dispose()
      attachment = null
      connected.set(false)
    }

    const hostEl = host as HostEl | undefined
    if (hostEl) {
      hostEl.connect = () => open()
      hostEl.disconnect = () => close()
    }

    effect(() => {
      const slug = ctx.slug()
      const auto = ctx.autoConnect()
      void termEl()
      if (auto && slug) open()
      else close()
    })

    effect(() => {
      const term = termEl() as TerminalEl | null
      if (!term || !host) return
      const refit = (): void => {
        try { term.fit?.() } catch { /* mid-teardown */ }
      }
      requestAnimationFrame(() => requestAnimationFrame(refit))
      if (typeof ResizeObserver === 'undefined') return
      const observer = new ResizeObserver(refit)
      observer.observe(host)
      observer.observe(term as Element)
      return () => observer.disconnect()
    })

    onCleanup(() => {
      close()
      if (hostEl) {
        hostEl.connect = undefined
        hostEl.disconnect = undefined
      }
    })

    return { connected, disconnectReason, captureTerm }
})
</controller>

<!--
  <sui-agent-attach> — xterm.js attached to a running PTY-backed agent
  session over the daemon's WebSocket bridge.

  Wraps <ship-terminal> (the xterm engine) with the byte-pump (see
  ./attach.ts) so views can drop in a fully functional attached terminal
  without re-implementing the WS lifecycle.

  Backend contract (apps/shipd/src/routes/agent_sessions_attach.rs):
    GET /api/v1/agent_sessions/{slug}/attach   (Upgrade: websocket)
      server → client: binary frames carrying raw PTY bytes
      client → server: binary frames written as PTY input

  Props
  ─────────────────────────────────────────────────────────────────────────
  slug          (string, '')           — agent-session slug to attach to.
                                         When empty the component renders
                                         the disconnected overlay without
                                         opening a WS.
  auto-connect  (bool, true, reflect)  — open the WS on mount. Set false
                                         to defer until the host calls the
                                         imperative connect() method.
  readonly      (bool, false, reflect) — pass through to <ship-terminal>;
                                         when true, inbound bytes still
                                         render but xterm doesn't capture
                                         keystrokes (display-only attach).

  Events (bubble, composed)
  ─────────────────────────────────────────────────────────────────────────
  sui-agent-attach-open       fired when the WS open handshake completes
  sui-agent-attach-disconnect detail: { reason: string }

  Methods (imperative API on the host element)
  ─────────────────────────────────────────────────────────────────────────
  host.connect()    open the WS (no-op if already connected)
  host.disconnect() close the WS without disposing the terminal
-->

<column gap="0">
  <row>
    <ship-terminal
      use:captureTerm
      readonly={readonly()}
      fit
    ></ship-terminal>
  </row>

  <if {!slug()}>
    <div data-test="empty-slug">
      <column gap="3" align="center">
        <p>No agent session selected</p>
        <p>Set the slug attribute to attach.</p>
      </column>
    </div>
  {else-if {!connected() && disconnectReason()} =>
    <div data-test="disconnected">
      <column gap="3" align="center">
        <p>Session disconnected</p>
        <p>{disconnectReason()}</p>
      </column>
    </div>
  }</if>
</column>
