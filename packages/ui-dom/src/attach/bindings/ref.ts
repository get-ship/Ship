// `bind:this={var}` — attach-time wirer.
//
// In view mode the ref signal does not exist on `$scope` until the
// attach pass resolves the SSR'd element. The compiler records the
// scope variable name in `binding.expr`; this wirer creates a
// `signal(el)` and writes it onto `scope[var]`, then returns a
// disposer that resets the signal to `null` on unmount.
//
// Components do not go through this path — their compiled setup body
// emits `signal(el)` inline, since the element is in hand at template
// construction time.

import { signal, type Signal } from '@ship/ui-runtime'

export function wireRef(
  el: Element,
  scope: Record<string, unknown>,
  varName: string,
): () => void {
  const existing = scope[varName]
  let ref: Signal<Element | null>
  if (isWritableSignal(existing)) {
    ;(existing as Signal<unknown>).set(el)
    ref = existing as Signal<Element | null>
  } else {
    ref = signal<Element | null>(el)
    scope[varName] = ref
  }
  return () => {
    ref.set(null)
  }
}

function isWritableSignal(v: unknown): v is Signal<unknown> {
  return (
    typeof v === 'function' &&
    'set' in (v as object) &&
    typeof (v as unknown as { set: unknown }).set === 'function'
  )
}
