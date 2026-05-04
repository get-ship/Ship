import { controller } from '@ship/ui-runtime'
import type { Scope } from './sui-annotation-layer.sui'
import {
  type AnnotationEvent,
  buildAnnotationEvent,
  getCSSSelector,
} from './sui-annotation.internals'

type Mode = 'off' | 'point' | 'region'

interface Pending {
  kind: 'point' | 'region'
  x: number
  y: number
  selector?: string
  rect?: [number, number, number, number]
}

interface Drag {
  startX: number
  startY: number
  curX: number
  curY: number
}

const DEFAULT_ENDPOINT = '/api/feedback'

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    let staged: AnnotationEvent[] = []
    let _pending: Pending | null = null
    let _drag: Drag | null = null
    const _positions: Record<string, { x: number; y: number }> = {}
    let _raf: number | null = null

    // Child containers created inside the overlay div.
    let dragRectEl: HTMLDivElement | null = null
    let markersEl: HTMLDivElement | null = null
    let popoverContainerEl: HTMLDivElement | null = null

    function getOverlay(): HTMLElement | null {
      return scope.overlayEl() as HTMLElement | null
    }

    function overlayPos(e: PointerEvent): { x: number; y: number } {
      const el = getOverlay()
      if (!el) return { x: 0, y: 0 }
      const r = el.getBoundingClientRect()
      return { x: e.clientX - r.left, y: e.clientY - r.top }
    }

    // ─── Gesture handlers ─────────────────────────────────────────────────

    const onPointerDown = (e: PointerEvent): void => {
      const mode = scope.mode() as Mode
      if (mode === 'off') return
      const t = e.target as HTMLElement
      if (t.closest('[data-annotation-popover]') || t.closest('sui-annotation-marker')) return
      if (_pending) { _pending = null; refresh(); return }
      e.preventDefault()
      const p = overlayPos(e)
      _drag = { startX: p.x, startY: p.y, curX: p.x, curY: p.y }
      ;(e.target as Element).setPointerCapture?.(e.pointerId)
    }

    const onPointerMove = (e: PointerEvent): void => {
      if (!_drag) return
      const p = overlayPos(e)
      _drag = { ..._drag, curX: p.x, curY: p.y }
      if (_raf) return
      _raf = requestAnimationFrame(() => {
        _raf = null
        updateDragRect()
      })
    }

    const onPointerUp = (e: PointerEvent): void => {
      if (!_drag) return
      const p = overlayPos(e)
      const dx = Math.abs(p.x - _drag.startX)
      const dy = Math.abs(p.y - _drag.startY)
      const mode = scope.mode() as Mode
      if (mode === 'point' || (mode === 'region' && dx < 8 && dy < 8)) {
        _pending = { kind: 'point', x: p.x, y: p.y, selector: 'body' }
      } else if (mode === 'region') {
        const x = Math.min(_drag.startX, p.x)
        const y = Math.min(_drag.startY, p.y)
        const rect: [number, number, number, number] = [Math.round(x), Math.round(y), Math.round(dx), Math.round(dy)]
        _pending = { kind: 'region', x: x + dx / 2, y: y + dy / 2, rect }
      }
      _drag = null
      refresh()
      void e
    }

    // ─── DOM rendering ────────────────────────────────────────────────────

    function updateDragRect() {
      if (!dragRectEl) return
      const d = _drag
      if (d && (Math.abs(d.curX - d.startX) > 4 || Math.abs(d.curY - d.startY) > 4)) {
        const x = Math.min(d.startX, d.curX)
        const y = Math.min(d.startY, d.curY)
        const w = Math.abs(d.curX - d.startX)
        const h = Math.abs(d.curY - d.startY)
        dragRectEl.style.cssText = `left:${x}px;top:${y}px;width:${w}px;height:${h}px`
        dragRectEl.style.display = ''
      } else {
        dragRectEl.style.display = 'none'
      }
    }

    function renderMarkers() {
      if (!markersEl) return
      markersEl.innerHTML = ''
      staged.forEach((a, i) => {
        const m = document.createElement('sui-annotation-marker') as HTMLElement & {
          x?: number
          y?: number
        }
        m.setAttribute('kind', a.kind)
        m.setAttribute('index', String(i))
        if (a.kind === 'region' && a.target.rect) {
          m.setAttribute('rect', a.target.rect.join(','))
        } else {
          const pos = _positions[a.id] ?? { x: 0, y: 0 }
          m.x = pos.x
          m.y = pos.y
        }
        markersEl!.appendChild(m)
      })
    }

    function renderPopover() {
      if (!popoverContainerEl) return
      popoverContainerEl.innerHTML = ''
      if (!_pending) return

      const p = _pending
      const left = Math.min(p.x + 12, 10_000)
      const top = Math.min(p.y + 12, 10_000)

      const div = document.createElement('div')
      div.className = 'popover'
      div.setAttribute('data-annotation-popover', '')
      div.style.cssText = `left:${left}px;top:${top}px`
      div.addEventListener('pointerdown', (e) => e.stopPropagation())

      const ta = document.createElement('textarea')
      ta.setAttribute('part', 'note-input')
      ta.placeholder = 'Add a note…'
      ta.addEventListener('keydown', (e: KeyboardEvent) => {
        if (e.key === 'Escape') { _pending = null; refresh() }
        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) confirmNote(ta.value)
      })

      const actions = document.createElement('div')
      actions.className = 'popover-actions'

      const cancelBtn = document.createElement('button')
      cancelBtn.type = 'button'
      cancelBtn.textContent = 'Cancel'
      cancelBtn.addEventListener('click', () => { _pending = null; refresh() })

      const sendBtn = document.createElement('button')
      sendBtn.type = 'button'
      sendBtn.dataset.primary = ''
      sendBtn.textContent = 'Send'
      sendBtn.addEventListener('click', () => confirmNote(ta.value))

      actions.append(cancelBtn, sendBtn)
      div.append(ta, actions)
      popoverContainerEl.appendChild(div)
    }

    function refresh() {
      updateDragRect()
      renderMarkers()
      renderPopover()
    }

    // ─── Note confirmation ────────────────────────────────────────────────

    function confirmNote(note: string) {
      if (!_pending) return
      const trimmed = note.trim()
      if (!trimmed) { _pending = null; refresh(); return }
      const p = _pending
      const ev = buildAnnotationEvent({
        viewId: (scope.viewId() as string) || undefined,
        kind: p.kind,
        selector: p.kind === 'point' ? p.selector : undefined,
        rect: p.kind === 'region' ? p.rect : undefined,
        note: trimmed,
      })
      _positions[ev.id] = { x: p.x, y: p.y }
      staged = [...staged, ev]
      host.dispatchEvent(
        new CustomEvent('sui-annotation-staged', { detail: ev, bubbles: true, composed: true }),
      )
      _pending = null
      if (!(scope.staging() as boolean)) void sendOne(ev)
      refresh()
    }

    // ─── Send / clear ─────────────────────────────────────────────────────

    async function sendOne(ev: AnnotationEvent): Promise<void> {
      const endpoint = (scope.endpoint() as string) || DEFAULT_ENDPOINT
      try {
        const resp = await fetch(endpoint, {
          method: 'POST',
          credentials: 'include',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(ev),
        })
        if (!resp.ok) {
          host.dispatchEvent(
            new CustomEvent('sui-annotation-error', {
              detail: { code: `http_${resp.status}`, id: ev.id },
              bubbles: true, composed: true,
            }),
          )
          return
        }
        staged = staged.filter((s) => s.id !== ev.id)
        host.dispatchEvent(
          new CustomEvent('sui-annotation-sent', {
            detail: { id: ev.id, status: resp.status },
            bubbles: true, composed: true,
          }),
        )
        refresh()
      } catch (e) {
        host.dispatchEvent(
          new CustomEvent('sui-annotation-error', {
            detail: { code: 'network_error', message: String(e), id: ev.id },
            bubbles: true, composed: true,
          }),
        )
      }
    }

    // ─── Public API ───────────────────────────────────────────────────────

    Object.defineProperty(host, 'staged', { get: () => staged, configurable: true })

    ;(host as HTMLElement & { send: () => Promise<void>; clear: () => void; viewRoot?: HTMLElement }).send =
      async () => {
        for (const ev of [...staged]) await sendOne(ev)
      }
    ;(host as HTMLElement & { send: () => Promise<void>; clear: () => void }).clear = () => {
      staged = []
      refresh()
    }

    // ─── Init ─────────────────────────────────────────────────────────────

    const overlayEl = getOverlay()
    if (overlayEl) {
      dragRectEl = document.createElement('div')
      dragRectEl.className = 'drag-rect'
      dragRectEl.style.display = 'none'
      overlayEl.appendChild(dragRectEl)

      markersEl = document.createElement('div')
      overlayEl.appendChild(markersEl)

      popoverContainerEl = document.createElement('div')
      overlayEl.appendChild(popoverContainerEl)

      overlayEl.addEventListener('pointerdown', onPointerDown)
      overlayEl.addEventListener('pointermove', onPointerMove)
      overlayEl.addEventListener('pointerup', onPointerUp)
    }

    return () => {
      if (_raf) { cancelAnimationFrame(_raf); _raf = null }
      const el = getOverlay()
      if (el) {
        el.removeEventListener('pointerdown', onPointerDown)
        el.removeEventListener('pointermove', onPointerMove)
        el.removeEventListener('pointerup', onPointerUp)
      }
    }
  },
})
