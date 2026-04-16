/**
 * <ship-annotation-layer>
 *
 * Framework-agnostic annotation capture overlay. Wraps any slotted
 * content, captures pointer gestures (point click or region drag),
 * opens a minimal inline popover for the note, and either POSTs each
 * annotation immediately or stages them for a batch `send()`.
 *
 * Attributes:
 *   - view-id   — required; sets `target.view_id` on emitted events
 *   - mode      — off | point | region (default "off")
 *   - endpoint  — override the default `/api/runtime/feedback`
 *   - staging   — boolean; if set, batch until `send()` is called
 *
 * Events (bubbles, composed):
 *   - ship-annotation-staged → detail: AnnotationEvent
 *   - ship-annotation-sent   → detail: { id, status }
 *   - ship-annotation-error  → detail: { code, message, id? }
 *
 * Methods:
 *   - send()  — POST all staged items serially
 *   - clear() — drop all staged items without sending
 *
 * Auth: relies on the ambient `ship_session` cookie; `fetch` runs
 * with `credentials: 'include'`.
 */

import { LitElement, html } from 'lit'
import {
  type AnnotationEvent,
  buildAnnotationEvent,
  getCSSSelector,
  getElementsInRect,
} from '../utils/annotation-helpers'
import { layerStyles, renderPopover } from './ship-annotation-layer.internals'
import './ship-annotation-marker'

type Mode = 'off' | 'point' | 'region'

interface Pending {
  kind: 'point' | 'region'
  x: number
  y: number
  selector?: string
  rect?: [number, number, number, number]
}

const DEFAULT_ENDPOINT = '/api/runtime/feedback'

export class ShipAnnotationLayer extends LitElement {
  static styles = layerStyles

  static properties = {
    viewId: { type: String, attribute: 'view-id', reflect: true },
    mode: { type: String, reflect: true },
    endpoint: { type: String, reflect: true },
    staging: { type: Boolean, reflect: true },
    staged: { state: true },
    _pending: { state: true },
    _drag: { state: true },
  }

  viewId = ''
  mode: Mode = 'off'
  endpoint: string = DEFAULT_ENDPOINT
  staging = false
  staged: AnnotationEvent[] = []

  private _pending: Pending | null = null
  private _drag: { startX: number; startY: number; curX: number; curY: number } | null = null
  private _overlayEl: HTMLDivElement | null = null
  private _positions: Record<string, { x: number; y: number }> = {}

  // ─── gestures ───

  private overlayPos(e: PointerEvent): { x: number; y: number } {
    const el = this._overlayEl
    if (!el) return { x: 0, y: 0 }
    const r = el.getBoundingClientRect()
    return { x: e.clientX - r.left, y: e.clientY - r.top }
  }

  private onPointerDown = (e: PointerEvent): void => {
    if (this.mode === 'off') return
    const t = e.target as HTMLElement
    if (t.closest('[data-annotation-popover]') || t.closest('ship-annotation-marker')) return
    if (this._pending) { this._pending = null; return }
    e.preventDefault()
    const p = this.overlayPos(e)
    this._drag = { startX: p.x, startY: p.y, curX: p.x, curY: p.y }
    ;(e.target as Element).setPointerCapture?.(e.pointerId)
  }

  private onPointerMove = (e: PointerEvent): void => {
    if (!this._drag) return
    const p = this.overlayPos(e)
    this._drag = { ...this._drag, curX: p.x, curY: p.y }
  }

  private onPointerUp = (e: PointerEvent): void => {
    if (!this._drag) return
    const p = this.overlayPos(e)
    const dx = Math.abs(p.x - this._drag.startX)
    const dy = Math.abs(p.y - this._drag.startY)
    if (this.mode === 'point' || (this.mode === 'region' && dx < 8 && dy < 8)) {
      let selector = 'body'
      const el = this.pickElementAt(p.x, p.y)
      if (el) selector = getCSSSelector(el)
      this._pending = { kind: 'point', x: p.x, y: p.y, selector }
    } else if (this.mode === 'region') {
      const x = Math.min(this._drag.startX, p.x)
      const y = Math.min(this._drag.startY, p.y)
      const rect: [number, number, number, number] = [
        Math.round(x), Math.round(y), Math.round(dx), Math.round(dy),
      ]
      this._pending = { kind: 'region', x: x + dx / 2, y: y + dy / 2, rect }
    }
    this._drag = null
    void e
  }

  private pickElementAt(x: number, y: number): Element | null {
    const slot = this.shadowRoot?.querySelector('slot')
    const assigned = slot?.assignedElements({ flatten: true }) ?? []
    if (!this._overlayEl) return null
    const overlayR = this._overlayEl.getBoundingClientRect()
    const clientX = overlayR.left + x
    const clientY = overlayR.top + y
    for (const root of assigned) {
      const host = root as HTMLElement
      const r = host.getBoundingClientRect()
      if (clientX < r.left || clientX > r.right || clientY < r.top || clientY > r.bottom) continue
      const el = host.ownerDocument.elementFromPoint(clientX, clientY)
      if (el && host.contains(el)) return el
    }
    return null
  }

  // ─── staging / send ───

  confirmNote(note: string): void {
    const p = this._pending
    if (!p) return
    const trimmed = note.trim()
    if (!trimmed) { this._pending = null; return }
    const elements =
      p.kind === 'region' && p.rect && this.ownerDocument
        ? getElementsInRect(this.ownerDocument, p.rect)
        : undefined
    const ev = buildAnnotationEvent({
      viewId: this.viewId || undefined,
      kind: p.kind,
      selector: p.kind === 'point' ? p.selector : undefined,
      rect: p.kind === 'region' ? p.rect : undefined,
      note: trimmed,
      selection:
        elements && elements.length > 0
          ? { kind: 'elements', text: elements.join(',') }
          : undefined,
    })
    this._positions[ev.id] = { x: p.x, y: p.y }
    this.staged = [...this.staged, ev]
    this.dispatchEvent(
      new CustomEvent('ship-annotation-staged', { detail: ev, bubbles: true, composed: true }),
    )
    this._pending = null
    if (!this.staging) void this.sendOne(ev)
  }

  cancelNote(): void { this._pending = null }

  async send(): Promise<void> {
    const queue = [...this.staged]
    for (const ev of queue) await this.sendOne(ev)
  }

  clear(): void { this.staged = [] }

  private async sendOne(ev: AnnotationEvent): Promise<void> {
    try {
      const resp = await fetch(this.endpoint || DEFAULT_ENDPOINT, {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(ev),
      })
      if (!resp.ok) {
        const body = (await resp.json().catch(() => ({}))) as {
          error?: { code?: string; message?: string }
        }
        this.dispatchEvent(
          new CustomEvent('ship-annotation-error', {
            detail: {
              code: body?.error?.code ?? `http_${resp.status}`,
              message: body?.error?.message ?? resp.statusText,
              id: ev.id,
            },
            bubbles: true, composed: true,
          }),
        )
        return
      }
      this.dispatchEvent(
        new CustomEvent('ship-annotation-sent', {
          detail: { id: ev.id, status: resp.status },
          bubbles: true, composed: true,
        }),
      )
      this.staged = this.staged.filter((s) => s.id !== ev.id)
    } catch (e) {
      this.dispatchEvent(
        new CustomEvent('ship-annotation-error', {
          detail: { code: 'network_error', message: String(e), id: ev.id },
          bubbles: true, composed: true,
        }),
      )
    }
  }

  // ─── render ───

  firstUpdated(): void {
    this._overlayEl = this.shadowRoot!.querySelector('.overlay') as HTMLDivElement
  }

  render() {
    const d = this._drag
    const dragRect =
      d && (Math.abs(d.curX - d.startX) > 4 || Math.abs(d.curY - d.startY) > 4)
        ? {
            x: Math.min(d.startX, d.curX),
            y: Math.min(d.startY, d.curY),
            w: Math.abs(d.curX - d.startX),
            h: Math.abs(d.curY - d.startY),
          }
        : null
    return html`
      <div class="surface">
        <slot></slot>
        <div
          class="overlay"
          @pointerdown=${this.onPointerDown}
          @pointermove=${this.onPointerMove}
          @pointerup=${this.onPointerUp}
        >
          ${this.staged.map((a, i) => this.renderMarker(a, i))}
          ${dragRect
            ? html`<div
                class="drag-rect"
                style=${`left:${dragRect.x}px;top:${dragRect.y}px;width:${dragRect.w}px;height:${dragRect.h}px`}
              ></div>`
            : null}
          ${this._pending ? renderPopover(this, this._pending) : null}
        </div>
      </div>
    `
  }

  private renderMarker(a: AnnotationEvent, i: number) {
    if (a.kind === 'region' && a.target.rect) {
      return html`<ship-annotation-marker
        kind="region"
        rect=${a.target.rect.join(',')}
        index=${i}
      ></ship-annotation-marker>`
    }
    if (a.kind === 'point') {
      const pos = this._positions[a.id] ?? { x: 0, y: 0 }
      return html`<ship-annotation-marker
        kind="point"
        .x=${pos.x}
        .y=${pos.y}
        index=${i}
      ></ship-annotation-marker>`
    }
    return null
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-annotation-layer')) {
  customElements.define('ship-annotation-layer', ShipAnnotationLayer)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-annotation-layer': ShipAnnotationLayer
  }
}
