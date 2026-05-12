import { join } from 'node:path'
import { mkdir } from 'node:fs/promises'
import { build, type Loader } from 'esbuild'
import type { SuiBundler, SuiBuildConfig, BuildResult, AdapterContext } from '../config.js'
import { resolveEntrypoints, defaultSourcemap } from './util.js'

const ESBUILD_LOADERS = new Set<string>([
  'base64', 'binary', 'copy', 'css', 'dataurl', 'empty', 'file',
  'global-css', 'js', 'json', 'jsx', 'local-css', 'text', 'ts', 'tsx',
])

export const esbuildAdapter: SuiBundler = {
  name: 'esbuild',

  async build(config: SuiBuildConfig, ctx: AdapterContext): Promise<BuildResult> {
    const t0 = Date.now()
    const entryPoints = resolveEntrypoints(config, ctx)
    const sourcemap = defaultSourcemap(config)

    if (config.loaders) {
      for (const [ext, loader] of Object.entries(config.loaders)) {
        if (!ESBUILD_LOADERS.has(loader)) {
          throw new Error(
            `sui-build esbuild adapter: unsupported loader "${loader}" for "${ext}". Valid loaders: ${[...ESBUILD_LOADERS].sort().join(', ')}`,
          )
        }
      }
    }

    await mkdir(ctx.outdir, { recursive: true })

    const result = await build({
      entryPoints,
      outdir: ctx.outdir,
      bundle: true,
      format: 'esm',
      target: 'es2020',
      minify: config.mode === 'production',
      sourcemap: sourcemap ? 'inline' : false,
      logLevel: 'silent',
      ...(config.loaders ? { loader: config.loaders as Record<string, Loader> } : {}),
    })

    const outputs = Object.keys(entryPoints).map((key) => join(ctx.outdir, `${key}.js`))
    const warnings = result.warnings.map((w) => w.text)

    return { outputs, warnings, durationMs: Date.now() - t0 }
  },
}
