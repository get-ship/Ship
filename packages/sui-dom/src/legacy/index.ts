/**
 * @ship/sui-dom/legacy — parse-on-the-client path.
 *
 * Retained for tests and external callers. Production views use
 * `@ship/sui-dom` (the default surface) which only exports `attach()`.
 * Importing from this subpath pulls the full parser (~5 KB gzip) into
 * the bundle; never import it from a production view.
 */

export { hydrate } from './hydrate'
export type { HydrateOptions, HydrateHandle } from './hydrate'

export { parseSetSource } from './parse'
export type {
  ParsedSource,
  DataDecl,
  StateDecl,
  ComponentDecl,
  PropDecl,
} from './parse'
