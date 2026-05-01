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
