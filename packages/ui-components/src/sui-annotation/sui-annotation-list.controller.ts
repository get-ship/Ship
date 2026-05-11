import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './sui-annotation-list.sui'
import type { Signal } from '@ship/sui-runtime'
import type { AnnotationEvent } from './sui-annotation.internals'

export default controller<Scope>({
  scope: (_, host) => {
    const staged = signal<AnnotationEvent[]>([])

    const emitRemove = (id: string): void => {
      host!.dispatchEvent(
        new CustomEvent('sui-annotation-remove', {
          detail: { id },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { staged, emitRemove }
  },

  mount(state, host) {
    const scope = state as unknown as Scope
    const stagedSignal = scope.staged as unknown as Signal<AnnotationEvent[]>

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const initial = (host as any).staged
    if (initial != null) stagedSignal.set(Array.isArray(initial) ? initial : [])

    Object.defineProperty(host, 'staged', {
      configurable: true,
      get: () => stagedSignal(),
      set: (v: AnnotationEvent[]) => stagedSignal.set(Array.isArray(v) ? v : []),
    })
  },
})
