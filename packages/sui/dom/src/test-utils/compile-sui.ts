// WASM-backed test helper. Compiles `.sui` source through the same
// Rust compiler that production uses, then evaluates the emitted
// module in-process so vitest tests assert against the production
// emit shape rather than a parallel JS parser.
//
// The legacy `@ship/sui-dom/legacy` path is gone — every directive
// must round-trip through Rust now. This helper is the only test-time
// bridge between `.sui` source and a mounted DOM tree.

import { readFileSync, statSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { initSync, compile as wasmCompile } from '@ship/sui-compiler-wasm'

import * as suiRuntime from '@ship/sui-runtime'
import * as suiDom from '@ship/sui-dom'
import * as suiRoute from '@ship/sui-runtime/route'
import { route as makeRoute, type RouteSignals } from '@ship/sui-runtime/route'
import '@ship/sui-runtime/template-hooks'

/* ----------------------------- wasm init ----------------------------- */

let wasmReady = false
function ensureWasm(): void {
  if (wasmReady) return
  const here = dirname(fileURLToPath(import.meta.url))
  const wasmPath = resolve(here, '../../../compiler-wasm/pkg/sui_compiler_bg.wasm')
  warnIfStale(wasmPath, here)
  const bytes = readFileSync(wasmPath)
  initSync({ module: bytes })
  wasmReady = true
}

// The wasm pkg embeds the primitives registry at build time. If primitives.json
// has changed since the pkg was built, the harness validates `.sui` against a
// stale registry — silently rejecting valid props or accepting removed ones.
// Warn loudly so the fix (`just sui-wasm`) is obvious rather than mysterious.
function warnIfStale(wasmPath: string, here: string): void {
  try {
    const registry = resolve(here, '../../../../../crates/sui/compiler/primitives.json')
    const wasmMtime = statSync(wasmPath).mtimeMs
    const regMtime = statSync(registry).mtimeMs
    if (regMtime > wasmMtime) {
      const days = Math.round((regMtime - wasmMtime) / 86_400_000)
      console.warn(
        `[sui-compiler-wasm] STALE: primitives.json is newer than the wasm pkg ` +
          `(~${days}d). The harness is validating against an out-of-date registry. ` +
          `Run \`just sui-wasm\` to rebuild.`,
      )
    }
  } catch {
    /* registry not locatable (published pkg / different layout) — skip the check */
  }
}

/* ----------------------------- compile API ----------------------------- */

export interface CompileOpts {
  mode?: 'view' | 'component' | 'auto'
  componentName?: string
  /** Marker passed to the compiler as the setup_url; eval rewrites the
   *  resulting `import setup from "<marker>"` to the injected setup. */
  setupMarker?: string
}

export interface CompiledSui {
  js: string
  css: string | null
  dts: string | null
  ssrHtml: string | null
}

export function compileSui(source: string, opts: CompileOpts = {}): CompiledSui {
  ensureWasm()
  const optsJson = JSON.stringify({
    mode: opts.mode ?? 'auto',
    componentName: opts.componentName,
    setupUrl: opts.setupMarker,
  })
  const result = wasmCompile(source, optsJson) as
    | { kind: 'ok'; js: string; css: string | null; dts: string | null; ssrHtml: string | null }
    | { kind: 'err'; message: string; line: number | null; col: number | null; category: string | null }
  if (result.kind === 'err') {
    const where = result.line != null ? ` (${result.line}:${result.col ?? 0})` : ''
    throw new Error(`sui-compiler: ${result.message}${where}`)
  }
  return { js: result.js, css: result.css, dts: result.dts, ssrHtml: result.ssrHtml }
}

/* ----------------------------- eval API ----------------------------- */

const SETUP_MARKER = '__test_setup__'

type UserSetup = ((ctx: unknown) => Record<string, unknown> | void) | { setup: (ctx: unknown) => unknown } | undefined

interface EvalDeps {
  __runtime: typeof suiRuntime
  __dom: typeof suiDom
  __route: typeof suiRoute
  __setup: { setup: (ctx: unknown) => unknown } | null
}

function stripImports(js: string): string {
  return js
    // import { …names… } from "@ship/sui-runtime"
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sui-runtime["']\s*;?\s*$/gm,
      'const { $1 } = __runtime;',
    )
    // import { …names… } from "@ship/sui-dom"
    .replace(
      /^\s*import\s*\{\s*([^}]+)\s*\}\s*from\s*["']@ship\/sui-dom["']\s*;?\s*$/gm,
      'const { $1 } = __dom;',
    )
    // import { route as __routeFactory } from "@ship/sui-runtime/route"
    .replace(
      /^\s*import\s*\{\s*route\s+as\s+__routeFactory\s*\}\s*from\s*["']@ship\/sui-runtime\/route["']\s*;?\s*$/gm,
      'const __routeFactory = __route.route;',
    )
    // side-effect: import "@ship/sui-runtime"
    .replace(/^\s*import\s+["']@ship\/sui-runtime["']\s*;?\s*$/gm, '')
    // side-effect: import "@ship/sui-runtime/template-hooks"
    .replace(/^\s*import\s+["']@ship\/sui-runtime\/template-hooks["']\s*;?\s*$/gm, '')
    // import setup from "<marker>"
    .replace(/^\s*import\s+setup\s+from\s+["'][^"']+["']\s*;?\s*$/gm, 'const setup = __setup;')
}

/** Evaluate a compiled JS module body and return its default export. */
export function evalCompiledJs(
  js: string,
  setup?: UserSetup,
): { setup: (ctx: unknown) => Record<string, unknown> & { __mount: (c: HTMLElement) => () => void } } {
  const body = stripImports(js).replace(/\bexport\s+default\s+/g, 'return ')
  const setupObj = wrapSetup(setup)
  const fn = new Function('__runtime', '__dom', '__route', '__setup', body)
  const deps: EvalDeps = {
    __runtime: suiRuntime,
    __dom: suiDom,
    __route: suiRoute,
    __setup: setupObj,
  }
  return fn(deps.__runtime, deps.__dom, deps.__route, deps.__setup) as ReturnType<typeof evalCompiledJs>
}

function wrapSetup(setup: UserSetup): { setup: (ctx: unknown) => unknown } | null {
  if (!setup) return null
  if (typeof setup === 'function') {
    return { setup: (ctx) => setup(ctx) ?? {} }
  }
  return setup
}

/* ----------------------------- mount API ----------------------------- */

export interface MountOpts {
  container: HTMLElement
  ship?: unknown
  route?: RouteSignals
  navigate?: (path: string) => void
  /** When the source declares `<prop>`s, compile as a custom element
   *  with this tag name. The compiled module's top-level
   *  `customElements.define(<name>, …)` registers the element; the
   *  test then instantiates it via `document.createElement(<name>)`. */
  componentName?: string
}

export interface MountHandle {
  dispose: () => void
}

/** Compile + eval + mount in one call. Mirrors the legacy `hydrate(template, setup, opts)` shape. */
export function compileAndMount(source: string, setup: UserSetup, opts: MountOpts): MountHandle {
  const usesSetup = setup != null
  const compileOpts: CompileOpts = usesSetup ? { setupMarker: SETUP_MARKER } : {}
  if (opts.componentName) {
    compileOpts.componentName = opts.componentName
    compileOpts.mode = 'component'
  }
  const compiled = compileSui(source, compileOpts)

  // Component mode: the compiled module is a side-effecting
  // `component(tag, def)` call that registers a custom element. No
  // setup/mount cycle — the test instantiates the element directly
  // via `document.createElement(tag)`.
  if (opts.componentName) {
    evalCompiledJs(compiled.js, setup)
    return { dispose: () => {} }
  }

  const def = evalCompiledJs(compiled.js, setup)
  const ctx = {
    ship: (opts.ship ?? {}) as suiRuntime.ShipSDK,
    route: opts.route ?? makeRoute(new URL('http://localhost/')),
    navigate: opts.navigate ?? (() => {}),
  }
  const scope = def.setup(ctx)
  const before = new Set(Array.from(opts.container.childNodes))
  const cleanup = scope.__mount(opts.container)
  return {
    dispose: () => {
      try { cleanup() } catch {}
      // Mirror legacy hydrate: remove every node mount appended so
      // `container.children` is empty after dispose.
      for (const n of Array.from(opts.container.childNodes)) {
        if (!before.has(n)) n.parentNode?.removeChild(n)
      }
    },
  }
}
