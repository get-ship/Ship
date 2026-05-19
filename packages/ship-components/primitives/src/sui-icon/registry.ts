// Lucide icon registry helpers for `<sui-icon>`.
//
// The lucide table is emitted as a JSON asset
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

// Pin state to globalThis so the singleton survives vitest's per-file
// module transforms: the controller is loaded via the mirror-tree
// `.compiled.js` (which transforms it through one path), while the
// test setup primes the registry via a direct relative import — under
// jsdom these can end up as separate module instances unless we stash
// state on a shared global. Production browsers see a single module
// regardless; this is purely a test-safety belt.
type RegistryState = {
  map: Record<string, string> | null
  pending: Promise<Record<string, string>> | null
}
const GLOBAL_KEY = '__shipIconRegistry__'
const g = globalThis as unknown as { [GLOBAL_KEY]?: RegistryState }
function state(): RegistryState {
  if (!g[GLOBAL_KEY]) g[GLOBAL_KEY] = { map: null, pending: null }
  return g[GLOBAL_KEY]
}

export function setIconRegistry(map: Record<string, string>): void {
  const s = state()
  s.map = map
  s.pending = null
}

export function getIconRegistry(): Record<string, string> | null {
  return state().map
}

export function loadIconRegistry(): Promise<Record<string, string>> {
  const s = state()
  if (s.map) return Promise.resolve(s.map)
  if (s.pending) return s.pending
  s.pending = fetch(ICON_REGISTRY_URL)
    .then((r) => {
      if (!r.ok) throw new Error(`lucide registry fetch ${r.status}`)
      return r.json() as Promise<Record<string, string>>
    })
    .then((map) => {
      s.map = map
      return map
    })
    .catch((err) => {
      s.pending = null
      throw err
    })
  return s.pending
}

export function resolveIconName(name: string): string | undefined {
  const map = state().map
  if (!name || !map) return undefined
  if (map[name]) return name
  const aliased = ALIASES[name]
  if (aliased && map[aliased]) return aliased
  return undefined
}

export function getIconSvg(resolved: string): string | undefined {
  return state().map?.[resolved]
}
