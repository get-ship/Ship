import { resolve } from 'node:path'
import type { SuiBuildConfig, AdapterContext } from '../config.js'

/** Merges entries + lazyBundles and resolves all source paths to absolute. */
export function resolveEntrypoints(
  config: SuiBuildConfig,
  ctx: AdapterContext,
): Record<string, string> {
  const merged = { ...config.entries, ...(config.lazyBundles ?? {}) }
  return Object.fromEntries(
    Object.entries(merged).map(([key, src]) => [key, resolve(ctx.packageRoot, src)]),
  )
}

/** Returns whether sourcemaps should be emitted for this config. */
export function defaultSourcemap(config: SuiBuildConfig): boolean {
  if (config.sourcemap !== undefined) return config.sourcemap
  return config.mode !== 'production'
}
