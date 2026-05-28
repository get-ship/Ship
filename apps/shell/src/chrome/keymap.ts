// Hub keyboard map.
//
// One global keydown handler — every shortcut the shell owns lives here.
// View-internal shortcuts live in their own controllers; the shell only
// claims keys that act on the chrome (command bar, dock, rail) or the
// current view's lifecycle (edit mode, Esc-to-blank).
//
// Bindings:
//   ⌘K / Ctrl+K    open the command palette (sui-open-command-palette event)
//   ⌘P / Ctrl+P    open the command bar in `find` mode
//   ⌘J / Ctrl+J    toggle the panel dock
//   ⌘B / Ctrl+B    toggle the rail (sui:rail:toggle event)
//   ⌘N / Ctrl+N    open the create dialog (ship:create:open event)
//   ⌘. / Ctrl+.    navigate to /studio
//   ⌘? / Ctrl+?    open help (sui-open-help event)
//   ⌃` (Ctrl+`)    toggle the terminal panel (sui-toggle-terminal event)
//   Alt+E          per-view edit-mode toggle (Alt to avoid Chrome devtools
//                  and Firefox tab-search collisions on ⌘⇧E / Ctrl+Shift+E)
//   Alt+Shift+E    annotation-mode toggle (sub-toggle inside edit mode)
//   Esc            close any open <sui-dialog>/<sui-drawer>; else if a view
//                  is mounted, return to the blank canvas

import type { Route } from './routing-types'

interface CommandBar extends HTMLElement {
  show(mode?: string, query?: string): void
  hide(): void
  toggle(mode?: string): void
  setMode(mode: string): void
}

/** Context bag the shell passes in so the keymap stays state-free. Every
 *  binding that needs to mutate shell state (mount lifecycle, route) calls
 *  back into these closures. */
export interface KeymapContext {
  navigate(path: string): void
  /** Current route, or `null` when on the blank canvas. */
  currentRoute(): Route | null
  /** Toggle the per-view edit mode for a specific view name. */
  toggleEditMode(view: string): void
  /** Toggle annotation mode (sub-toggle inside edit mode). */
  toggleAnnotationMode(view: string): void
  /** Tear down the current mount (Esc-to-blank path). */
  disposeCurrentMount(): void
  /** Re-publish the live scope (Esc clears it). */
  publishScope(scope: Record<string, unknown> | null): void
  /** Render the default landing UI into the canvas. */
  showLanding(slot: HTMLElement): void
  /** Resolve the panel dock. */
  getDock(): { toggle(): void; activate(tab: string): void; hide(): void } | null
}

function getCommandBar(): CommandBar | null {
  return document.getElementById('ship-command-bar') as CommandBar | null
}

/** Register the global keymap + the chrome-event bridges that pair with it
 *  (sui-toggle-terminal → dock activate+toggle; sui:dock:close → dock hide).
 *  Returns a dispose fn for tests; production callers can ignore it.  */
export function registerKeymap(ctx: KeymapContext): () => void {
  const onKey = (e: KeyboardEvent): void => {
    const mod = e.metaKey || e.ctrlKey
    // `e.key` reports the produced character: Chromium yields lowercase 'b'
    // for Ctrl+B without Shift, but some automation drivers (and a few
    // platform keyboard layouts) emit uppercase. Normalize so shortcuts
    // trigger regardless of caller.
    const key = e.key.toLowerCase()

    if (mod && key === 'j') {
      e.preventDefault()
      ctx.getDock()?.toggle()
      return
    }

    if (mod && key === 'k') {
      e.preventDefault()
      // ⌘K is the command palette — the shell's primary action surface. It
      // and the blank-canvas hint emit the bare `sui-open-command-palette`
      // intent, which the shell routes to the one `<sui-command-bar>`. ⌘P
      // opens the same bar in `find` mode.
      document.dispatchEvent(new CustomEvent('sui-open-command-palette', { bubbles: true }))
      return
    }

    if (mod && key === 'p') {
      e.preventDefault()
      getCommandBar()?.show('find')
      return
    }

    // ⌘. / Ctrl+. — open Studio (the views/studio deck view).
    if (mod && e.key === '.') {
      e.preventDefault()
      ctx.navigate('/studio')
      return
    }

    // ⌘? / Ctrl+? — open help (stub event; help surface is a future job).
    if (mod && (e.key === '?' || (e.shiftKey && key === '/'))) {
      e.preventDefault()
      document.dispatchEvent(new CustomEvent('sui-open-help', { bubbles: true }))
      return
    }

    // ⌃` — toggle terminal panel. Uses ctrlKey explicitly (not Meta) per
    // the spec: ⌃ is the macOS Control key, not Command.
    if (e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey && e.key === '`') {
      e.preventDefault()
      document.dispatchEvent(new CustomEvent('sui-toggle-terminal', { bubbles: true }))
      return
    }

    if (e.altKey && !mod && !e.shiftKey && key === 'e') {
      e.preventDefault()
      // Per-view edit mode: flip the persisted signal for the current view.
      // The edit-mode chrome (chrome/edit-mode.ts) reacts and swaps the
      // dock tabs + projects the view-authoring surfaces. Alt+E replaced
      // the prior ⌘⇧E / Ctrl+Shift+E binding because of browser collisions
      // (Firefox tab search, Chrome devtools toggle).
      const cur = ctx.currentRoute()
      if (cur) ctx.toggleEditMode(cur.view)
      return
    }

    if (e.altKey && e.shiftKey && !mod && key === 'e') {
      e.preventDefault()
      // Annotation mode (sub-toggle inside edit mode). No-op when edit mode
      // is off — the runtime guards this; the keyboard binding just relays.
      const cur = ctx.currentRoute()
      if (cur) ctx.toggleAnnotationMode(cur.view)
      return
    }

    if (mod && key === 'b') {
      e.preventDefault()
      document.dispatchEvent(new CustomEvent('sui:rail:toggle', { bubbles: true }))
      return
    }

    if (mod && key === 'n') {
      e.preventDefault()
      document.dispatchEvent(new CustomEvent('ship:create:open', { bubbles: true }))
      return
    }

    if (e.key === 'Escape') {
      // Native <dialog> handles Esc already. Backstop: close any open
      // sui-dialog or sui-drawer that may not be using the native dialog
      // element.
      let closedAny = false
      for (const el of document.querySelectorAll<HTMLElement & { close(): void }>(
        'sui-dialog[open], sui-drawer[open]',
      )) {
        try { el.close(); closedAny = true } catch { /* ignore — native handles it */ }
      }
      // If nothing dismissable was open and a view is currently mounted,
      // return to the blank canvas. The spec calls for Esc to clear the
      // active view's content from the canvas slot when no other modal
      // surface is consuming the key.
      const cur = ctx.currentRoute()
      if (!closedAny && cur) {
        e.preventDefault()
        history.pushState({}, '', '/')
        const slot = document.getElementById('ship-content')
        if (slot) {
          ctx.disposeCurrentMount()
          ctx.publishScope(null)
          ctx.showLanding(slot)
        }
      }
    }
  }

  // Bridge `sui-toggle-terminal` → panel-dock toggle. The terminal lives
  // inside the dock; the canonical "terminal" UX is the dock with its
  // terminal tab activated. activate() is idempotent.
  const onTerminal = (): void => {
    const dock = ctx.getDock()
    if (!dock) return
    dock.activate('terminal')
    dock.toggle()
  }

  // Close event from the dock.
  const onDockClose = (): void => { ctx.getDock()?.hide() }

  document.addEventListener('keydown', onKey)
  document.addEventListener('sui-toggle-terminal', onTerminal)
  document.addEventListener('sui:dock:close', onDockClose, true)

  return () => {
    document.removeEventListener('keydown', onKey)
    document.removeEventListener('sui-toggle-terminal', onTerminal)
    document.removeEventListener('sui:dock:close', onDockClose, true)
  }
}
