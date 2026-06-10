// Shared mount harness for the arcade pack's behavioral specs.
//
// Each test file still declares its own `vi.mock('@ship/arcade-engine', …)`
// (the mock is hoisted per-module and must run before the controller import),
// then imports these helpers to compile + mount <sui-stage> with light-DOM
// children and settle the async createStage mount deterministically.

import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { renderComponent } from '@ship/test-setup'
import { compileSui, evalCompiledJs } from '@ship/sui-dom/test-utils'
import stageController from '../src/sui-stage/sui-stage.controller.ts'

export interface StageHost extends HTMLElement {
  start(): void
  pause(): void
  resume(): void
  reset(): void
  fire(action: string, detail?: unknown): void
  bind(name: string, sig: unknown): void
  set(name: string, value: unknown): void
  stage?: unknown
}

// Read from the package root (vitest cwd). `import.meta.url` is unreliable in
// files that mock @ship/arcade-engine: importActual drags the vendored Pixi
// module through vite's transform, leaving import.meta.url as a non-file URL.
function srcFile(rel: string): string {
  return readFileSync(resolve(process.cwd(), 'src', rel), 'utf8')
}

export const STAGE_SUI = srcFile('sui-stage/sui-stage.sui')

/**
 * Compile + register one of the pack's child custom elements (sui-sprite,
 * sui-hud, …) under its own tag. The child tags are markerless declarative
 * elements parsed by the stage from light DOM — they only become registered
 * custom elements when their own `.sui` is compiled. Idempotent per tag.
 */
export function registerChild(tag: string): void {
  if (customElements.get(tag)) return
  const compiled = compileSui(srcFile(`${tag}/${tag}.sui`), { componentName: tag, mode: 'component' })
  evalCompiledJs(compiled.js)
}

/**
 * Mount <sui-stage> with optional light-DOM `inner` children + attrs, then
 * drive a handful of macrotasks so the controller's async createStage mount
 * (palette resolve, child wiring, ready event) fully settles. `onHost` runs
 * synchronously before the first flush — use it to attach listeners that must
 * observe the async `sui-stage:ready` / first state events.
 */
export async function mountStage(
  inner = '',
  attrs: Record<string, string> = {},
  onHost?: (host: StageHost) => void,
) {
  const handle = renderComponent(STAGE_SUI, 'sui-stage', { controller: stageController, attrs })
  const host = handle.el as StageHost
  if (inner) host.innerHTML = inner
  onHost?.(host)
  for (let i = 0; i < 4; i++) await handle.flush()
  return { handle, host }
}
