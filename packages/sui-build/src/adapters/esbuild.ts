import { join } from 'node:path'
import { mkdir } from 'node:fs/promises'
import { build } from 'esbuild'
import type { SuiBundler, SuiBuildConfig, BuildResult, AdapterContext } from '../config.js'
import { resolveEntrypoints, defaultSourcemap } from './util.js'

export const esbuildAdapter: SuiBundler = {
  name: 'esbuild',

  async build(config: SuiBuildConfig, ctx: AdapterContext): Promise<BuildResult> {
    const t0 = Date.now()
    const entryPoints = resolveEntrypoints(config, ctx)
    const sourcemap = defaultSourcemap(config)

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
    })

    const outputs = Object.keys(entryPoints).map((key) => join(ctx.outdir, `${key}.js`))
    const warnings = result.warnings.map((w) => w.text)

    return { outputs, warnings, durationMs: Date.now() - t0 }
  },
}
