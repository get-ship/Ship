import { controller, signal, effect, debounce, untrack, ref } from '@ship/sui-runtime'
import type { Scope } from './sui-canvas.sui'
import { mountExcalidraw, type CanvasProps, type CanvasTheme } from './excalidraw-bridge'

interface ChangePayload {
  value: string
  sceneVersion: number
}

// <sui-canvas> controller. Owns the Excalidraw React-root lifecycle
// through the bridge: mount on connect, dispose on disconnect. Prop
// changes (value / readonly / theme) flow into the React tree via an
// effect; scene edits flow back out as a 200ms-debounced
// `sui-canvas-change` event.
export default controller<Scope>({
  scope: () => {
    // Raw scene edits land here; `debounced` gates the outbound event.
    const lastChange = signal<ChangePayload | null>(null)
    const debounced = debounce(lastChange, 200)
    const surfaceEl = ref<HTMLElement>(null)
    return { lastChange, debounced, surfaceEl }
  },

  mount(state, host) {
    const surface = state.surfaceEl as HTMLElement | null
    if (!surface) return

    const readProps = (): CanvasProps => ({
      value: (state.value as string) ?? '',
      readonly: Boolean(state.readonly),
      theme: ((state.theme as string) === 'dark' ? 'dark' : 'light') as CanvasTheme,
    })

    const bridge = mountExcalidraw(surface, {
      ...untrack(readProps),
      onChange: (value, sceneVersion) => {
        // Proxy routes the assignment through the signal's setter.
        state.lastChange = { value, sceneVersion }
      },
    })

    // Push external prop changes into the React tree. The first run is
    // the initial mount (already rendered above), so skip it.
    let first = true
    const stopProps = effect(() => {
      const props = readProps()
      if (first) {
        first = false
        return
      }
      bridge.update(props)
    })

    // Emit the debounced change event. Skip the initial `null`.
    const stopEmit = effect(() => {
      const change = state.debounced
      if (!change) return
      host.dispatchEvent(
        new CustomEvent('sui-canvas-change', {
          detail: change,
          bubbles: true,
          composed: true,
        }),
      )
    })

    return () => {
      stopProps()
      stopEmit()
      bridge.destroy()
    }
  },
})
