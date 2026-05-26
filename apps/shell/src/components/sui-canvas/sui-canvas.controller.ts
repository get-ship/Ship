/**
 * Paired controller for <sui-canvas>.
 *
 * Subscribes to kernel events `canvas.content` and `canvas.morph` filtered
 * by `canvas-id`. Forwards interaction events (clicks, input changes)
 * inside the canvas surface back to the kernel as `canvas.interaction`.
 *
 * The agent never touches DOM directly — it emits the two event types via
 * the existing `mcp__ship__emit_event` tool. The wire is documented in
 * the .sui header.
 */

import { controller, signal, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { morph } from '@ship/sui-runtime'
import type { Scope } from './sui-canvas.sui'
import { getCanvasBridge, type CanvasEventDetail } from './bridge'

interface MorphPatch {
  textContent?: string
  innerHTML?: string
  attrs?: Record<string, string | null>
  classAdd?: string[]
  classRemove?: string[]
}

type CanvasHost = HTMLElement & {
  setContent(sui: string): void
  applyMorph(target: string, patch: MorphPatch): void
  clear(): void
  getContentHtml(): string
}

export default controller({
  scope: (props: ViewScope<Scope>) => {
    const canvasIdProp = props.canvasId
    const hasContent = signal(false)
    const surfaceEl = ref<HTMLElement>(null)
    return { canvasIdProp, hasContent, surfaceEl }
  },

  mount(state, host) {
    const bridge = getCanvasBridge()

    const surface = state.$signals.surfaceEl()
    if (!surface) return

    // Slot for agent-emitted content. The empty-state lives at
    // scope-render time; once setContent fires we replace surface
    // contents with the rendered markup, dropping the empty state.
    const contentRoot = host.ownerDocument.createElement('div')
    contentRoot.setAttribute('data-canvas-content', '')
    contentRoot.setAttribute('morph', '')
    contentRoot.style.minHeight = '100%'

    function setContent(sui: string): void {
      // Clear surface, mount contentRoot, morph SUI into it. Using
      // morph() preserves DOM identity for repeated full-content
      // updates (input focus, scroll position survive). For the very
      // first paint contentRoot is empty, so morph behaves as an
      // initial render.
      if (!contentRoot.isConnected) {
        surface!.replaceChildren(contentRoot)
      }
      morph(contentRoot, sui)
      state.$signals.hasContent.set(true)
      host.dispatchEvent(new CustomEvent('sui-canvas-content', {
        detail: { canvas_id: state.$signals.canvasIdProp(), length: sui.length },
        bubbles: true, composed: true,
      }))
    }

    function applyMorph(target: string, patch: MorphPatch): void {
      if (!contentRoot.isConnected) return
      const el = contentRoot.querySelector(target) as HTMLElement | null
      if (!el) {
        host.dispatchEvent(new CustomEvent('sui-canvas-morph', {
          detail: { canvas_id: state.$signals.canvasIdProp(), target, patch, miss: true },
          bubbles: true, composed: true,
        }))
        return
      }
      if (typeof patch.textContent === 'string') el.textContent = patch.textContent
      if (typeof patch.innerHTML === 'string') morph(el, patch.innerHTML)
      if (patch.attrs) {
        for (const [k, v] of Object.entries(patch.attrs)) {
          if (v === null) el.removeAttribute(k)
          else el.setAttribute(k, v)
        }
      }
      if (patch.classAdd) for (const c of patch.classAdd) el.classList.add(c)
      if (patch.classRemove) for (const c of patch.classRemove) el.classList.remove(c)
      host.dispatchEvent(new CustomEvent('sui-canvas-morph', {
        detail: { canvas_id: state.$signals.canvasIdProp(), target, patch },
        bubbles: true, composed: true,
      }))
    }

    function clear(): void {
      if (contentRoot.isConnected) contentRoot.remove()
      state.$signals.hasContent.set(false)
    }

    // ── Interaction streaming ────────────────────────────────────────
    // Clicks on `<button>`, links, and inputs inside the canvas
    // content bubble up here. We forward them as canvas.interaction
    // kernel events. The agent (subscribed to its mailbox) sees the
    // event in realtime. POSTing /api/v1/events/emit avoids a custom
    // route — re-uses the platform's event rail.

    function emitInteraction(kind: string, payload: Record<string, unknown>): void {
      const detail = { canvas_id: state.$signals.canvasIdProp(), kind, ...payload }
      host.dispatchEvent(new CustomEvent('sui-canvas-interact', {
        detail, bubbles: true, composed: true,
      }))
      void bridge.postInteraction(detail as CanvasEventDetail).catch((err) => {
        console.warn('[sui-canvas] interaction post failed:', err)
      })
    }

    function describeTarget(el: HTMLElement): Record<string, unknown> {
      const id = el.id || el.closest('[id]')?.id || ''
      const role = el.getAttribute('role') ?? ''
      const text = (el.textContent ?? '').trim().slice(0, 80)
      return { tag: el.tagName.toLowerCase(), id, role, text }
    }

    const onClick = (e: Event): void => {
      const t = e.target as HTMLElement | null
      if (!t || !contentRoot.contains(t)) return
      const interactive = t.closest('button, a, [data-canvas-action]') as HTMLElement | null
      if (!interactive) return
      const action = interactive.getAttribute('data-canvas-action') ?? ''
      emitInteraction('click', { ...describeTarget(interactive), action })
    }
    const onInput = (e: Event): void => {
      const t = e.target as (HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement) | null
      if (!t || !contentRoot.contains(t)) return
      emitInteraction('input', {
        ...describeTarget(t as HTMLElement),
        name: t.getAttribute('name') ?? '',
        value: (t as HTMLInputElement).value ?? '',
      })
    }

    surface.addEventListener('click', onClick)
    surface.addEventListener('change', onInput)
    // `input` event for live typing — debounce-free for v0; agent can
    // decide what to do with the firehose.
    surface.addEventListener('input', onInput)

    // ── Bridge subscription ──────────────────────────────────────────
    const offBridge = bridge.subscribe(state.$signals.canvasIdProp(), {
      onContent: ({ sui }) => setContent(sui),
      onMorph: ({ target, patch }) => applyMorph(target, patch as MorphPatch),
      onClear: () => clear(),
    })

    // ── Host imperative API ──────────────────────────────────────────
    const hostApi = host as CanvasHost
    hostApi.setContent = setContent
    hostApi.applyMorph = applyMorph
    hostApi.clear = clear
    hostApi.getContentHtml = () =>
      contentRoot.isConnected ? contentRoot.innerHTML : ''

    host.dispatchEvent(new CustomEvent('sui-canvas-mounted', {
      detail: { canvas_id: state.$signals.canvasIdProp() },
      bubbles: true, composed: true,
    }))

    return () => {
      surface.removeEventListener('click', onClick)
      surface.removeEventListener('change', onInput)
      surface.removeEventListener('input', onInput)
      offBridge()
    }
  },
})
