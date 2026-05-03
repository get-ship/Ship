/**
 * @ship/ui-dom — DOM-walk hydrators.
 *
 * F1 mount unification removed `attach()` — hosts call
 * `mount(target, payload)` from `@ship/ui-runtime` instead. ui-dom
 * now exists only to publish the per-kind hydrators the compiler-
 * emitted `hydrate(root, ctx)` calls during SSR resume.
 *
 * The parser (`hydrate`, `parse`, `directives/`) lives in the
 * `@ship/ui-dom/legacy` subpath for tests and external callers only.
 * Never import legacy from a production view — it adds ~5 KB gzip.
 *
 * @packageDocumentation
 */

export { loadSetup, pairedTsPath } from './setup-loader'

// Direct-emit hydrators (Phase 3b.2).
//
// The compiler emits per-view/per-component `hydrate(root, ctx)` that
// walks SSR'd DOM via `walk(root, [path])` and dispatches to per-kind
// hydrators below.
export {
  walk,
  hydrateText,
  collectTextOpens,
  collectElementMarkers,
  hydrateAttr,
  hydrateClass,
  hydrateStyle,
  hydrateOn,
  hydrateBindValue,
  hydrateBindChecked,
  hydrateRef,
  hydrateWhileLoading,
  hydrateWhileError,
} from './hydrate'
