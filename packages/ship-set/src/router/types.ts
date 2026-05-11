// View-router types. See ../../docs/ship-adrs/references/docs/view-directory-convention.md

export type AuthRequirement = 'public' | 'user' | 'admin'

export interface RailEntry {
  label: string
  icon: string
}

export interface ViewManifest {
  /** Stable identity; matches the directory name. */
  name: string
  /** URL pattern this view answers to. Static segments and `[param]` placeholders are supported in v1. */
  path: string
  title?: string
  layout?: string
  auth?: AuthRequirement
  rail?: RailEntry
  preload?: string[]
  events?: string[]
}

export interface RouteEntry {
  manifest: ViewManifest
  /** Absolute path to the view directory (e.g. /repo/views/home). */
  dir: string
  /** View name as the registry sees it (typically the directory stem). */
  name: string
  /** Tag names registered from this view's `_components/`. Empty if the view has none. */
  privateComponents: string[]
}

export interface RouteRegistry {
  routes: RouteEntry[]
}

export interface MatchResult {
  entry: RouteEntry
  /** Captured `[param]` segments, keyed by name without the brackets. */
  params: Record<string, string>
}
