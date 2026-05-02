// Per-marker wirer + subtree-rewire helper.
//
// Phase 2 simplifies the dispatch surface: control-flow boundaries
// (`if` / `for` / `match` / `await`) no longer surface in the manifest.
// They're wired through `wireBoundaries` (see `./boundaries`), which
// invokes per-boundary handlers emitted on the view's setup return.
// `wireMarker` only handles leaf bindings + `while-loading` /
// `while-error` (still client-side, still in the manifest).
//
// `wireSubtree` walks a subtree's leaf markers — used historically by
// the deleted `wireBoundary` to re-wire freshly mounted SSR'd content.
// Phase 2 mounts boundary contents through the compiler-emitted
// helpers (`bindIf` / `bindFor`) which build DOM with inline reactive
// bindings — no manifest walk needed. `wireSubtree` is retained for
// the `while-loading` / `while-error` boundaries which still attach
// leaves through the manifest.

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
import { wireWhileLoading, wireWhileError } from './while-boundaries'
import { indexBindings, walkMarkers, type MarkerHit } from './manifest'

/** Threaded through every wirer so while-* boundaries can rewire on swap. */
export interface WireContext {
  byKey: Map<string, Binding>
  scope: Record<string, unknown>
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
 * Used by the `while-loading` / `while-error` boundaries to rewire
 * leaves inside their freshly mounted active branch on signal change.
 * `attach()` itself uses `walkMarkers` directly so the leaf walk and
 * the boundary-handler walk can be cleanly ordered.
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
