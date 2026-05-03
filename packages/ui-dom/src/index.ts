/**
 * @ship/ui-dom — production runtime surface.
 *
 * Default import surface: `attach()`, types, lifecycle helpers.
 * The parser (`hydrate`, `parse`, `directives/`) lives in the
 * `@ship/ui-dom/legacy` subpath for tests and external callers only.
 * Never import legacy from a production view — it adds ~5 KB gzip.
 *
 * @packageDocumentation
 */

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

// Direct-emit hydrators (Phase 3b.2).
//
// The compiler emits per-view/per-component `hydrate(root, ctx)` that
// walks SSR'd DOM via `walk(root, [path])` and dispatches to per-kind
// hydrators below. This is the replacement for the manifest-driven
// walker in `attach/manifest.ts` + `attach/wire.ts` + `attach/bindings/*`.
export {
  walk,
  hydrateText,
  collectTextOpens,
  hydrateAttr,
  hydrateClass,
  hydrateStyle,
  hydrateOn,
  hydrateBindValue,
  hydrateBindChecked,
  hydrateRef,
} from './hydrate'
