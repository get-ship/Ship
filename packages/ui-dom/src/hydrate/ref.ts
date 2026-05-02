// `bind:this={ref}` hydrator — sets the ref signal to the SSR'd element.
//
// The compiler resolves the ref signal at emit time and passes it in.
// On unmount the signal is reset to `null`.

import type { Signal } from '@ship/ui-runtime'

export function hydrateRef(
  el: Element,
  sig: Signal<Element | null>,
): () => void {
  sig.set(el)
  return () => {
    sig.set(null)
  }
}
