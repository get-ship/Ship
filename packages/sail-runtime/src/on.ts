// Event handler wiring with modifier support.
//
// The Sail compiler turns `on:click|preventDefault={fn}` into a call
// to this `on()` helper at hydrate time. Authors don't import this
// directly — they write the declarative form in templates.
//
// See .artifacts/specs/sail/01-language-spec.md § Event handlers.

import type { Disposer } from './bind'

export interface OnOptions {
  /** Modifier list, in order applied. */
  modifiers?: Modifier[]
  /** addEventListener options. */
  capture?: boolean
  passive?: boolean
  once?: boolean
}

export interface Modifier {
  kind:
    | 'preventDefault'
    | 'stopPropagation'
    | 'stopImmediatePropagation'
    | 'self'
    | 'trusted'
    | 'key'
    | 'debounce'
    | 'throttle'
  arg?: string
}

/**
 * Attach an event listener to `el`. Returns a disposer that removes
 * the listener and clears any pending debounce/throttle timers.
 *
 * Modifier semantics:
 * - `preventDefault` — calls `event.preventDefault()` before handler
 * - `stopPropagation` — calls `event.stopPropagation()` before handler
 * - `stopImmediatePropagation` — same
 * - `self` — only fires when `event.target === el`
 * - `trusted` — only fires for `event.isTrusted` events (user-initiated)
 * - `key:NAME` — only fires when `event.key === NAME` (e.g., `key:Enter`,
 *   `key:Escape`); supports compound modifiers via `key:meta+j`
 * - `debounce:MS` — coalesces rapid fires; fires once `MS` ms after last call
 * - `throttle:MS` — rate-limits to one fire per `MS` ms
 */
export function on(
  el: EventTarget,
  event: string,
  handler: (event: Event) => unknown,
  opts: OnOptions = {},
): Disposer {
  const modifiers = opts.modifiers ?? []
  let timer: ReturnType<typeof setTimeout> | undefined
  let throttleArmed = true

  const wrapped = (e: Event) => {
    // Pre-handler modifiers
    for (const m of modifiers) {
      switch (m.kind) {
        case 'preventDefault':
          e.preventDefault()
          break
        case 'stopPropagation':
          e.stopPropagation()
          break
        case 'stopImmediatePropagation':
          e.stopImmediatePropagation()
          break
        case 'self':
          if (e.target !== el) return
          break
        case 'trusted':
          if (!e.isTrusted) return
          break
        case 'key': {
          if (!matchesKey(e as KeyboardEvent, m.arg ?? '')) return
          break
        }
      }
    }

    // Debounce / throttle take priority over the actual call
    const debounceMod = modifiers.find((m) => m.kind === 'debounce')
    const throttleMod = modifiers.find((m) => m.kind === 'throttle')

    if (debounceMod) {
      const ms = Number(debounceMod.arg) || 0
      if (timer !== undefined) clearTimeout(timer)
      timer = setTimeout(() => {
        timer = undefined
        handler(e)
      }, ms)
      return
    }

    if (throttleMod) {
      const ms = Number(throttleMod.arg) || 0
      if (!throttleArmed) return
      throttleArmed = false
      handler(e)
      timer = setTimeout(() => {
        throttleArmed = true
      }, ms)
      return
    }

    handler(e)
  }

  const listenerOpts: AddEventListenerOptions = {}
  if (opts.capture) listenerOpts.capture = true
  if (opts.passive) listenerOpts.passive = true
  if (opts.once) listenerOpts.once = true

  el.addEventListener(event, wrapped, listenerOpts)
  return () => {
    el.removeEventListener(event, wrapped, listenerOpts)
    if (timer !== undefined) clearTimeout(timer)
  }
}

/**
 * Match a keyboard event against a `key:` modifier argument.
 *
 * - Single key: `Enter`, `Escape`, `a`
 * - Compound (modifier+key): `meta+j`, `ctrl+shift+a`, `alt+x`
 * - Aliases: `cmd` ≡ `meta`, `mod` ≡ `meta` on mac else `ctrl`
 */
export function matchesKey(e: KeyboardEvent, spec: string): boolean {
  const parts = spec.split('+').map((p) => p.toLowerCase())
  const key = parts.pop() ?? ''
  const wantMeta = parts.includes('meta') || parts.includes('cmd')
  const wantCtrl = parts.includes('ctrl') || parts.includes('control')
  const wantShift = parts.includes('shift')
  const wantAlt = parts.includes('alt') || parts.includes('option')
  // `mod` = meta on darwin, ctrl elsewhere
  const wantMod = parts.includes('mod')
  const isMac =
    typeof navigator !== 'undefined' &&
    /Mac|iPhone|iPad|iPod/i.test(navigator.platform || navigator.userAgent || '')

  if (wantMeta && !e.metaKey) return false
  if (wantCtrl && !e.ctrlKey) return false
  if (wantShift && !e.shiftKey) return false
  if (wantAlt && !e.altKey) return false
  if (wantMod && !(isMac ? e.metaKey : e.ctrlKey)) return false

  return e.key.toLowerCase() === key
}
