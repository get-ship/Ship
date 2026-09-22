// Host event helpers for <ship-arcade>. Kept out of the controller so the
// sail-pure-region typecheck never names DOM types.

export type Host = {
  dispatchEvent: (e: unknown) => boolean
}

export type KeyEv = {
  key: string
  preventDefault: () => void
}

export function emitSelect(host: Host | undefined, id: string): void {
  const g = globalThis as {
    CustomEvent?: new (n: string, i?: { detail?: unknown; bubbles?: boolean; composed?: boolean }) => unknown
  }
  if (!g.CustomEvent || !host) return
  host.dispatchEvent(new g.CustomEvent('ship-select', {
    detail: { game: id }, bubbles: true, composed: true,
  }))
}
