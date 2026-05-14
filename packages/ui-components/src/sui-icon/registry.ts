// Lucide icon registry helpers for `<sui-icon>`.
//
// The lucide table itself is curated in
// `packages/ship-set/src/web-components/core/lucide-icons.generated.ts`
// and emitted alongside as a JSON asset
// (`crates/sui/ship-sui-server/assets/lucide-icons.json`) served at
// `/_sui/lucide-icons.json`. R4 of the components-bundle audit moved
// the SVG paths out of `sui-components.js` to drop ~13 KB raw / ~2 KB
// gz off the every-page bundle.
//
// Loading model:
//   - Production (browser): the registry loads on first mount of any
//     `<sui-icon>` via `loadIconRegistry()`. Subsequent mounts share
//     the in-flight promise; once resolved, lookups are synchronous.
//     The shell adds a `<link rel="prefetch">` hint so the JSON sits
//     in HTTP cache by the time the first icon mounts.
//   - Tests / SSR: callers prime the cache synchronously via
//     `setIconRegistry(map)`. The package's vitest setup primes from
//     the generated `.ts` so tests render synchronously.

const ALIASES: Record<string, string> = {
  git: 'git-branch',
  refresh: 'refresh-cw',
  stop: 'stop-circle',
  warning: 'triangle-alert',
}

const ICON_REGISTRY_URL = '/_sui/lucide-icons.json'

let registry: Record<string, string> | null = null
let pending: Promise<Record<string, string>> | null = null

export function setIconRegistry(map: Record<string, string>): void {
  registry = map
  pending = null
}

export function getIconRegistry(): Record<string, string> | null {
  return registry
}

export function loadIconRegistry(): Promise<Record<string, string>> {
  if (registry) return Promise.resolve(registry)
  if (pending) return pending
  pending = fetch(ICON_REGISTRY_URL)
    .then((r) => {
      if (!r.ok) throw new Error(`lucide registry fetch ${r.status}`)
      return r.json() as Promise<Record<string, string>>
    })
    .then((map) => {
      registry = map
      return map
    })
    .catch((err) => {
      pending = null
      throw err
    })
  return pending
}

export function resolveIconName(name: string): string | undefined {
  if (!name || !registry) return undefined
  if (registry[name]) return name
  const aliased = ALIASES[name]
  if (aliased && registry[aliased]) return aliased
  return undefined
}

export function getIconSvg(resolved: string): string | undefined {
  return registry ? registry[resolved] : undefined
}
