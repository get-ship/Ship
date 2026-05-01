// While-loading / while-error boundary wirers.
//
// Unlike if/for/match, these boundaries render locally using render
// callbacks the compiler emitted onto the setup return object — no
// server round-trip needed. The signal drives show/hide; on transition
// the callback builds fresh DOM nodes with inline reactive bindings.

import { effect } from '@ship/ui-runtime'
import type { Binding } from '../attach'
import { wireSubtree, type WireContext } from './wire'

type WhileRender = () => { nodes: Node[]; cleanup: () => void }
type WhileErrorRender = (err: unknown) => { nodes: Node[]; cleanup: () => void }

function isActive(el: Element): boolean {
  return !el.hasAttribute('hidden')
}

/**
 * Wire a `{#while-loading of sig}` boundary found in SSR'd DOM.
 *
 * The render callback (`binding.renderKey`) lives in `ctx.scope` as
 * emitted by the compiler. On signal transitions, the render callback
 * builds fresh CSR nodes with inline reactive bindings — no
 * `wireSubtree` pass needed for re-renders.
 */
export function wireWhileLoading(
  el: Element,
  binding: Binding,
  ctx: WireContext,
): () => void {
  let current: Element = el
  let branchDispose: (() => void) | null = null

  if (isActive(current)) {
    branchDispose = wireSubtree(current as HTMLElement, ctx, { skipRoot: true })
  }

  const renderFn = binding.renderKey
    ? (ctx.scope[binding.renderKey] as WhileRender | undefined)
    : undefined

  const sig = ctx.scope[binding.expr] as
    | (() => { kind?: string })
    | undefined
  let firstRun = true

  const condDispose = effect(() => {
    let isLoading = true
    if (sig) {
      try {
        const v = sig()
        isLoading = !v || v.kind === 'loading'
      } catch { /* signal not ready; default to loading */ }
    }
    if (firstRun) { firstRun = false; return }
    applyTransition(isLoading)
  })

  function applyTransition(show: boolean): void {
    if (branchDispose) { try { branchDispose() } catch {} ; branchDispose = null }
    const parent = current.parentNode
    if (!parent) return
    const owner = current.ownerDocument ?? document
    const next = owner.createElement('sui-while-loading') as HTMLElement
    next.setAttribute('key', binding.key)
    if (show) {
      next.style.display = 'contents'
      if (renderFn) {
        const result = renderFn()
        for (const node of result.nodes) next.appendChild(node)
        branchDispose = () => { try { result.cleanup() } catch {} }
      }
    } else {
      next.setAttribute('hidden', '')
    }
    parent.replaceChild(next, current)
    current = next
  }

  return () => {
    try { condDispose() } catch {}
    if (branchDispose) { try { branchDispose() } catch {} ; branchDispose = null }
  }
}

/**
 * Wire a `{#while-error err of sig}` boundary found in SSR'd DOM.
 *
 * Mirror of `wireWhileLoading` for the error state. The render
 * callback receives the error value from the signal envelope.
 */
export function wireWhileError(
  el: Element,
  binding: Binding,
  ctx: WireContext,
): () => void {
  let current: Element = el
  let branchDispose: (() => void) | null = null

  if (isActive(current)) {
    branchDispose = wireSubtree(current as HTMLElement, ctx, { skipRoot: true })
  }

  const renderFn = binding.renderKey
    ? (ctx.scope[binding.renderKey] as WhileErrorRender | undefined)
    : undefined

  const sig = ctx.scope[binding.expr] as
    | (() => { kind?: string; error?: unknown })
    | undefined
  let firstRun = true

  const condDispose = effect(() => {
    let isError = false
    let errorVal: unknown
    if (sig) {
      try {
        const v = sig()
        isError = !!v && v.kind === 'error'
        errorVal = v?.error
      } catch {}
    }
    if (firstRun) { firstRun = false; return }
    applyTransition(isError, errorVal)
  })

  function applyTransition(show: boolean, errorVal: unknown): void {
    if (branchDispose) { try { branchDispose() } catch {} ; branchDispose = null }
    const parent = current.parentNode
    if (!parent) return
    const owner = current.ownerDocument ?? document
    const next = owner.createElement('sui-while-error') as HTMLElement
    next.setAttribute('key', binding.key)
    if (show) {
      next.style.display = 'contents'
      if (renderFn) {
        const result = renderFn(errorVal)
        for (const node of result.nodes) next.appendChild(node)
        branchDispose = () => { try { result.cleanup() } catch {} }
      }
    } else {
      next.setAttribute('hidden', '')
    }
    parent.replaceChild(next, current)
    current = next
  }

  return () => {
    try { condDispose() } catch {}
    if (branchDispose) { try { branchDispose() } catch {} ; branchDispose = null }
  }
}
