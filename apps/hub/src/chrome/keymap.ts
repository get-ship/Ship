// Hub keyboard map.
//
// One global keydown handler for the chrome-level shortcuts the hub owns.
// View-internal shortcuts live in their own controllers; this only claims
// keys that act on chrome surfaces the hub actually mounts.
//
// Bindings:
//   ⌘P / Ctrl+P    open the command palette in `find` mode (#ship-command-palette)
//   ⌘N / Ctrl+N    open the create surface (ship:create:open → palette create mode)
//   ⌘. / Ctrl+.    navigate to /studio
//   Esc            close any open <sui-dialog>/<sui-drawer> as a backstop
//
// ⌘K is owned by <ship-command-palette> itself (its controller binds a
// document-level keydown); the hub does not re-bind it here — a second
// handler would re-introduce the open→close race that single ownership
// avoids.

interface CommandPalette extends HTMLElement {
  show(mode?: string, query?: string): void
  hide(): void
  toggle(mode?: string): void
  setMode(mode: string): void
}

/** Context bag the host passes in so the keymap stays state-free. The hub's
 *  only chrome-mutating binding is navigation; the router's `navigate` is
 *  injected rather than imported so the keymap stays decoupled and testable. */
export interface KeymapContext {
  navigate(path: string): void
}

function getCommandPalette(): CommandPalette | null {
  return document.getElementById('ship-command-palette') as CommandPalette | null
}

/** Register the global keymap. Returns a dispose fn for tests; production
 *  callers can ignore it. */
export function registerKeymap(ctx: KeymapContext): () => void {
  const onKey = (e: KeyboardEvent): void => {
    const mod = e.metaKey || e.ctrlKey
    // `e.key` reports the produced character: Chromium yields lowercase 'p'
    // for Ctrl+P without Shift, but some automation drivers (and a few
    // platform keyboard layouts) emit uppercase. Normalize so shortcuts
    // trigger regardless of caller.
    const key = e.key.toLowerCase()

    if (mod && key === 'p') {
      e.preventDefault()
      getCommandPalette()?.show('find')
      return
    }

    if (mod && key === 'n') {
      e.preventDefault()
      // The command palette listens for `ship:create:open` and opens in
      // create mode.
      document.dispatchEvent(new CustomEvent('ship:create:open', { bubbles: true }))
      return
    }

    // ⌘. / Ctrl+. — open Studio (the /studio route tree).
    if (mod && e.key === '.') {
      e.preventDefault()
      ctx.navigate('/studio')
      return
    }

    if (e.key === 'Escape') {
      // Native <dialog> handles Esc already. Backstop: close any open
      // sui-dialog or sui-drawer that may not be using the native dialog
      // element.
      for (const el of document.querySelectorAll<HTMLElement & { close(): void }>(
        'sui-dialog[open], sui-drawer[open]',
      )) {
        try { el.close() } catch { /* ignore — native handles it */ }
      }
    }
  }

  document.addEventListener('keydown', onKey)

  return () => {
    document.removeEventListener('keydown', onKey)
  }
}
