/**
 * @ship/ui-dom — public API for the Phase-2 directive interpreter.
 *
 * The interpreter reads `.sui` source at runtime, hydrates it
 * against the `@ship/ui-runtime` primitives, and returns a disposer.
 * This is the validation harness for the runtime semantics; the
 * Phase-4 compiler will eventually emit equivalent calls statically.
 *
 * Surface map:
 *   - `hydrate(template, setup?, opts)` — entry point.
 *   - `loadSetup(url)` — resolve a paired `.ts` setup file.
 *   - `parseSetSource(src)` — exposed for tooling that wants the raw
 *     directive list (devserver, error overlay).
 *
 * @packageDocumentation
 */

export { hydrate } from './hydrate'
export type { HydrateOptions, HydrateHandle } from './hydrate'

export { attach } from './attach'
export type {
  AttachOptions,
  AttachDisposer,
  Binding,
  BindingKind,
  SetCompiledModule,
  SetupContext,
} from './attach'

export { loadSetup, pairedTsPath } from './setup-loader'

export { parseSetSource } from './parse'
export type {
  ParsedSource,
  DataDecl,
  StateDecl,
  ComponentDecl,
  PropDecl,
} from './parse'
