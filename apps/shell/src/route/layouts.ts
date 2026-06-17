// Layout chain resolution — host-side wiring for the LAYOUTS table
// emitted by `crates/deck/deck-bundler/src/routes.rs::generate_routes`.
//
// The runtime doesn't need a new feature: layouts are nested web
// components with light-DOM slot distribution (see
// `packages/sui/runtime/src/slot.ts::resolveSlots`). The host walks
// the LAYOUTS table for the current URL, builds the chrome chain, and
// mounts the route content into the innermost `<div slot="route">`.
//
// The shape of `LayoutShape` mirrors the generated `routes.gen.ts`.
//   export const LAYOUTS: LayoutShape[] = [
//     { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
//   ]
//   export type LayoutShape = { pathPrefix: string; tag: string }

export interface LayoutShape {
  /** URL pathname prefix that triggers this layout. */
  pathPrefix: string
  /** Custom-element tag to mount (the chrome wrapper). */
  tag: string
}

/**
 * Walk LAYOUTS for the current URL. Longest-prefix match wins.
 *
 * Returns the ordered list of chrome components to wrap the route
 * content in, **deepest-first**. The first element of the returned
 * array is the innermost chrome (closest to the route content); the
 * last is the outermost chrome (closest to the document root).
 *
 * An empty return means "no chrome — mount the route content directly
 * into the root element."
 *
 * Examples:
 *
 *   LAYOUTS = [{ pathPrefix: '/studio', tag: 'sui-studio-chrome' }]
 *   resolveLayoutChain('/studio/agents/abc', LAYOUTS)
 *     => [{ pathPrefix: '/studio', tag: 'sui-studio-chrome' }]
 *
 *   LAYOUTS = [
 *     { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
 *     { pathPrefix: '/studio/agents', tag: 'sui-agents-chrome' },
 *   ]
 *   resolveLayoutChain('/studio/agents/abc', LAYOUTS)
 *     => [
 *       { pathPrefix: '/studio/agents', tag: 'sui-agents-chrome' },
 *       { pathPrefix: '/studio', tag: 'sui-studio-chrome' },
 *     ]
 *
 *   resolveLayoutChain('/home', []) => []
 *   resolveLayoutChain('/', LAYOUTS) => []
 *
 * The function is pure: same inputs, same outputs. The host calls it
 * at navigation time, then builds the chrome chain, then mounts.
 */
export function resolveLayoutChain(
  url: string | URL,
  layouts: readonly LayoutShape[],
): LayoutShape[] {
  const pathname = pathnameOf(url)
  // Filter, then sort by prefix length descending (longest first).
  const matches: LayoutShape[] = []
  for (const layout of layouts) {
    if (!layout || typeof layout.pathPrefix !== 'string' || layout.pathPrefix.length === 0) {
      continue
    }
    if (pathname === layout.pathPrefix || pathname.startsWith(layout.pathPrefix + '/') || (layout.pathPrefix === '/' && pathname === '/')) {
      matches.push(layout)
    }
  }
  matches.sort((a, b) => b.pathPrefix.length - a.pathPrefix.length)
  return matches
}

/** Extract the pathname from a URL string or URL object. */
function pathnameOf(url: string | URL): string {
  if (typeof url === 'string') {
    // Tolerate relative URLs by treating them as already a pathname.
    if (url.startsWith('http://') || url.startsWith('https://') || url.startsWith('file://')) {
      try {
        return new URL(url).pathname
      } catch {
        return url
      }
    }
    return url.split('?')[0].split('#')[0] || '/'
  }
  return url.pathname
}
