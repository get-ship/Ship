// Host event helpers for <ship-game-frame>. Kept out of the controller so the
// sail-pure-region typecheck never names DOM types.

export type Host = {
  dispatchEvent: (e: unknown) => boolean
}

export function emitAction(host: Host | undefined, action: string): void {
  const g = globalThis as {
    CustomEvent?: new (n: string, i?: { detail?: unknown; bubbles?: boolean; composed?: boolean }) => unknown
  }
  if (!g.CustomEvent || !host) return
  host.dispatchEvent(new g.CustomEvent('ship-action', {
    detail: { action }, bubbles: true, composed: true,
  }))
}
