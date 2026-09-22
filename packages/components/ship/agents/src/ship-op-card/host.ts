/** Host emit for <ship-op-card> — keep DOM types out of the .sail controller. */

export function emitToggle(host: Element | undefined, open: boolean): void {
  if (!host) return
  host.toggleAttribute('open', open)
  const g = globalThis as {
    CustomEvent?: new (n: string, i?: { detail?: unknown; bubbles?: boolean; composed?: boolean }) => unknown
  }
  if (!g.CustomEvent) return
  host.dispatchEvent(
    new g.CustomEvent('ship-toggle', { detail: { open }, bubbles: true, composed: true }) as Event,
  )
}
