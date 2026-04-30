// Two-way binding: `bind:checked`.

import { effect, type Signal } from '@ship-set/runtime'

export function wireBindChecked(
  el: HTMLInputElement,
  scope: Record<string, unknown>,
  expr: string,
): () => void {
  const sig = scope[expr]
  if (!isWritableSignal(sig)) {
    console.warn(
      '[ship-set/attach] bind:checked expects scope.' + expr + ' to be a writable signal.',
    )
    return () => {}
  }
  const writable = sig as Signal<unknown>
  const onChange = (): void => {
    writable.set(el.checked)
  }
  el.addEventListener('change', onChange)
  const dispose = effect(() => {
    const v = writable()
    const next = !!v
    if (el.checked !== next) el.checked = next
  })
  return () => {
    el.removeEventListener('change', onChange)
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
