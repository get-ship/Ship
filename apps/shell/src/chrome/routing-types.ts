// Shared route shape used by main.ts + chrome modules (keymap, etc).
// Kept tiny on purpose: routing semantics + helpers stay in main.ts for now;
// extracting this lets chrome modules type-import without circular deps.

export interface Route {
  /** First path segment, e.g. "studio". */
  view: string
  /** Remaining path segments joined by "/", e.g. "agents/abc". */
  sub?: string
}
