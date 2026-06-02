import { controller, signal, effect, debounce, ref } from '@ship/sui-runtime'
import type { Scope } from './sui-canvas.sui'
import { createCanvas } from '../engine/create-canvas'
import type { Transform } from '../engine/types'

// SSR fallback = the verified sui-combobox shape: plain JSON.parse in the
// controller, try/caught. The live path is prop:* identity onto the engine.
function parseSnapshot(raw: string): { transform?: Partial<Transform> } | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as { transform?: Partial<Transform> }
  } catch {
    return null
  }
}

// <sui-canvas> controller — the bare substrate primitive. Builds a
// createCanvas() in scope, forwards raw pointer/wheel events to the engine
// in on:*, and emits a debounced sui-canvas-viewport event via host.emit!.
export default controller<Scope>({
  scope: ({ value }) => {
    const viewportEl = ref<HTMLDivElement>(null)
    const canvas = createCanvas({
      getViewportElement: () => viewportEl(),
      initialTransform: parseSnapshot(value())?.transform,
    })
    // Drives the debounced outbound viewport event.
    const lastTransform = signal<Transform | null>(null)
    const debounced = debounce(lastTransform, 200)
    return { canvas, viewportEl, lastTransform, debounced }
  },

  derived: {
    transformCss: ({ canvas }) => {
      const t = canvas.transform()
      return `transform: translate(${t.x}px, ${t.y}px) scale(${t.scale})`
    },
  },

  on: {
    pointerDown: (s, e: PointerEvent) => {
      s.canvas.onPointerDown(e)
    },
    pointerMove: (s, e: PointerEvent) => {
      s.canvas.onPointerMove(e)
    },
    pointerUp: (s, e: PointerEvent) => {
      s.canvas.onPointerUp(e)
    },
    wheel: (s, e: WheelEvent) => {
      s.canvas.onWheel(e)
      e.preventDefault()
    },
  },

  mount(state, host) {
    // controller<Scope> OVER-CONSTRAINS union-typed reads — read the
    // reflected enum/number props imperatively off the host (the verified
    // sui-table.controller.ts:95 precedent), not through the typed scope.
    const minScale = host.getAttribute('min-scale')
    const maxScale = host.getAttribute('max-scale')
    state.canvas.setOptions({
      ...(minScale != null ? { minScale: Number(minScale) } : {}),
      ...(maxScale != null ? { maxScale: Number(maxScale) } : {}),
    })

    // Feed engine transform changes into the debounce source.
    const stopTrack = effect(() => {
      state.canvas.transform()
      state.lastTransform = state.canvas.transform()
    })

    // Emit the debounced viewport event. Skip the initial null.
    const stopEmit = effect(() => {
      const t = state.debounced
      if (!t) return
      host.emit!('sui-canvas-viewport', { transform: t })
    })

    return () => {
      stopTrack()
      stopEmit()
      state.canvas.dispose()
    }
  },
})
