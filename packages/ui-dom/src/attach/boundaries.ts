// Control-flow boundaries — `if` / `for` / `match` / `await`.
//
// **Phase E-2** — server round-trip swap-on-signal-change.
//
// When the boundary's discriminating expression changes (e.g.
// `agents().kind` flips from `loading` to `ok`), the runtime POSTs the
// current scope state plus the boundary's `b<N>` key to
// `/_ship-set/render-fragment/<viewName>`. The daemon walks the IR
// against that scope and returns the rendered body for the boundary —
// no surrounding wrapper, just the children. The runtime swaps the
// boundary's `<sui-{kind} key="bN">` element in-place with a fresh
// active wrapper and recursively
// wires every leaf marker the new HTML carries. The previous subtree's
// wirers are disposed before the new ones attach.
//
// Always-active swap shape: at runtime we have full control over the
// HTML being mounted, so we never use the `<template>` placeholder
// shape that the SSR walker emits for un-rendered branches. An empty
// active wrapper (`<div style="display:contents"></div>`) renders the
// same as the placeholder for the user (nothing) and gives the marker
// walker a single, unconditional shape to wire.
//
// Failures (4xx / 5xx / network error) log via `console.error` and
// leave the existing DOM in place — the runtime degrades to whatever
// the SSR rendered.

import { effect } from '@ship/ui-runtime'
import type { Binding } from '../attach'
import { compileExpr } from './eval'
import { serializeScope, wireSubtree, type WireContext } from './wire'

/**
 * Wire a control-flow boundary marker.
 *
 * `el` is the boundary element as found in the SSR'd DOM — either a
 * `<sui-{kind} key="bN" style="display:contents">` (active branch) or
 * `<sui-{kind} key="bN" hidden>` (placeholder). After the first swap the
 * closure-scoped `current` reference advances to the freshly mounted active
 * element.
 */
export function wireBoundary(
  el: Element,
  binding: Binding,
  ctx: WireContext,
): () => void {
  let current: Element = el
  let branchDispose: (() => void) | null = null

  // Wire any leaf bindings that already live inside the SSR'd active
  // branch (the placeholder has none). Tracked separately from the
  // condition's effect-disposer so a swap can replace just the body.
  if (isActive(current)) {
    branchDispose = wireSubtree(current as HTMLElement, ctx, { skipRoot: true })
  }

  const ev = binding.expr ? compileExpr(binding.expr) : null
  let firstRun = true
  let inflight = 0

  const condDispose = effect(() => {
    if (ev) {
      try {
        ev(ctx.scope)
      } catch {
        // Expression may not evaluate cleanly on first run; deps still
        // register via the read attempt. Subsequent firings still trigger
        // the swap (the daemon evaluates against the merged init).
      }
    }
    if (firstRun) {
      firstRun = false
      return
    }
    void runSwap()
  })

  function runSwap(): Promise<void> {
    if (!ctx.viewName) {
      console.error(
        '[ship-set/attach] cannot swap control-flow boundary ' +
          binding.key +
          ': attach() was called without a viewName option.',
      )
      return Promise.resolve()
    }
    const myFetch = ++inflight
    const init = serializeScope(ctx.scope, ctx.initKeys)
    const url =
      '/_ship-set/render-fragment/' + encodeURIComponent(ctx.viewName)
    return fetch(url, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ init, key: binding.key }),
    })
      .then(async (res) => {
        if (!res.ok) {
          const body = await res.text().catch(() => '<no body>')
          throw new Error(
            'fragment-render ' + res.status + ': ' + body.slice(0, 200),
          )
        }
        return (await res.json()) as { html: string }
      })
      .then((body) => {
        // A newer swap may have started while this one was in flight;
        // drop stale responses.
        if (myFetch !== inflight) return
        applySwap(body.html)
      })
      .catch((err) => {
        console.error(
          '[ship-set/attach] fragment render failed:',
          err,
          '\nbinding:',
          binding.key,
        )
      })
  }

  function applySwap(html: string): void {
    if (branchDispose) {
      try {
        branchDispose()
      } catch (err) {
        console.error('[ship-set/attach] branch dispose threw:', err)
      }
      branchDispose = null
    }
    const replacement = buildReplacement(current, binding, html)
    const parent = current.parentNode
    if (!parent) {
      console.error(
        '[ship-set/attach] boundary element detached from DOM; cannot swap',
      )
      return
    }
    parent.replaceChild(replacement, current)
    current = replacement
    branchDispose = wireSubtree(replacement, ctx, { skipRoot: true })
  }

  return () => {
    try {
      condDispose()
    } catch (err) {
      console.error('[ship-set/attach] boundary cond dispose threw:', err)
    }
    if (branchDispose) {
      try {
        branchDispose()
      } catch (err) {
        console.error('[ship-set/attach] branch dispose threw:', err)
      }
      branchDispose = null
    }
  }
}

function isActive(el: Element): boolean {
  return !el.hasAttribute('hidden')
}

/**
 * Build a fresh active wrapper carrying the rendered body. Parses the
 * fragment HTML through a `<template>` element — per the HTML spec
 * `<script>` inside `<template>.content` is inert, so we don't have to
 * worry about embedded scripts re-executing on swap. The
 * `Range.createContextualFragment` alternative requires the range's
 * context node to be in a document; we'd rather build the new wrapper
 * out-of-document and then substitute it in atomically.
 */
function buildReplacement(
  prior: Element,
  binding: Binding,
  html: string,
): HTMLElement {
  const owner = prior.ownerDocument ?? document
  const wrapper = owner.createElement('sui-' + binding.kind) as HTMLElement
  wrapper.setAttribute('key', binding.key)
  wrapper.style.display = 'contents'
  if (html) {
    const tpl = owner.createElement('template')
    tpl.innerHTML = html
    wrapper.appendChild(tpl.content)
  }
  return wrapper
}
