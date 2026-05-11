// View router — walks `views/<name>/manifest.jsonc` files into a route
// registry and resolves URL pathnames to view entries. See
// docs/ship-adrs/references/docs/view-directory-convention.md for the
// directory shape this router consumes.

export { parseJsonc } from './jsonc'
export { parseManifest, validateManifest, ManifestError } from './manifest'
export { loadRouteRegistry, RegistryError } from './registry'
export { matchRoute } from './match'
export type {
  AuthRequirement,
  MatchResult,
  RailEntry,
  RouteEntry,
  RouteRegistry,
  ViewManifest,
} from './types'
