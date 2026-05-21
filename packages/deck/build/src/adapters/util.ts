import { resolve } from 'node:path'
import type { SuiBuildConfig, AdapterContext } from '../config.js'

/** Merges entries + lazyBundles and resolves all source paths to absolute.
 *  Lazy-bundle keys are prefixed with `lazyOutSubdir` so the resulting key
 *  doubles as the relative output path under `outdir` — adapters that key
 *  outputs by file name (esbuild's entryPoints object, the bun loop) get
 *  subdirectory placement for free. */
export function resolveEntrypoints(
  config: SuiBuildConfig,
  ctx: AdapterContext,
): Record<string, string> {
  const eager = Object.entries(config.entries).map(
    ([key, src]) => [key, resolve(ctx.packageRoot, src)] as const,
  )
  const subdir = config.lazyOutSubdir?.replace(/\/+$/, '') ?? ''
  const lazy = Object.entries(config.lazyBundles ?? {}).map(
    ([key, src]) =>
      [subdir ? `${subdir}/${key}` : key, resolve(ctx.packageRoot, src)] as const,
  )
  return Object.fromEntries([...eager, ...lazy])
}

/** Returns whether sourcemaps should be emitted for this config. */
export function defaultSourcemap(config: SuiBuildConfig): boolean {
  if (config.sourcemap !== undefined) return config.sourcemap
  return config.mode !== 'production'
}
