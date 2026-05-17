// edit-mode.ts — per-view edit-mode UI state.
//
// View authoring (the per-view "edit mode") needs a single, shared bit
// per (project, view) pair so the shell can decide whether to render the
// dev panel + tabs and any view can read the same state without going
// through chrome events.
//
// State model
//   - Each (project, view) has an independent boolean signal.
//   - State persists across reloads via localStorage under
//     `ship.edit-mode.<project>.<view>`.
//   - Entering/exiting fires backend POSTs to
//     `/api/v1/views/<name>/edit-mode/{enter,exit}`. The body includes
//     the current session id (looked up from window.__shipSessionId,
//     which kernel-bridge/setup-wizard writes at boot).
//
// The signals are memoized: calling `editModeForView('foo')` twice
// returns the same Signal instance, so multiple consumers in the shell
// + the active view auto-update together.

import { signal, type Signal } from './signal'

const SIGNALS = new Map<string, Signal<boolean>>()

interface EditModeWindow extends Window {
  __shipProjectId?: string
  __shipSessionId?: string
}

function projectId(): string {
  if (typeof window === 'undefined') return 'default'
  return (window as EditModeWindow).__shipProjectId ?? 'default'
}

function sessionId(): string | null {
  if (typeof window === 'undefined') return null
  return (window as EditModeWindow).__shipSessionId ?? null
}

function storageKey(name: string): string {
  return `ship.edit-mode.${projectId()}.${name}`
}

function readPersisted(name: string): boolean {
  if (typeof localStorage === 'undefined') return false
  try { return localStorage.getItem(storageKey(name)) === '1' }
  catch { return false }
}

function writePersisted(name: string, on: boolean): void {
  if (typeof localStorage === 'undefined') return
  try {
    if (on) localStorage.setItem(storageKey(name), '1')
    else localStorage.removeItem(storageKey(name))
  } catch { /* best-effort */ }
}

async function postLifecycle(name: string, action: 'enter' | 'exit'): Promise<void> {
  if (typeof fetch === 'undefined') return
  const body = JSON.stringify({ session_id: sessionId() })
  try {
    await fetch(`/api/v1/views/${encodeURIComponent(name)}/edit-mode/${action}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      credentials: 'same-origin',
      body,
    })
  } catch { /* best-effort — backend may not be up yet */ }
}

/** Return the (memoized) Signal<boolean> for whether edit-mode is on
 *  for `name`. Subsequent calls with the same name return the same
 *  signal instance. */
export function editModeForView(name: string): Signal<boolean> {
  const k = storageKey(name)
  const existing = SIGNALS.get(k)
  if (existing) return existing
  const sig = signal<boolean>(readPersisted(name))
  SIGNALS.set(k, sig)
  return sig
}

/** Flip edit-mode on/off for `name`. Persists locally, fires the
 *  lifecycle POST, and dispatches `sui:edit-mode:change` on document so
 *  shell chrome (which doesn't run inside a tracked effect) can react. */
export function toggleEditMode(name: string): boolean {
  const sig = editModeForView(name)
  const next = !sig()
  setEditMode(name, next)
  return next
}

/** Set edit-mode explicitly. No-op when already at the target value. */
export function setEditMode(name: string, on: boolean): void {
  const sig = editModeForView(name)
  if (sig() === on) return
  sig.set(on)
  writePersisted(name, on)
  void postLifecycle(name, on ? 'enter' : 'exit')
  if (typeof document !== 'undefined') {
    document.dispatchEvent(new CustomEvent('sui:edit-mode:change', {
      detail: { view: name, on },
      bubbles: true,
    }))
  }
}

/** Test-only: drop the memoized signal cache. Used by unit tests so
 *  successive cases start from a clean slate. Not exported from
 *  `index.ts` — internal. */
export function __resetEditModeRegistryForTests(): void {
  SIGNALS.clear()
  ANNOTATION_SIGNALS.clear()
}

// ── Annotation mode (sub-toggle inside edit mode) ────────────────────
//
// Annotation mode is a SECOND boolean per (project, view), only
// meaningful when edit-mode is also on. ON ⇒ the overlay binds
// hover/click handlers so the user picks elements to annotate; OFF ⇒
// the view is interactive normally even though the dev panel + tabs
// are still in edit-mode shape.
//
// Session-only: not persisted to localStorage so a reload always
// starts annotation mode off (per the design call D2 — user wants
// each edit session to start clean).

const ANNOTATION_SIGNALS = new Map<string, Signal<boolean>>()

export function annotationModeForView(name: string): Signal<boolean> {
  const k = `ship.annotation-mode.${projectId()}.${name}`
  const existing = ANNOTATION_SIGNALS.get(k)
  if (existing) return existing
  const sig = signal<boolean>(false)
  ANNOTATION_SIGNALS.set(k, sig)
  return sig
}

export function toggleAnnotationMode(name: string): boolean {
  const sig = annotationModeForView(name)
  const next = !sig()
  setAnnotationMode(name, next)
  return next
}

export function setAnnotationMode(name: string, on: boolean): void {
  const sig = annotationModeForView(name)
  if (sig() === on) return
  // Annotation mode requires edit mode on; refuse to flip on otherwise.
  if (on && !editModeForView(name)()) return
  sig.set(on)
  if (typeof document !== 'undefined') {
    document.dispatchEvent(new CustomEvent('sui:annotation-mode:change', {
      detail: { view: name, on },
      bubbles: true,
    }))
  }
}
