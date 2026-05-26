import { basename, dirname, join } from 'node:path'
import { mkdir } from 'node:fs/promises'
import * as esbuild from 'esbuild'
import type { SuiBundler, SuiBuildConfig, BuildResult, AdapterContext } from '../config.js'
import { resolveEntrypoints, defaultSourcemap } from './util.js'
import { mirrorResolverPlugin } from './import-rewrite.js'

// esbuild's loader vocabulary. `SuiBuildConfig.loaders` is documented to use
// these names verbatim (config.ts), so the map passes straight through.
const ESBUILD_LOADERS = new Set<string>([
  'base64', 'binary', 'copy', 'css', 'dataurl', 'default', 'empty',
  'file', 'js', 'json', 'jsx', 'text', 'ts', 'tsx',
])

/**
 * esbuild bundler adapter — the runtime-portable sibling of `bunAdapter`.
 * Both bundle the same already-compiled entrypoints through the shared
 * `mirrorResolverPlugin` (Bun mirrored esbuild's plugin shape), so this is a
 * drop-in alternative selected via `sui-build --bundler esbuild`. It does NOT
 * remove the bun path — bun stays the default for dev.
 */
export const esbuildAdapter: SuiBundler = {
  name: 'esbuild',

  async build(config: SuiBuildConfig, ctx: AdapterContext): Promise<BuildResult> {
    if (config.loaders) {
      for (const [ext, loader] of Object.entries(config.loaders)) {
        if (!ESBUILD_LOADERS.has(loader)) {
          throw new Error(
            `sui-build esbuild adapter: unsupported loader "${loader}" for "${ext}". Valid loaders: ${[...ESBUILD_LOADERS].sort().join(', ')}`,
          )
        }
      }
    }

    const t0 = Date.now()
    const entries = resolveEntrypoints(config, ctx)
    const sourcemap = defaultSourcemap(config)
    const outputs: string[] = []
    const warnings: string[] = []
    const isProd = config.mode === 'production'
    // Match bun's NODE_ENV-driven Lit export selection: production builds take
    // Lit's default (production) export; dev builds add the `development`
    // condition so the dev-mode warnings/asserts resolve. `define` keeps the
    // inline `process.env.NODE_ENV` checks consistent with the bun adapter.
    const nodeEnv = isProd ? 'production' : 'development'

    for (const [key, entrypoint] of Object.entries(entries)) {
      // Keys with a `/` denote subdirectory placement (lazy bundles routed via
      // `lazyOutSubdir`). Compute per-entry outfile so output naming matches
      // the bun adapter byte-for-byte at the directory level.
      const subDir = dirname(key)
      const baseName = basename(key)
      const entryOutdir = subDir === '.' ? ctx.outdir : join(ctx.outdir, subDir)
      if (subDir !== '.') await mkdir(entryOutdir, { recursive: true })
      const outfile = join(entryOutdir, `${baseName}.js`)

      let result: esbuild.BuildResult
      try {
        result = await esbuild.build({
          entryPoints: [entrypoint],
          outfile,
          bundle: true,
          format: 'esm',
          platform: 'browser',
          target: 'esnext',
          minify: isProd,
          sourcemap: sourcemap ? 'inline' : false,
          define: { 'process.env.NODE_ENV': JSON.stringify(nodeEnv) },
          plugins: [mirrorResolverPlugin(ctx.packageRoot)],
          logLevel: 'silent',
          write: true,
          ...(isProd ? {} : { conditions: ['development'] }),
          ...(config.loaders ? { loader: config.loaders as Record<string, esbuild.Loader> } : {}),
        })
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err)
        throw new Error(`esbuild adapter build failed for entry "${key}": ${msg}`)
      }

      for (const w of result.warnings) {
        warnings.push(`[${key}] ${w.text}`)
      }
      outputs.push(outfile)
    }

    return { outputs, warnings, durationMs: Date.now() - t0 }
  },
}
