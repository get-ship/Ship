import { controller, signal, attachBoundary, detachBoundary } from '@ship/sui-runtime'
import type { Signal, BoundarySource } from '@ship/sui-runtime'
import type { Scope } from './sui-error-boundary.sui'

interface HostWithReset extends HTMLElement {
  reset?: () => void
}

export default controller<Scope>({
  scope: (_baseScope: unknown, host?: Element) => {
    const caughtError = signal<Error | null>(null)
    const hasFallbackSlot = signal<boolean>(false)
    const reset = (): void => {
      caughtError.set(null)
      if (host instanceof HTMLElement) host.removeAttribute('data-error-message')
    }
    return { caughtError, hasFallbackSlot, reset }
  },

  mount(state, host) {
    const caughtError = state['caughtError'] as unknown as Signal<Error | null> | undefined
    const hasFallbackSlot = state['hasFallbackSlot'] as unknown as Signal<boolean> | undefined
    const reset = state['reset'] as unknown as (() => void) | undefined
    if (!caughtError || !hasFallbackSlot || !reset) {
      throw new Error('[sui-error-boundary] scope signals missing — controller wiring is broken.')
    }

    // Authors commonly wrap their custom fallback in <template slot="fallback">
    // so the markup isn't visible while the boundary is healthy. Templates
    // don't render their contents, so we clone the inert template into a
    // live wrapper element bearing the same slot name — the assignment
    // becomes visible the moment the boundary swaps to its fallback branch.
    const templates = host.querySelectorAll(':scope > template[slot="fallback"]')
    for (const tpl of Array.from(templates)) {
      const wrap = document.createElement('div')
      wrap.setAttribute('slot', 'fallback')
      wrap.appendChild((tpl as HTMLTemplateElement).content.cloneNode(true))
      tpl.replaceWith(wrap)
    }
    hasFallbackSlot.set(!!host.querySelector(':scope > [slot="fallback"]'))

    attachBoundary(host, {
      catch(err, source) {
        // Re-entrancy guard: once the fallback is rendered, a second
        // inner throw shouldn't overwrite the original error.
        if (caughtError() !== null) return
        const error = err instanceof Error ? err : new Error(String(err))
        host.setAttribute('data-error-message', error.message)
        caughtError.set(error)
        host.dispatchEvent(
          new CustomEvent('sui-error-boundary:caught', {
            bubbles: true,
            composed: true,
            detail: { error, source: source as BoundarySource },
          }),
        )
      },
      reset,
    })

    ;(host as HostWithReset).reset = reset

    return () => {
      detachBoundary(host)
      delete (host as HostWithReset).reset
    }
  },
})
