// Host attach for <ship-viewer>: role landmark, src identity, child event re-emit.
// Lives outside the .sail controller so the sail-pure-region typecheck never
// names DOM types.

import { emit } from '@ship/sail-runtime'
import type { Signal } from '@ship/sail-runtime'
import type { ViewerSource } from './media'

export function attachViewer(
  host: HTMLElement,
  srcSig: Signal<ViewerSource | null>,
): () => void {
  if (!host.hasAttribute('role')) host.setAttribute('role', 'region')

  Object.defineProperty(host, 'src', {
    configurable: true,
    get: () => srcSig(),
    set: (s: ViewerSource | null) => { srcSig.set(s) },
  })

  const reemit = (name: 'sail-viewer-load' | 'sail-viewer-error') => (e: Event) => {
    if (e.target === host) return
    emit(host, name, (e as CustomEvent).detail)
  }
  const onLoad = reemit('sail-viewer-load')
  const onError = reemit('sail-viewer-error')

  const sr = host.shadowRoot ?? host
  for (const n of ['sail-viewer-load', 'sui-viewer-load'] as const) sr.addEventListener(n, onLoad)
  for (const n of ['sail-viewer-error', 'sui-viewer-error'] as const) sr.addEventListener(n, onError)

  return () => {
    for (const n of ['sail-viewer-load', 'sui-viewer-load'] as const) sr.removeEventListener(n, onLoad)
    for (const n of ['sail-viewer-error', 'sui-viewer-error'] as const) sr.removeEventListener(n, onError)
  }
}
