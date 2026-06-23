// Layout chain resolution — host-side wiring for the LAYOUTS table
// emitted by `/home/madvib/dev/sui//home/madvib/dev/sui/crates/deck/deck-bundler/src/routes.rs::generate_routes`.
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
 * The function is pure: same inputs, same outputs. The host calls it
 * at navigation time, then builds the chrome chain, then mounts.
 */
export function resolveLayoutChain(
  url: string | URL,
  layouts: readonly LayoutShape[],
): LayoutShape[] {
  const pathname = pathnameOf(url)
  const matches: LayoutShape[] = []
  for (const layout of layouts) {
    if (!layout || typeof layout.pathPrefix !== 'string' || layout.pathPrefix.length === 0) {
      continue
    }
    if (matchPrefix(pathname, layout.pathPrefix)) {
      matches.push(layout)
    }
  }
  matches.sort((a, b) => b.pathPrefix.length - a.pathPrefix.length)
  return matches
}

/**
 * Match a pathname against a layout prefix. The prefix matches if:
 *  - pathname === prefix (exact match), or
 *  - pathname starts with prefix + '/' (e.g. `/studio/agents` matches `/studio`), or
 *  - prefix === '/' (matches every pathname).
 */
function matchPrefix(pathname: string, prefix: string): boolean {
  if (prefix === '/') return true
  if (pathname === prefix) return true
  return pathname.startsWith(prefix + '/')
}

/** Extract the pathname from a URL string or URL object. */
function pathnameOf(url: string | URL): string {
  if (typeof url === 'string') {
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
