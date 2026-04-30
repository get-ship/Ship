// <use-shell> — read-only access to host context (theme, current user,
// app metadata).
//
// This is the seam where view code reads ambient state owned by the
// host page (the Ship shell). The shell exposes a `window.__shipShell`
// global; the hook reads it once at mount and exposes it to the view
// via the `as=` identifier.
//
// Example:
//   <use-shell as="shell" />
//
//   <header class="theme-{shell.theme}">
//     {shell.currentUser?.name}
//   </header>
//
// The shape of the shell context is intentionally loose at v1 — fields
// vary by host. Once Ship's shell publishes a stable schema, this hook
// can narrow to a typed view; for now, authors get `Record<string, unknown>`.

import type { TemplateHook, TemplateHookContext } from '../controller'

/** Shape the shell exposes via `window.__shipShell`. Loose by design. */
export interface ShellContext {
  /** Active theme name (`'light' | 'dark' | string`). */
  theme?: string
  /** Currently signed-in user, if any. */
  currentUser?: { id: string; name?: string; [k: string]: unknown }
  /** App metadata (name, version, etc.). */
  app?: { name?: string; version?: string; [k: string]: unknown }
  /** Anything else the host exposes. */
  [key: string]: unknown
}

declare global {
  interface Window {
    __shipShell?: ShellContext
  }
}

interface ShellAttrs extends Record<string, unknown> {
  /** Identifier the hook contributes to scope (set via `as="X"`). */
  as: string
}

export const useShellHook: TemplateHook<ShellAttrs, ShellContext> = {
  name: 'shell',
  attrs: { as: 'literal' },
  factory: (_ctx: TemplateHookContext<ShellAttrs>): ShellContext => {
    // Read-once snapshot. The host shell can publish updates by
    // emitting a custom event; v1 doesn't subscribe — the hook can
    // grow signal-backed access later if a real view needs it.
    return (typeof window !== 'undefined' && window.__shipShell) || {}
  },
}
