// HUD data-binding: descendants of <sui-hud> with `data-bind="<key>"`
// reactively follow the stage's named signal. The element's `data-bind-attr`
// (or default `value`) is set whenever the signal changes.
//
// SUI signals are callable getters (`signal()` reads the current value);
// `effect(fn)` registers `fn` as a reactive observer. Reading a signal
// inside the effect tracks it; subsequent writes re-run `fn`.

import { effect } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'

export function wireHudBindings(
  host: Element,
  bound: Map<string, Signal<unknown>>,
): () => void {
  const stops: Array<() => void> = []
  const targets = host.querySelectorAll('sui-hud [data-bind]')
  for (const el of targets) {
    const name = el.getAttribute('data-bind')
    if (!name) continue
    const sig = bound.get(name)
    if (!sig) continue
    // Default to writing textContent (covers <span>, <div>, etc); only switch
    // to attribute writes when the author explicitly asks via data-bind-attr
    // (e.g. `<progress value>` or a CSS custom prop carrier).
    const targetAttr = el.getAttribute('data-bind-attr')
    const stop = effect(() => {
      const v = sig()
      const text = v === null || v === undefined ? '' : String(v)
      if (targetAttr) {
        if (text === '') el.removeAttribute(targetAttr)
        else el.setAttribute(targetAttr, text)
      } else {
        el.textContent = text
      }
    })
    stops.push(stop)
  }
  return () => {
    for (const stop of stops) stop()
    stops.length = 0
  }
}
