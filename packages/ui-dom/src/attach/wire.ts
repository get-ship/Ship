// Per-marker wirer + subtree-rewire + scope-serialization helpers.
//
// Phase E-2 introduces three concerns the v1 attach.ts collapsed onto
// itself:
//  1. dispatching one marker hit to the right per-kind wirer
//     (`wireMarker`)
//  2. wiring every marker under an arbitrary subtree on demand
//     (`wireSubtree`) — used by control-flow swap to attach the freshly
//     mounted DOM
//  3. building the JSON `init` body for a fragment-render request
//     (`serializeScope`) — pulls signal values out of the live scope so
//     the daemon walks the IR with what the runtime currently has.
//
// All three move out of attach.ts to keep that file under the 300-line
// cap and to give boundaries.ts a clean dependency story.

import type { Binding } from '../attach'
import { wireText } from './bindings/text'
import { wireOn } from './bindings/event'
import { wireBindValue } from './bindings/bind-value'
import { wireBindChecked } from './bindings/bind-checked'
import { wireRef } from './bindings/ref'
import { wireAttr } from './bindings/attr'
import { wireClass } from './bindings/class'
import { wireStyle } from './bindings/style'
import { wireAttrDynamic } from './bindings/attr-dynamic'
import { wireFormMutation } from './bindings/form-mutation'
import { wireBoundary } from './boundaries'
import { wireWhileLoading, wireWhileError } from './while-boundaries'
import { indexBindings, walkMarkers, type MarkerHit } from './manifest'

/** Threaded through every wirer so boundaries can swap and re-attach. */
export interface WireContext {
  byKey: Map<string, Binding>
  scope: Record<string, unknown>
  /** Names of `<state>` + `<data>` declarations — keys to serialize on swap. */
  initKeys: Set<string>
  /** View name. When absent, control-flow swap is disabled (the runtime
   *  warns and leaves the boundary as-is). */
  viewName?: string
}

/** Wire one marker hit. Returns a disposer (or undefined to skip). */
export function wireMarker(
  hit: MarkerHit,
  ctx: WireContext,
): (() => void) | undefined {
  if (hit.kind === 'text') {
    return wireText(hit.open, hit.close, ctx.scope, hit.binding.expr)
  }
  const { binding, el } = hit
  const kind = binding.kind
  if (kind === 'text') return undefined
  if (kind === 'bind:value') {
    return wireBindValue(
      el as HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement,
      ctx.scope,
      binding.expr,
    )
  }
  if (kind === 'bind:checked') {
    return wireBindChecked(el as HTMLInputElement, ctx.scope, binding.expr)
  }
  if (kind === 'bind:this') {
    return wireRef(el, ctx.scope, binding.expr)
  }
  if (kind === 'attr-dynamic') {
    return wireAttrDynamic(el, ctx.scope, binding.expr)
  }
  if (kind === 'if' || kind === 'for' || kind === 'match' || kind === 'await') {
    return wireBoundary(el, binding, ctx)
  }
  if (kind === 'while-loading') {
    return wireWhileLoading(el, binding, ctx)
  }
  if (kind === 'while-error') {
    return wireWhileError(el, binding, ctx)
  }
  if (kind.startsWith('on:')) {
    return wireOn(el, kind.slice(3), ctx.scope, binding.expr)
  }
  if (kind.startsWith('attr:')) {
    return wireAttr(el, kind.slice(5), ctx.scope, binding.expr)
  }
  if (kind.startsWith('class:')) {
    return wireClass(el, kind.slice(6), ctx.scope, binding.expr)
  }
  if (kind.startsWith('style:')) {
    return wireStyle(el as HTMLElement, kind.slice(6), ctx.scope, binding.expr)
  }
  if (kind === 'mutation') {
    return wireFormMutation(el, binding.expr, ctx.scope)
  }
  console.warn('[ship-set/attach] unknown binding kind:', kind)
  return undefined
}

/**
 * Walk every marker under `root` and wire each via `wireMarker`.
 * Returns a single disposer that tears down every wirer attached
 * during this pass.
 *
 * When called from `wireBoundary`, pass `skipRoot: true` so the
 * boundary's own marker isn't re-wired (which would loop back into
 * `wireBoundary` indefinitely). The top-level `attach()` call uses
 * `skipRoot: false` because the SSR walker never puts a binding marker
 * on the root container.
 */
export function wireSubtree(
  root: HTMLElement,
  ctx: WireContext,
  opts: { skipRoot?: boolean } = {},
): () => void {
  const cleanups: Array<() => void> = []
  for (const hit of walkMarkers(root, ctx.byKey, opts.skipRoot !== undefined ? { skipRoot: opts.skipRoot } : {})) {
    const c = wireMarker(hit, ctx)
    if (c) cleanups.push(c)
  }
  return () => {
    for (const c of cleanups) {
      try {
        c()
      } catch (err) {
        console.error('[ship-set/attach] subtree cleanup threw:', err)
      }
    }
  }
}

export { indexBindings }

/** Names skipped by `serializeScope` — they're either runtime callables
 *  the daemon can't accept, or already on the scope of every render. */
const SKIP_KEYS = new Set(['route', 'ship', 'navigate'])

/**
 * Build the JSON body the daemon's fragment-render endpoint expects.
 *
 * Keys come from two sources:
 *   1. `initKeys` — names of declared `<state>` / `<data>` signals.
 *      Always included so the daemon's initial-scope builder seeds
 *      against the current runtime values.
 *   2. Other entries in `scope` whose value carries the `__peek` brand
 *      — derived signals from `controller({derived: …})`. Calling them
 *      with no args is safe (signals + deriveds both return cached
 *      values); event handlers don't have `__peek`, so we skip them.
 *
 * Anything that throws on read or doesn't JSON-serialize cleanly is
 * dropped from the payload — the daemon's eval falls back to "name not
 * in scope" → empty fragment, which the runtime handles.
 */
export function serializeScope(
  scope: Record<string, unknown>,
  initKeys: Set<string>,
): Record<string, unknown> {
  const out: Record<string, unknown> = {}
  for (const k of initKeys) {
    pull(scope, k, out)
  }
  for (const [k, v] of Object.entries(scope)) {
    if (k in out) continue
    if (SKIP_KEYS.has(k)) continue
    if (typeof v === 'function' && '__peek' in (v as object)) {
      try {
        out[k] = (v as () => unknown)()
      } catch {
        // Skip non-serializable / threw on read.
      }
    }
  }
  return out
}

function pull(
  scope: Record<string, unknown>,
  k: string,
  out: Record<string, unknown>,
): void {
  const v = scope[k]
  if (typeof v === 'function') {
    try {
      out[k] = (v as () => unknown)()
    } catch {
      // Drop on read failure — daemon will fall back to default.
    }
  } else if (v !== undefined) {
    out[k] = v
  }
}
