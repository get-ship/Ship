/**
 * <ship-log-stream>
 *
 * Append-only scrollable log pane backed by a Server-Sent Events
 * connection. Opens an EventSource against `url`, consumes events of
 * the named type (`event`, default `"log"`), and appends each data
 * payload as a new line in a monospace scroll container.
 *
 * Attributes:
 *   - url — SSE endpoint (required). Changing the attribute reconnects.
 *   - event — SSE event type to consume (default `"log"`)
 *   - autoscroll — when present, keeps view pinned to the bottom
 *     unless the user scrolls up; default off
 *   - active — when present, the stream is open. Toggle this attribute
 *     from the caller to open/close without tearing down the element.
 *     Absent = no connection.
 *   - max-bytes — rolling cap on retained payload (default 1_000_000).
 *     When exceeded, earliest content is trimmed.
 *
 * Events:
 *   - ship-stream-open  — fired on EventSource.onopen
 *   - ship-stream-error — fired on EventSource.onerror
 *   - ship-stream-close — fired after close() teardown
 *
 * CSS parts:
 *   - scroll — the scrolling pre container
 */

import { LitElement, css, html } from 'lit'
import { defineOnce, MONO } from '../util'

export class ShipLogStream extends LitElement {
  static styles = css`
    :host {
      display: block;
      min-height: 8rem;
      max-height: 24rem;
      border-radius: var(--radius-md);
      border: 1px solid color-mix(in oklch, var(--border) 60%, transparent);
      background: color-mix(in oklch, var(--background) 95%, black 5%);
      color: var(--foreground);
      overflow: hidden;
    }
    [part='scroll'] {
      height: 100%;
      max-height: inherit;
      margin: 0;
      padding: var(--space-3) var(--space-4);
      font-family: ${MONO};
      font-size: var(--text-xs);
      line-height: 1.55;
      white-space: pre-wrap;
      word-break: break-word;
      overflow-y: auto;
    }
    [part='scroll'][data-empty] {
      color: var(--muted-foreground);
      font-style: italic;
    }
  `

  static properties = {
    url: { type: String, reflect: true },
    event: { type: String, reflect: true },
    autoscroll: { type: Boolean, reflect: true },
    active: { type: Boolean, reflect: true },
    maxBytes: { type: Number, attribute: 'max-bytes' },
  }

  url = ''
  event = 'log'
  autoscroll = false
  active = false
  maxBytes = 1_000_000

  private _source: EventSource | null = null
  private _buffer = ''
  private _stickyBottom = true

  updated(changed: Map<string, unknown>) {
    if (changed.has('url') || changed.has('event')) {
      // Reconnect if the endpoint changed while active.
      if (this.active && this._source) {
        this.close()
        this.open()
      }
    }
    if (changed.has('active')) {
      if (this.active) this.open()
      else this.close()
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.close()
  }

  open() {
    if (!this.url || this._source) return
    try {
      this._source = new EventSource(this.url, { withCredentials: true })
    } catch {
      this.dispatchEvent(new CustomEvent('ship-stream-error'))
      return
    }
    this._source.addEventListener(this.event, (ev) => {
      const chunk = (ev as MessageEvent).data ?? ''
      this._append(chunk)
      // Emit a tick so ops-center indicators can pulse while bytes flow.
      // Detail carries the chunk byte count; callers typically don't need it.
      this.dispatchEvent(new CustomEvent('ship-log-tick', {
        detail: { bytes: chunk.length },
        bubbles: true,
        composed: true,
      }))
    })
    this._source.onopen = () => {
      this.dispatchEvent(new CustomEvent('ship-stream-open'))
    }
    this._source.onerror = () => {
      this.dispatchEvent(new CustomEvent('ship-stream-error'))
    }
  }

  close() {
    if (this._source) {
      this._source.close()
      this._source = null
      this.dispatchEvent(new CustomEvent('ship-stream-close'))
    }
  }

  clear() {
    this._buffer = ''
    this.requestUpdate()
  }

  private _append(chunk: string) {
    this._buffer += chunk
    if (this._buffer.length > this.maxBytes) {
      const overflow = this._buffer.length - this.maxBytes
      const cut = this._buffer.indexOf('\n', overflow)
      this._buffer = this._buffer.slice(cut >= 0 ? cut + 1 : overflow)
    }
    this.requestUpdate()
  }

  private _onScroll = (e: Event) => {
    const pre = e.currentTarget as HTMLElement
    this._stickyBottom = pre.scrollHeight - pre.clientHeight <= pre.scrollTop + 4
  }

  protected firstUpdated() {
    // Honor initial `active` presence without waiting for an attribute
    // change callback.
    if (this.active && !this._source) this.open()
  }

  protected updated_scroll() {
    if (!this.autoscroll || !this._stickyBottom) return
    const pre = this.renderRoot.querySelector<HTMLElement>('[part="scroll"]')
    if (pre) pre.scrollTop = pre.scrollHeight
  }

  protected render() {
    // Schedule post-render autoscroll on next frame so it runs after
    // the new textContent lays out.
    queueMicrotask(() => this.updated_scroll())
    const empty = this._buffer.length === 0
    return html`<pre
      part="scroll"
      ?data-empty=${empty}
      @scroll=${this._onScroll}
    >${empty ? '(no output yet)' : this._buffer}</pre>`
  }
}

defineOnce('ship-log-stream', ShipLogStream)
