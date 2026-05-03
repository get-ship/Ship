// Event-handler hydrator (`on:<event>`).
//
// The compiler emits `hydrateOn(el, 'click', () => $scope.handler)` —
// each fire reads the getter so the handler can swap when the
// underlying signal changes (rare, but preserves the property authors
// expect when `on:` is conditional).

export function hydrateOn(
  el: Element,
  event: string,
  handlerGetter: () => unknown,
): () => void {
  const listener = (e: Event): void => {
    const handler = handlerGetter()
    if (typeof handler !== 'function') return
    try {
      ;(handler as (e: Event) => unknown)(e)
    } catch (err) {
      console.error('[ship-set/hydrate] on:' + event + ' handler threw:', err)
    }
  }
  el.addEventListener(event, listener)
  return () => el.removeEventListener(event, listener)
}
