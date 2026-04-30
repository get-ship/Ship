// Two-way binding: `bind:value`.
//
// Compiler-enforced shape: `expr` is a single identifier that resolves
// to a writable Signal in scope. We wire two directions:
//   - DOM → signal: `input` event writes `el.value` back via `.set()`
//   - signal → DOM: `effect()` reads the signal and assigns `.value`
//
// The disposer tears down both paths.

import { effect, type Signal } from '@ship-set/runtime'

export function wireBindValue(
  el: HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement,
  scope: Record<string, unknown>,
  expr: string,
): () => void {
  const sig = scope[expr]
  if (!isWritableSignal(sig)) {
    console.warn(
      '[ship-set/attach] bind:value expects scope.' + expr + ' to be a writable signal.',
    )
    return () => {}
  }
  const writable = sig as Signal<unknown>
  const onInput = (): void => {
    writable.set((el as HTMLInputElement).value)
  }
  el.addEventListener('input', onInput)
  const dispose = effect(() => {
    const v = writable()
    const next = v === null || v === undefined ? '' : String(v)
    if ((el as HTMLInputElement).value !== next) {
      ;(el as HTMLInputElement).value = next
    }
  })
  return () => {
    el.removeEventListener('input', onInput)
    dispose()
  }
}

function isWritableSignal(v: unknown): v is Signal<unknown> {
  return (
    typeof v === 'function' &&
    'set' in (v as object) &&
    typeof (v as unknown as { set: unknown }).set === 'function'
  )
}
