// Test helpers for the attach runtime suite.
//
// SSR fixtures are hand-written: the daemon's Rust SSR walker would
// produce equivalent HTML for each `.sui` source, but pulling the
// compiler into vitest is too heavy. The shipd integration test
// already covers the real end-to-end path.

import { signal, type Signal } from '@ship/ui-runtime'
import type { SetCompiledModule, SetupContext } from '../../src/attach'

const containers: HTMLElement[] = []

/** Create a fresh container appended to the document body. */
export function mkContainer(): HTMLElement {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}

/** Tear down every container created during the test. */
export function clearContainers(): void {
  for (const c of containers.splice(0)) c.remove()
}

/** Mount SSR'd HTML + JSON island into a fresh container. */
export function mountSsr(html: string, init: Record<string, unknown>): HTMLElement {
  const c = mkContainer()
  const islandJson = JSON.stringify(init).replace(/<\//g, '<\\/')
  c.innerHTML =
    html +
    `<script type="application/json" id="__set_init__">${islandJson}</script>`
  return c
}

/** Build a compiled-module shape from a hand-written setup function. */
export function makeModule(
  setup: (ctx: SetupContext) => Record<string, unknown>,
  _bindings?: unknown[],
): SetCompiledModule {
  return { default: { setup } }
}

/** Yield to the microtask queue. */
export const flush = (): Promise<void> =>
  new Promise((r) => {
    setTimeout(r, 0)
  })

/** Pluck a state signal from the resolver-friendly setup ctx. */
export function getStateSignal(
  ctx: SetupContext,
  name: string,
): Signal<unknown> {
  const v = ctx.scope?.[name]
  if (typeof v !== 'function' || !('set' in (v as object))) {
    throw new Error(`expected state signal '${name}' in scope`)
  }
  return v as Signal<unknown>
}

/** Construct a writable signal alias outside an existing scope. */
export function aliasSignal<T>(initial: T): Signal<T> {
  return signal<T>(initial)
}
